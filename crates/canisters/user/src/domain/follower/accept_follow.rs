//! Domain logic for the `accept_follow` flow.
//!
//! The flow consists of:
//! 1. Look up the follow request by actor URI in the `follow_requests` table.
//! 2. Build an `Accept(Follow)` activity.
//! 3. Send the activity to the Federation Canister.
//! 4. On success, in a single transaction: delete the follow request and insert
//!    the follower.

use activitypub::activity::{Activity, ActivityObject, ActivityType};
use activitypub::context::ACTIVITY_STREAMS_CONTEXT;
use activitypub::object::{BaseObject, OneOrMany};
use did::federation::{SendActivityArgs, SendActivityArgsObject};
use did::user::{AcceptFollowArgs, AcceptFollowError, AcceptFollowResponse};
use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::*;

use crate::domain::follow_request::FollowRequestRepository;
use crate::domain::profile::ProfileRepository;
use crate::error::{CanisterError, CanisterResult};
use crate::schema::{FollowRequest, Follower, FollowerInsertRequest, Schema};

/// Execute the accept-follow flow.
pub async fn accept_follow(
    AcceptFollowArgs { actor_uri }: AcceptFollowArgs,
) -> AcceptFollowResponse {
    ic_utils::log!("accept_follow: accepting follow from {actor_uri}");

    match accept_follow_inner(&actor_uri).await {
        Ok(()) => AcceptFollowResponse::Ok,
        Err(AcceptFollowDomainError::RequestNotFound) => {
            AcceptFollowResponse::Err(AcceptFollowError::RequestNotFound)
        }
        Err(AcceptFollowDomainError::Internal(e)) => {
            ic_utils::log!("accept_follow: error: {e}");
            AcceptFollowResponse::Err(AcceptFollowError::Internal(e))
        }
    }
}

enum AcceptFollowDomainError {
    RequestNotFound,
    Internal(String),
}

impl From<CanisterError> for AcceptFollowDomainError {
    fn from(e: CanisterError) -> Self {
        Self::Internal(e.to_string())
    }
}

async fn accept_follow_inner(actor_uri: &str) -> Result<(), AcceptFollowDomainError> {
    // check that the follow request exists
    if FollowRequestRepository::find_by_actor_uri(actor_uri)?.is_none() {
        ic_utils::log!("accept_follow: no pending request from {actor_uri}");
        return Err(AcceptFollowDomainError::RequestNotFound);
    }

    // build own actor URI
    let own_profile = ProfileRepository::oneshot().get_profile()?;
    let own_actor_uri = crate::domain::urls::actor_uri(&own_profile.handle.0)?;

    // build and send Accept(Follow) activity
    let activity = make_accept_activity(&own_actor_uri, actor_uri);
    let target_inbox = crate::domain::urls::inbox_url_from_actor_uri(actor_uri);
    let args = SendActivityArgs::One(SendActivityArgsObject {
        activity_json: serde_json::to_string(&activity)
            .expect("Activity serialization must not fail"),
        target_inbox,
    });

    crate::adapters::federation::send_activity(args).await?;

    // on success, atomically: delete follow request + insert follower
    accept_follow_transaction(actor_uri)?;

    Ok(())
}

/// Atomically delete the follow request and insert the follower.
fn accept_follow_transaction(follower_uri: &str) -> CanisterResult<()> {
    DBMS_CONTEXT.with(|ctx| {
        let tx_id =
            ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
        let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);

        db.delete::<FollowRequest>(
            DeleteBehavior::Restrict,
            Some(Filter::eq(
                "actor_uri",
                Value::from(follower_uri.to_string()),
            )),
        )?;

        db.insert::<Follower>(FollowerInsertRequest {
            actor_uri: follower_uri.into(),
            created_at: ic_utils::now().into(),
        })?;

        db.commit()?;

        Ok(())
    })
}

/// Build an `Accept(Follow)` [`Activity`].
fn make_accept_activity(own_actor_uri: &str, follower_actor_uri: &str) -> Activity {
    Activity {
        base: BaseObject {
            context: Some(activitypub::context::Context::Uri(
                ACTIVITY_STREAMS_CONTEXT.to_string(),
            )),
            kind: ActivityType::Accept,
            to: Some(OneOrMany::One(follower_actor_uri.to_string())),
            ..Default::default()
        },
        actor: Some(own_actor_uri.to_string()),
        object: Some(ActivityObject::Activity(Box::new(Activity {
            base: BaseObject {
                kind: ActivityType::Follow,
                ..Default::default()
            },
            actor: Some(follower_actor_uri.to_string()),
            object: Some(ActivityObject::Id(own_actor_uri.to_string())),
            target: None,
            result: None,
            origin: None,
            instrument: None,
        }))),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::domain::follower::FollowerRepository;
    use crate::test_utils::setup;

    #[tokio::test]
    async fn test_should_accept_follow_request() {
        setup();

        // insert a pending follow request
        FollowRequestRepository::insert("https://mastic.social/users/alice")
            .expect("should insert follow request");

        let response = accept_follow(AcceptFollowArgs {
            actor_uri: "https://mastic.social/users/alice".to_string(),
        })
        .await;

        assert_eq!(response, AcceptFollowResponse::Ok);

        // follow request should be removed
        let request =
            FollowRequestRepository::find_by_actor_uri("https://mastic.social/users/alice")
                .expect("should query");
        assert!(request.is_none(), "follow request should be deleted");

        // follower should be added
        let followers = FollowerRepository::get_followers().expect("should query");
        assert_eq!(followers.len(), 1);
        assert_eq!(
            followers[0].actor_uri.0,
            "https://mastic.social/users/alice"
        );
    }

    #[tokio::test]
    async fn test_should_reject_accept_when_no_request() {
        setup();

        let response = accept_follow(AcceptFollowArgs {
            actor_uri: "https://mastic.social/users/nobody".to_string(),
        })
        .await;

        assert_eq!(
            response,
            AcceptFollowResponse::Err(AcceptFollowError::RequestNotFound)
        );
    }

    #[test]
    fn test_make_accept_activity_structure() {
        let own = "https://mastic.social/users/bob";
        let follower = "https://mastic.social/users/alice";
        let activity = make_accept_activity(own, follower);

        assert_eq!(activity.base.kind, ActivityType::Accept);
        assert_eq!(activity.actor.as_deref(), Some(own));
        assert_eq!(activity.base.to, Some(OneOrMany::One(follower.to_string())));

        // object should be a Follow activity
        let ActivityObject::Activity(inner) = activity.object.expect("should have object") else {
            panic!("expected Activity variant");
        };
        assert_eq!(inner.base.kind, ActivityType::Follow);
        assert_eq!(inner.actor.as_deref(), Some(follower));

        let ActivityObject::Id(obj_uri) = inner.object.expect("should have inner object") else {
            panic!("expected Id variant");
        };
        assert_eq!(obj_uri, own);
    }
}
