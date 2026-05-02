//! Domain logic for the `unlike_status` flow.
//!
//! The flow consists of:
//! 1. Look up the status URI in the `liked` table.
//! 2. If absent, return success (idempotent — nothing to undo).
//! 3. Delete the row.
//! 4. Build an `Undo(Like)` activity wrapping the original `Like` and
//!    dispatch it via the Federation Canister to the status author's
//!    inbox.
//!
//! Like the `like_status` flow, the status URI is opaque and the
//! author's inbox is derived by stripping `/statuses/{id}` and appending
//! `/inbox`.

use activitypub::activity::{Activity, ActivityObject, ActivityType};
use activitypub::context::ACTIVITY_STREAMS_CONTEXT;
use activitypub::object::{BaseObject, OneOrMany};
use did::federation::{SendActivityArgs, SendActivityArgsObject};
use did::user::{UnlikeStatusArgs, UnlikeStatusError, UnlikeStatusResponse};

use crate::domain::liked::repository::LikedRepository;
use crate::domain::profile::ProfileRepository;
use crate::error::CanisterResult;

/// Execute the unlike-status flow.
pub async fn unlike_status(
    UnlikeStatusArgs { status_url }: UnlikeStatusArgs,
) -> UnlikeStatusResponse {
    match unlike_status_inner(status_url).await {
        Ok(()) => UnlikeStatusResponse::Ok,
        Err(err) => {
            ic_utils::log!("Failed to unlike status: {err}");
            UnlikeStatusResponse::Err(UnlikeStatusError::Internal(err.to_string()))
        }
    }
}

async fn unlike_status_inner(status_uri: String) -> CanisterResult<()> {
    ic_utils::log!("Unliking status with URI: {status_uri}");

    // Idempotent: nothing to do if the row is absent.
    if !LikedRepository::oneshot().is_liked(&status_uri)? {
        ic_utils::log!("Status not liked: {status_uri}; nothing to do");
        return Ok(());
    }

    LikedRepository::oneshot().unlike_status(&status_uri)?;

    let Some(author_actor_uri) = crate::domain::urls::actor_uri_from_status_uri(&status_uri) else {
        ic_utils::log!("unlike_status: could not derive author actor URI from {status_uri}");
        return Ok(());
    };
    let target_inbox = crate::domain::urls::inbox_url_from_actor_uri(&author_actor_uri);

    let own_profile = ProfileRepository::get_profile()?;
    let own_actor_uri = crate::domain::urls::actor_uri(&own_profile.handle.0)?;
    let activity = make_undo_like_activity(&own_actor_uri, &author_actor_uri, &status_uri);

    let args = SendActivityArgs::One(SendActivityArgsObject {
        activity_json: serde_json::to_string(&activity)
            .expect("Activity serialization must not fail"),
        target_inbox,
    });

    crate::adapters::federation::send_activity(args).await?;

    Ok(())
}

/// Build an `Undo(Like)` [`Activity`] cancelling a previous like.
fn make_undo_like_activity(
    own_actor_uri: &str,
    author_actor_uri: &str,
    status_uri: &str,
) -> Activity {
    let inner_like = Activity {
        base: BaseObject {
            kind: ActivityType::Like,
            ..Default::default()
        },
        actor: Some(own_actor_uri.to_string()),
        object: Some(ActivityObject::Id(status_uri.to_string())),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    };

    Activity {
        base: BaseObject {
            context: Some(activitypub::context::Context::Uri(
                ACTIVITY_STREAMS_CONTEXT.to_string(),
            )),
            kind: ActivityType::Undo,
            to: Some(OneOrMany::One(author_actor_uri.to_string())),
            ..Default::default()
        },
        actor: Some(own_actor_uri.to_string()),
        object: Some(ActivityObject::Activity(Box::new(inner_like))),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    }
}

#[cfg(test)]
mod tests {

    use did::federation::{SendActivityArgs, SendActivityArgsObject};

    use super::*;
    use crate::adapters::federation::mock::captured;
    use crate::test_utils::setup;

    const STATUS_URI: &str = "https://mastic.social/users/alice/statuses/42";
    const AUTHOR_URI: &str = "https://mastic.social/users/alice";
    const OWN_URI: &str = "https://mastic.social/users/rey_canisteryo";

    #[tokio::test]
    async fn test_should_unlike_status_after_like() {
        setup();

        LikedRepository::oneshot()
            .like_status(STATUS_URI)
            .expect("should insert liked");

        let response = unlike_status(UnlikeStatusArgs {
            status_url: STATUS_URI.to_string(),
        })
        .await;

        assert_eq!(response, UnlikeStatusResponse::Ok);
        assert!(
            !LikedRepository::oneshot()
                .is_liked(STATUS_URI)
                .expect("should query")
        );

        let captured = captured();
        assert_eq!(captured.len(), 1);
        let SendActivityArgs::One(SendActivityArgsObject {
            activity_json,
            target_inbox,
        }) = &captured[0]
        else {
            panic!("expected SendActivityArgs::One");
        };
        assert_eq!(target_inbox, &format!("{AUTHOR_URI}/inbox"));

        let activity: Activity = serde_json::from_str(activity_json).expect("valid activity");
        assert_eq!(activity.base.kind, ActivityType::Undo);
        let ActivityObject::Activity(inner) = activity.object.expect("inner") else {
            panic!("expected wrapped activity");
        };
        assert_eq!(inner.base.kind, ActivityType::Like);
        let ActivityObject::Id(obj) = inner.object.expect("inner object") else {
            panic!("expected Id");
        };
        assert_eq!(obj, STATUS_URI);
    }

    #[tokio::test]
    async fn test_should_be_idempotent_when_not_liked() {
        setup();

        let response = unlike_status(UnlikeStatusArgs {
            status_url: STATUS_URI.to_string(),
        })
        .await;

        assert_eq!(response, UnlikeStatusResponse::Ok);
        assert!(captured().is_empty(), "no activity dispatched");
    }

    #[test]
    fn test_make_undo_like_activity_should_wrap_inner_like() {
        let activity = make_undo_like_activity(OWN_URI, AUTHOR_URI, STATUS_URI);

        assert_eq!(activity.base.kind, ActivityType::Undo);
        assert_eq!(
            activity.base.to,
            Some(OneOrMany::One(AUTHOR_URI.to_string()))
        );

        let ActivityObject::Activity(inner) = activity.object.expect("inner") else {
            panic!("expected Activity variant");
        };
        assert_eq!(inner.base.kind, ActivityType::Like);
        assert_eq!(inner.actor.as_deref(), Some(OWN_URI));
        let ActivityObject::Id(obj) = inner.object.expect("inner object") else {
            panic!("expected Id");
        };
        assert_eq!(obj, STATUS_URI);
    }
}
