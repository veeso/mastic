//! Domain logic for the `like_status` flow.
//!
//! The flow consists of:
//! 1. Check the `liked` table to ensure the like is idempotent: if the
//!    caller already liked the status the call returns `Ok` without
//!    inserting a duplicate row or re-emitting an activity.
//! 2. Insert a new entry in the `liked` table.
//! 3. Build a `Like` activity targeting the status URI and dispatch it
//!    via the Federation Canister to the status author's inbox.
//!
//! The status URI is treated as opaque: the author's inbox is derived
//! from the URI by stripping the trailing `/statuses/{id}` segment and
//! appending `/inbox`. The author's User Canister is responsible for
//! validating that the URI points at a status it owns and incrementing
//! the cached `like_count` (see
//! [`crate::domain::activity::handle_incoming`]).

use activitypub::activity::{Activity, ActivityObject, ActivityType};
use activitypub::context::ACTIVITY_STREAMS_CONTEXT;
use activitypub::object::{BaseObject, OneOrMany};
use did::federation::{SendActivityArgs, SendActivityArgsObject};
use did::user::{LikeStatusArgs, LikeStatusError, LikeStatusResponse};

use crate::domain::liked::repository::LikedRepository;
use crate::domain::profile::ProfileRepository;
use crate::error::CanisterResult;

/// Execute the like-status flow.
pub async fn like_status(LikeStatusArgs { status_url }: LikeStatusArgs) -> LikeStatusResponse {
    match like_status_inner(status_url).await {
        Ok(()) => LikeStatusResponse::Ok,
        Err(err) => {
            ic_utils::log!("Failed to like status: {err}");
            LikeStatusResponse::Err(LikeStatusError::Internal(err.to_string()))
        }
    }
}

async fn like_status_inner(status_uri: String) -> CanisterResult<()> {
    ic_utils::log!("Liking status with URI: {status_uri}");

    // Idempotent: if already liked, do not duplicate or re-emit.
    if LikedRepository::oneshot().is_liked(&status_uri)? {
        ic_utils::log!("Status already liked: {status_uri}");
        return Ok(());
    }

    // Insert the like into the database first; if federation dispatch
    // later fails, the user can re-trigger and the row already exists,
    // making the second call a no-op.
    LikedRepository::oneshot().like_status(&status_uri)?;

    // Derive the author's actor URI and inbox from the status URI.
    let Some(author_actor_uri) = crate::domain::urls::actor_uri_from_status_uri(&status_uri) else {
        ic_utils::log!("like_status: could not derive author actor URI from {status_uri}");
        return Ok(());
    };
    let target_inbox = crate::domain::urls::inbox_url_from_actor_uri(&author_actor_uri);

    // Build the Like activity actored by the caller.
    let own_profile = ProfileRepository::get_profile()?;
    let own_actor_uri = crate::domain::urls::actor_uri(&own_profile.handle.0)?;
    let activity = make_like_activity(&own_actor_uri, &author_actor_uri, &status_uri);

    let args = SendActivityArgs::One(SendActivityArgsObject {
        activity_json: serde_json::to_string(&activity)
            .expect("Activity serialization must not fail"),
        target_inbox,
    });

    crate::adapters::federation::send_activity(args).await?;

    Ok(())
}

/// Build a `Like` [`Activity`] addressed to the status author and pointing
/// at the status URI.
fn make_like_activity(own_actor_uri: &str, author_actor_uri: &str, status_uri: &str) -> Activity {
    Activity {
        base: BaseObject {
            context: Some(activitypub::context::Context::Uri(
                ACTIVITY_STREAMS_CONTEXT.to_string(),
            )),
            kind: ActivityType::Like,
            to: Some(OneOrMany::One(author_actor_uri.to_string())),
            ..Default::default()
        },
        actor: Some(own_actor_uri.to_string()),
        object: Some(ActivityObject::Id(status_uri.to_string())),
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
    async fn test_should_like_status() {
        setup();

        let response = like_status(LikeStatusArgs {
            status_url: STATUS_URI.to_string(),
        })
        .await;

        assert_eq!(response, LikeStatusResponse::Ok);
        assert!(
            LikedRepository::oneshot()
                .is_liked(STATUS_URI)
                .expect("should query")
        );

        let captured = captured();
        assert_eq!(captured.len(), 1, "exactly one activity dispatched");
        let SendActivityArgs::One(SendActivityArgsObject {
            activity_json,
            target_inbox,
        }) = &captured[0]
        else {
            panic!("expected SendActivityArgs::One");
        };
        assert_eq!(target_inbox, &format!("{AUTHOR_URI}/inbox"));

        let activity: Activity = serde_json::from_str(activity_json).expect("valid activity");
        assert_eq!(activity.base.kind, ActivityType::Like);
        assert_eq!(activity.actor.as_deref(), Some(OWN_URI));
        let ActivityObject::Id(obj) = activity.object.expect("object") else {
            panic!("expected Id variant");
        };
        assert_eq!(obj, STATUS_URI);
    }

    #[tokio::test]
    async fn test_should_be_idempotent_when_already_liked() {
        setup();

        let first = like_status(LikeStatusArgs {
            status_url: STATUS_URI.to_string(),
        })
        .await;
        assert_eq!(first, LikeStatusResponse::Ok);

        let second = like_status(LikeStatusArgs {
            status_url: STATUS_URI.to_string(),
        })
        .await;
        assert_eq!(second, LikeStatusResponse::Ok);

        // Only one activity dispatched across both calls.
        assert_eq!(captured().len(), 1);
    }

    #[test]
    fn test_make_like_activity_should_address_author() {
        let activity = make_like_activity(OWN_URI, AUTHOR_URI, STATUS_URI);

        assert_eq!(activity.base.kind, ActivityType::Like);
        assert_eq!(
            activity.base.to,
            Some(OneOrMany::One(AUTHOR_URI.to_string()))
        );
        assert_eq!(activity.actor.as_deref(), Some(OWN_URI));
    }
}
