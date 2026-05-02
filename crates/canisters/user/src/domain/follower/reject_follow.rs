//! Domain logic for the `reject_follow` flow.
//!
//! The flow consists of:
//! 1. Look up the follow request by actor URI in the `follow_requests` table.
//! 2. Build a `Reject(Follow)` activity.
//! 3. Send the activity to the Federation Canister.
//! 4. On success, delete the follow request.

use activitypub::activity::{Activity, ActivityObject, ActivityType};
use activitypub::context::ACTIVITY_STREAMS_CONTEXT;
use activitypub::object::{BaseObject, OneOrMany};
use did::federation::{SendActivityArgs, SendActivityArgsObject};
use did::user::{RejectFollowArgs, RejectFollowError, RejectFollowResponse};

use crate::domain::follow_request::FollowRequestRepository;
use crate::domain::profile::ProfileRepository;
use crate::error::CanisterError;

/// Execute the reject-follow flow.
pub async fn reject_follow(
    RejectFollowArgs { actor_uri }: RejectFollowArgs,
) -> RejectFollowResponse {
    ic_utils::log!("reject_follow: rejecting follow from {actor_uri}");

    match reject_follow_inner(&actor_uri).await {
        Ok(()) => RejectFollowResponse::Ok,
        Err(RejectFollowDomainError::RequestNotFound) => {
            RejectFollowResponse::Err(RejectFollowError::RequestNotFound)
        }
        Err(RejectFollowDomainError::Internal(e)) => {
            ic_utils::log!("reject_follow: error: {e}");
            RejectFollowResponse::Err(RejectFollowError::Internal(e))
        }
    }
}

enum RejectFollowDomainError {
    RequestNotFound,
    Internal(String),
}

impl From<CanisterError> for RejectFollowDomainError {
    fn from(e: CanisterError) -> Self {
        Self::Internal(e.to_string())
    }
}

async fn reject_follow_inner(actor_uri: &str) -> Result<(), RejectFollowDomainError> {
    // check that the follow request exists
    if FollowRequestRepository::oneshot()
        .find_by_actor_uri(actor_uri)?
        .is_none()
    {
        ic_utils::log!("reject_follow: no pending request from {actor_uri}");
        return Err(RejectFollowDomainError::RequestNotFound);
    }

    // build own actor URI
    let own_profile = ProfileRepository::oneshot().get_profile()?;
    let own_actor_uri = crate::domain::urls::actor_uri(&own_profile.handle.0)?;

    // build and send Reject(Follow) activity
    let activity = make_reject_activity(&own_actor_uri, actor_uri);
    let target_inbox = crate::domain::urls::inbox_url_from_actor_uri(actor_uri);
    let args = SendActivityArgs::One(SendActivityArgsObject {
        activity_json: serde_json::to_string(&activity)
            .expect("Activity serialization must not fail"),
        target_inbox,
    });

    crate::adapters::federation::send_activity(args).await?;

    // on success, delete the follow request
    FollowRequestRepository::oneshot().delete_by_actor_uri(actor_uri)?;

    Ok(())
}

/// Build a `Reject(Follow)` [`Activity`].
fn make_reject_activity(own_actor_uri: &str, follower_actor_uri: &str) -> Activity {
    Activity {
        base: BaseObject {
            context: Some(activitypub::context::Context::Uri(
                ACTIVITY_STREAMS_CONTEXT.to_string(),
            )),
            kind: ActivityType::Reject,
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
    use crate::test_utils::setup;

    #[tokio::test]
    async fn test_should_reject_follow_request() {
        setup();

        // insert a pending follow request
        FollowRequestRepository::oneshot()
            .insert("https://mastic.social/users/alice")
            .expect("should insert follow request");

        let response = reject_follow(RejectFollowArgs {
            actor_uri: "https://mastic.social/users/alice".to_string(),
        })
        .await;

        assert_eq!(response, RejectFollowResponse::Ok);

        // follow request should be removed
        let request = FollowRequestRepository::oneshot()
            .find_by_actor_uri("https://mastic.social/users/alice")
            .expect("should query");
        assert!(request.is_none(), "follow request should be deleted");
    }

    #[tokio::test]
    async fn test_should_reject_when_no_request() {
        setup();

        let response = reject_follow(RejectFollowArgs {
            actor_uri: "https://mastic.social/users/nobody".to_string(),
        })
        .await;

        assert_eq!(
            response,
            RejectFollowResponse::Err(RejectFollowError::RequestNotFound)
        );
    }

    #[test]
    fn test_make_reject_activity_structure() {
        let own = "https://mastic.social/users/bob";
        let follower = "https://mastic.social/users/alice";
        let activity = make_reject_activity(own, follower);

        assert_eq!(activity.base.kind, ActivityType::Reject);
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
