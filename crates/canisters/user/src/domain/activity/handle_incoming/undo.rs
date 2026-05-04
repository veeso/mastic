//! Handle `Undo` activity dispatch.

use activitypub::Activity;
use activitypub::activity::{ActivityObject, ActivityType};
use db_utils::repository::Repository;
use did::user::ReceiveActivityError;

use crate::repository::follow_request::FollowRequestRepository;
use crate::repository::follower::FollowerRepository;

/// Handle an incoming `Undo(Follow)` or `Undo(Like)` activity.
///
/// - `Undo(Follow)`: removes the sender from the `followers` table
///   (accepted inbound follow) and from the `follow_requests` table
///   (pending inbound follow). Idempotent: missing entries do not produce
///   an error.
/// - `Undo(Like)`: decrements the cached `like_count` of the targeted local
///   status when the URI points at one of our statuses; ignored otherwise.
/// - Any other inner activity is silently accepted but not acted on.
pub(super) fn handle_undo(activity: &Activity) -> Result<(), ReceiveActivityError> {
    let sender_uri = activity
        .actor
        .as_deref()
        .ok_or(ReceiveActivityError::ProcessingFailed)?;

    let Some(ActivityObject::Activity(inner)) = &activity.object else {
        ic_utils::log!("handle_incoming: Undo missing inner activity");
        return Err(ReceiveActivityError::ProcessingFailed);
    };

    match inner.base.kind {
        ActivityType::Follow => {
            ic_utils::log!("handle_incoming: Undo(Follow) from {sender_uri}");

            FollowerRepository::oneshot()
                .delete_by_actor_uri(sender_uri)
                .map_err(|e| {
                    ic_utils::log!("handle_incoming: failed to delete follower: {e}");
                    ReceiveActivityError::Internal(e.to_string())
                })?;
            FollowRequestRepository::oneshot()
                .delete_by_actor_uri(sender_uri)
                .map_err(|e| {
                    ic_utils::log!("handle_incoming: failed to delete follow request: {e}");
                    ReceiveActivityError::Internal(e.to_string())
                })?;
            Ok(())
        }
        ActivityType::Like => super::like::handle_undo_like(inner, sender_uri),
        ActivityType::Announce => super::announce::handle_undo_announce(inner, sender_uri),
        other => {
            ic_utils::log!("handle_incoming: ignoring Undo of unsupported inner type: {other:?}");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {

    use activitypub::activity::{Activity, ActivityObject, ActivityType};
    use activitypub::object::BaseObject;
    use db_utils::repository::Repository;
    use did::user::{ReceiveActivityArgs, ReceiveActivityError, ReceiveActivityResponse};

    use super::super::handle_incoming;
    use super::super::test_helpers::make_undo_follow_json;
    use crate::repository::follow_request::FollowRequestRepository;
    use crate::repository::follower::FollowerRepository;
    use crate::test_utils::setup;

    #[test]
    fn test_should_remove_follower_on_undo_follow() {
        setup();

        FollowerRepository::oneshot()
            .insert("https://mastic.social/users/alice")
            .expect("should insert");

        let json = make_undo_follow_json(
            "https://mastic.social/users/alice",
            "https://mastic.social/users/rey_canisteryo",
        );

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });
        assert_eq!(response, ReceiveActivityResponse::Ok);

        let followers = FollowerRepository::oneshot()
            .get_followers()
            .expect("should query");
        assert!(followers.is_empty(), "follower entry should be deleted");
    }

    #[test]
    fn test_should_remove_pending_follow_request_on_undo_follow() {
        setup();

        FollowRequestRepository::oneshot()
            .insert("https://mastic.social/users/alice")
            .expect("should insert");

        let json = make_undo_follow_json(
            "https://mastic.social/users/alice",
            "https://mastic.social/users/rey_canisteryo",
        );

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });
        assert_eq!(response, ReceiveActivityResponse::Ok);

        let request = FollowRequestRepository::oneshot()
            .find_by_actor_uri("https://mastic.social/users/alice")
            .expect("should query");
        assert!(request.is_none(), "follow request should be deleted");
    }

    #[test]
    fn test_should_succeed_undo_follow_when_no_entry_exists() {
        setup();

        let json = make_undo_follow_json(
            "https://mastic.social/users/alice",
            "https://mastic.social/users/rey_canisteryo",
        );

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });
        assert_eq!(response, ReceiveActivityResponse::Ok);
    }

    #[test]
    fn test_should_fail_undo_when_missing_inner_activity() {
        setup();

        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Undo,
                ..Default::default()
            },
            actor: Some("https://mastic.social/users/alice".to_string()),
            object: Some(ActivityObject::Id(
                "https://mastic.social/users/rey_canisteryo".to_string(),
            )),
            target: None,
            result: None,
            origin: None,
            instrument: None,
        };
        let json = serde_json::to_string(&activity).unwrap();

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });

        assert_eq!(
            response,
            ReceiveActivityResponse::Err(ReceiveActivityError::ProcessingFailed)
        );
    }

    #[test]
    fn test_should_ignore_undo_of_unsupported_inner_type() {
        setup();

        // Undo(Block) is not implemented; expected to be silently accepted.
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Undo,
                ..Default::default()
            },
            actor: Some("https://mastic.social/users/alice".to_string()),
            object: Some(ActivityObject::Activity(Box::new(Activity {
                base: BaseObject {
                    kind: ActivityType::Block,
                    ..Default::default()
                },
                actor: Some("https://mastic.social/users/alice".to_string()),
                object: Some(ActivityObject::Id(
                    "https://mastic.social/users/rey_canisteryo".to_string(),
                )),
                target: None,
                result: None,
                origin: None,
                instrument: None,
            }))),
            target: None,
            result: None,
            origin: None,
            instrument: None,
        };
        let json = serde_json::to_string(&activity).unwrap();

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });

        assert_eq!(response, ReceiveActivityResponse::Ok);
    }
}
