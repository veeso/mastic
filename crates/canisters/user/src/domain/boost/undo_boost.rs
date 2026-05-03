//! Flow for undoing a boost of a status.
//!
//! See `.claude/specs/2026-05-02-wi-1-7-boost-status-design.md` ("Undo Flow").
//!
//! Pipeline:
//! 1. Idempotency: if no boost row for `status_url` exists → `Ok` without
//!    dispatching anything.
//! 2. Resolve the wrapper id from the boost row (== `boost.status_id` ==
//!    `boost.id` with the shared-snowflake design).
//! 3. Compute recipients (followers ∪ original author, deduplicated;
//!    excluding self when self == author).
//! 4. In one transaction: delete Boost, FeedEntry, then wrapper Status.
//! 5. Dispatch `Undo(Announce)` to the recipients via the federation
//!    adapter. The inner `Announce` activity reuses
//!    `<own_actor>/statuses/<wrapper_id>` as its id, matching what the
//!    original boost emitted.

use activitypub::Activity;
use activitypub::activity::{ActivityObject, ActivityType};
use activitypub::context::ACTIVITY_STREAMS_CONTEXT;
use activitypub::object::{BaseObject, OneOrMany};
use db_utils::transaction::Transaction;
use did::federation::{SendActivityArgs, SendActivityArgsObject};
use did::user::{UndoBoostArgs, UndoBoostError, UndoBoostResponse};

use crate::domain::boost::repository::BoostRepository;
use crate::domain::follower::FollowerRepository;
use crate::domain::profile::ProfileRepository;
use crate::domain::urls;
use crate::error::{CanisterError, CanisterResult};
use crate::schema::Schema;

const AS_PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";

pub async fn undo_boost(UndoBoostArgs { status_url }: UndoBoostArgs) -> UndoBoostResponse {
    match undo_boost_inner(status_url).await {
        Ok(()) => UndoBoostResponse::Ok,
        Err(err) => {
            ic_utils::log!("Failed to undo boost: {err}");
            UndoBoostResponse::Err(UndoBoostError::Internal(err.to_string()))
        }
    }
}

async fn undo_boost_inner(status_url: String) -> CanisterResult<()> {
    ic_utils::log!("Undoing boost on {status_url}");

    let Some(boost) = BoostRepository::oneshot().find_by_original_uri(&status_url)? else {
        ic_utils::log!("No boost row for {status_url}; idempotent ok");
        return Ok(());
    };

    let wrapper_id = boost.id.expect("id").0;
    let own_profile = ProfileRepository::oneshot().get_profile()?;
    let own_actor_uri = urls::actor_uri(&own_profile.handle.0)?;

    let mut recipients: Vec<String> = FollowerRepository::oneshot()
        .get_followers()?
        .into_iter()
        .map(|f| f.actor_uri.0)
        .collect();
    if let Some(author) = urls::actor_uri_from_status_uri(&status_url)
        && author != own_actor_uri
    {
        recipients.push(author);
    }
    recipients.sort();
    recipients.dedup();

    Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
        BoostRepository::with_transaction(tx).delete_boost_with_wrapper(wrapper_id)
    })?;

    if recipients.is_empty() {
        return Ok(());
    }

    let undo_activity = make_undo_announce(&own_actor_uri, wrapper_id, &status_url, &recipients);
    let undo_json =
        serde_json::to_string(&undo_activity).expect("Activity serialization must not fail");
    let batch: Vec<SendActivityArgsObject> = recipients
        .iter()
        .map(|recipient| SendActivityArgsObject {
            activity_json: undo_json.clone(),
            target_inbox: format!("{recipient}/inbox"),
        })
        .collect();
    crate::adapters::federation::send_activity(SendActivityArgs::Batch(batch)).await?;

    Ok(())
}

fn make_undo_announce(
    own_actor_uri: &str,
    wrapper_id: u64,
    status_url: &str,
    recipients: &[String],
) -> Activity {
    let inner_announce = Activity {
        base: BaseObject {
            id: Some(format!("{own_actor_uri}/statuses/{wrapper_id}")),
            kind: ActivityType::Announce,
            ..Default::default()
        },
        actor: Some(own_actor_uri.to_string()),
        object: Some(ActivityObject::Id(status_url.to_string())),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    };
    let cc = if recipients.is_empty() {
        None
    } else {
        Some(OneOrMany::Many(recipients.to_vec()))
    };
    Activity {
        base: BaseObject {
            context: Some(activitypub::context::Context::Uri(
                ACTIVITY_STREAMS_CONTEXT.to_string(),
            )),
            kind: ActivityType::Undo,
            to: Some(OneOrMany::One(AS_PUBLIC.to_string())),
            cc,
            ..Default::default()
        },
        actor: Some(own_actor_uri.to_string()),
        object: Some(ActivityObject::Activity(Box::new(inner_announce))),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    }
}

#[cfg(test)]
mod tests {
    use activitypub::activity::{ActivityObject, ActivityType};
    use did::common::{Status, Visibility};
    use did::federation::{FetchStatusResponse, SendActivityArgs};
    use did::user::{BoostStatusArgs, BoostStatusResponse, UndoBoostArgs, UndoBoostResponse};

    use super::undo_boost;
    use crate::adapters::federation::mock::{captured, push_fetch_status_response, reset_captured};
    use crate::domain::boost::boost_status::boost_status;
    use crate::domain::boost::repository::BoostRepository;
    use crate::domain::follower::FollowerRepository;
    use crate::test_utils::setup;

    const STATUS_URI: &str = "https://remote.example/users/bob/statuses/42";
    const FOLLOWER_URI: &str = "https://remote.example/users/charlie";

    fn fixture_status() -> Status {
        Status {
            id: 42,
            content: "hi".into(),
            author: "https://remote.example/users/bob".into(),
            created_at: 1_000,
            visibility: Visibility::Public,
            like_count: 0,
            boost_count: 0,
            spoiler_text: None,
            sensitive: false,
        }
    }

    async fn boost_first() {
        FollowerRepository::oneshot()
            .insert(FOLLOWER_URI)
            .expect("insert follower");
        push_fetch_status_response(FetchStatusResponse::Ok(fixture_status()));
        assert_eq!(
            boost_status(BoostStatusArgs {
                status_url: STATUS_URI.into()
            })
            .await,
            BoostStatusResponse::Ok
        );
    }

    #[tokio::test]
    async fn test_should_undo_boost() {
        setup();
        boost_first().await;
        reset_captured();

        let resp = undo_boost(UndoBoostArgs {
            status_url: STATUS_URI.into(),
        })
        .await;
        assert_eq!(resp, UndoBoostResponse::Ok);
        assert!(
            !BoostRepository::oneshot()
                .is_boosted(STATUS_URI)
                .expect("query")
        );

        let captured = captured();
        let SendActivityArgs::Batch(batch) = &captured[0] else {
            panic!("expected batch");
        };
        assert!(!batch.is_empty());
        let activity: activitypub::Activity =
            serde_json::from_str(&batch[0].activity_json).unwrap();
        assert_eq!(activity.base.kind, ActivityType::Undo);
        let ActivityObject::Activity(inner) = activity.object.expect("inner") else {
            panic!("expected nested Activity");
        };
        assert_eq!(inner.base.kind, ActivityType::Announce);
    }

    #[tokio::test]
    async fn test_idempotent_when_no_boost_exists() {
        setup();

        let resp = undo_boost(UndoBoostArgs {
            status_url: STATUS_URI.into(),
        })
        .await;
        assert_eq!(resp, UndoBoostResponse::Ok);
        assert!(captured().is_empty(), "no dispatch when no boost row");
    }

    #[tokio::test]
    async fn test_double_undo_dispatches_only_once() {
        setup();
        boost_first().await;
        reset_captured();

        let _ = undo_boost(UndoBoostArgs {
            status_url: STATUS_URI.into(),
        })
        .await;
        let after_first = captured().len();
        let _ = undo_boost(UndoBoostArgs {
            status_url: STATUS_URI.into(),
        })
        .await;
        let after_second = captured().len();

        assert_eq!(after_first, after_second);
    }
}
