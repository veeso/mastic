//! Domain logic for the `unfollow_user` flow.
//!
//! The flow consists of:
//! 1. Look up the target in the `following` table.
//! 2. If absent, return success (idempotent).
//! 3. Delete the row regardless of its status (`Pending` cancels an outbound
//!    follow request, `Accepted` ends an established follow).
//! 4. Build an `Undo(Follow)` activity and dispatch it to the target via the
//!    Federation Canister.

use activitypub::activity::{Activity, ActivityObject, ActivityType};
use activitypub::context::ACTIVITY_STREAMS_CONTEXT;
use activitypub::object::{BaseObject, OneOrMany};
use did::federation::{SendActivityArgs, SendActivityArgsObject};
use did::user::{UnfollowUserArgs, UnfollowUserError, UnfollowUserResponse};

use crate::domain::following::FollowingRepository;
use crate::domain::profile::ProfileRepository;
use crate::error::CanisterResult;

/// Execute the unfollow-user flow.
pub async fn unfollow_user(
    UnfollowUserArgs { actor_uri }: UnfollowUserArgs,
) -> UnfollowUserResponse {
    ic_utils::log!("unfollow_user: attempting to unfollow {actor_uri}");

    match unfollow_user_inner(&actor_uri).await {
        Ok(()) => UnfollowUserResponse::Ok,
        Err(err) => {
            let err = err.to_string();
            ic_utils::log!("unfollow_user: error: {err}");
            UnfollowUserResponse::Err(UnfollowUserError::Internal(err))
        }
    }
}

async fn unfollow_user_inner(target_actor_uri: &str) -> CanisterResult<()> {
    // Idempotent: delete returns false when no row matched — nothing to do.
    // Removes the row regardless of status (Pending or Accepted).
    if !FollowingRepository::delete_by_actor_uri(target_actor_uri)? {
        ic_utils::log!("unfollow_user: not following {target_actor_uri}, no-op");
        return Ok(());
    }
    ic_utils::log!("unfollow_user: removed following entry for {target_actor_uri}");

    // Build own actor URI for the Undo(Follow) payload.
    let own_profile = ProfileRepository::get_profile()?;
    let own_actor_uri = crate::domain::urls::actor_uri(&own_profile.handle.0)?;

    let activity = make_undo_follow_activity(&own_actor_uri, target_actor_uri);
    let target_inbox = crate::domain::urls::inbox_url_from_actor_uri(target_actor_uri);

    let args = SendActivityArgs::One(SendActivityArgsObject {
        activity_json: serde_json::to_string(&activity)
            .expect("Activity serialization must not fail"),
        target_inbox,
    });

    crate::adapters::federation::send_activity(args).await?;

    Ok(())
}

/// Build an `Undo(Follow)` [`Activity`] cancelling a previous follow.
fn make_undo_follow_activity(own_actor_uri: &str, target_actor_uri: &str) -> Activity {
    let inner_follow = Activity {
        base: BaseObject {
            kind: ActivityType::Follow,
            ..Default::default()
        },
        actor: Some(own_actor_uri.to_string()),
        object: Some(ActivityObject::Id(target_actor_uri.to_string())),
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
            to: Some(OneOrMany::One(target_actor_uri.to_string())),
            ..Default::default()
        },
        actor: Some(own_actor_uri.to_string()),
        object: Some(ActivityObject::Activity(Box::new(inner_follow))),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    }
}

#[cfg(test)]
mod tests {

    use did::user::{UnfollowUserArgs, UnfollowUserResponse};

    use super::*;
    use crate::schema::FollowStatus;
    use crate::test_utils::setup;

    const TARGET_URI: &str = "https://mastic.social/users/alice";
    const OWN_URI: &str = "https://mastic.social/users/rey_canisteryo";

    #[tokio::test]
    async fn test_should_unfollow_accepted_target() {
        setup();

        FollowingRepository::insert_pending(TARGET_URI).expect("should insert");
        FollowingRepository::update_status(TARGET_URI, FollowStatus::Accepted)
            .expect("should accept");

        let response = unfollow_user(UnfollowUserArgs {
            actor_uri: TARGET_URI.to_string(),
        })
        .await;

        assert_eq!(response, UnfollowUserResponse::Ok);
        assert!(
            FollowingRepository::find_by_actor_uri(TARGET_URI)
                .expect("should query")
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_should_unfollow_pending_target() {
        setup();

        FollowingRepository::insert_pending(TARGET_URI).expect("should insert");

        let response = unfollow_user(UnfollowUserArgs {
            actor_uri: TARGET_URI.to_string(),
        })
        .await;

        assert_eq!(response, UnfollowUserResponse::Ok);
        assert!(
            FollowingRepository::find_by_actor_uri(TARGET_URI)
                .expect("should query")
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_should_be_idempotent_when_not_following() {
        setup();

        let response = unfollow_user(UnfollowUserArgs {
            actor_uri: TARGET_URI.to_string(),
        })
        .await;

        assert_eq!(response, UnfollowUserResponse::Ok);
    }

    #[test]
    fn test_make_undo_follow_activity_should_wrap_inner_follow() {
        let activity = make_undo_follow_activity(OWN_URI, TARGET_URI);

        assert_eq!(activity.base.kind, ActivityType::Undo);
        assert_eq!(activity.actor.as_deref(), Some(OWN_URI));
        assert_eq!(
            activity.base.to,
            Some(OneOrMany::One(TARGET_URI.to_string()))
        );

        let ActivityObject::Activity(inner) = activity.object.expect("should have object") else {
            panic!("expected Activity variant");
        };
        assert_eq!(inner.base.kind, ActivityType::Follow);
        assert_eq!(inner.actor.as_deref(), Some(OWN_URI));
        let ActivityObject::Id(inner_obj) = inner.object.expect("should have inner object") else {
            panic!("expected Id variant");
        };
        assert_eq!(inner_obj, TARGET_URI);
    }
}
