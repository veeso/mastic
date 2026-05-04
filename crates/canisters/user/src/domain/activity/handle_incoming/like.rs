//! Handle `Like` and `Undo(Like)` activities.

use activitypub::Activity;
use db_utils::repository::Repository;
use did::user::ReceiveActivityError;

use crate::repository::status::StatusRepository;

/// Handle an incoming `Like` activity.
///
/// Increments the cached `like_count` on the targeted local status. The
/// status URI is parsed against this canister's `public_url`; URIs that
/// point at a different host or a different local handle are ignored.
/// We cannot authoritatively prove that a remote sender's `Like` actually
/// references a status we own beyond URI matching, so the count is
/// best-effort.
///
/// Idempotency note: ActivityPub does not require unique delivery, so the
/// same `Like` may be received multiple times. We have no `(actor, status)`
/// table to deduplicate against — the cached count is a hint, not a
/// reconciled total.
pub(super) fn handle_like(activity: &Activity) -> Result<(), ReceiveActivityError> {
    let actor_uri = activity
        .actor
        .as_deref()
        .ok_or(ReceiveActivityError::ProcessingFailed)?;
    let Some(status_uri) = super::extract_object_uri(activity) else {
        ic_utils::log!("handle_incoming: Like missing object URI");
        return Err(ReceiveActivityError::ProcessingFailed);
    };
    ic_utils::log!("handle_incoming: Like from {actor_uri} on {status_uri}");

    let Some((_handle, id)) = super::parse_local_status(&status_uri)? else {
        ic_utils::log!("handle_incoming: Like target {status_uri} is not a local status");
        return Ok(());
    };

    if !StatusRepository::oneshot()
        .increment_like_count(id)
        .map_err(|e| {
            ic_utils::log!("handle_incoming: failed to increment like_count: {e}");
            ReceiveActivityError::Internal(e.to_string())
        })?
    {
        ic_utils::log!("handle_incoming: Like target status {id} not found, ignoring");
    }
    Ok(())
}

/// Handle an `Undo(Like)` body: decrement the cached `like_count` if the
/// inner activity refers to a local status. Ignored otherwise.
pub(super) fn handle_undo_like(
    inner: &Activity,
    sender_uri: &str,
) -> Result<(), ReceiveActivityError> {
    let Some(status_uri) = super::extract_object_uri(inner) else {
        ic_utils::log!("handle_incoming: Undo(Like) missing object URI");
        return Err(ReceiveActivityError::ProcessingFailed);
    };
    ic_utils::log!("handle_incoming: Undo(Like) from {sender_uri} on {status_uri}");

    let Some((_handle, id)) = super::parse_local_status(&status_uri)? else {
        ic_utils::log!("handle_incoming: Undo(Like) target {status_uri} is not local");
        return Ok(());
    };

    if !StatusRepository::oneshot()
        .decrement_like_count(id)
        .map_err(|e| {
            ic_utils::log!("handle_incoming: failed to decrement like_count: {e}");
            ReceiveActivityError::Internal(e.to_string())
        })?
    {
        ic_utils::log!("handle_incoming: Undo(Like) target status {id} not found, ignoring");
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use activitypub::activity::{Activity, ActivityType};
    use activitypub::object::BaseObject;
    use db_utils::repository::Repository;
    use did::user::{ReceiveActivityArgs, ReceiveActivityError, ReceiveActivityResponse};

    use super::super::handle_incoming;
    use super::super::test_helpers::{make_like_json, make_undo_like_json};
    use crate::repository::status::StatusRepository;
    use crate::test_utils::setup;

    #[test]
    fn test_should_increment_like_count_on_local_like() {
        setup();
        crate::test_utils::insert_status(42, "Hello", did::common::Visibility::Public, 1_000_000);

        let json = make_like_json(
            "https://mastic.social/users/alice",
            "https://mastic.social/users/rey_canisteryo/statuses/42",
        );
        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });
        assert_eq!(response, ReceiveActivityResponse::Ok);

        let status = StatusRepository::oneshot()
            .find_by_id(42)
            .expect("should query")
            .expect("should find");
        assert_eq!(status.like_count.0, 1);
    }

    #[test]
    fn test_should_decrement_like_count_on_undo_like() {
        setup();
        crate::test_utils::insert_status(42, "Hello", did::common::Visibility::Public, 1_000_000);

        // Increment first via Like, then decrement via Undo(Like).
        let like_json = make_like_json(
            "https://mastic.social/users/alice",
            "https://mastic.social/users/rey_canisteryo/statuses/42",
        );
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: like_json,
            }),
            ReceiveActivityResponse::Ok
        );

        let undo_json = make_undo_like_json(
            "https://mastic.social/users/alice",
            "https://mastic.social/users/rey_canisteryo/statuses/42",
        );
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: undo_json,
            }),
            ReceiveActivityResponse::Ok
        );

        let status = StatusRepository::oneshot()
            .find_by_id(42)
            .expect("should query")
            .expect("should find");
        assert_eq!(status.like_count.0, 0);
    }

    #[test]
    fn test_should_ignore_like_targeting_remote_status() {
        setup();

        let json = make_like_json(
            "https://mastic.social/users/alice",
            "https://other.example/users/bob/statuses/99",
        );
        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });
        // Acknowledged but no local effect since URI is not on this instance.
        assert_eq!(response, ReceiveActivityResponse::Ok);
    }

    #[test]
    fn test_should_ignore_like_targeting_other_local_handle() {
        setup();
        crate::test_utils::insert_status(42, "Hello", did::common::Visibility::Public, 1_000_000);

        let json = make_like_json(
            "https://mastic.social/users/alice",
            "https://mastic.social/users/someone_else/statuses/42",
        );
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: json,
            }),
            ReceiveActivityResponse::Ok
        );

        let status = StatusRepository::oneshot()
            .find_by_id(42)
            .expect("should query")
            .expect("should find");
        assert_eq!(
            status.like_count.0, 0,
            "like_count must not change when handle does not match"
        );
    }

    #[test]
    fn test_should_succeed_like_when_target_status_missing() {
        setup();

        let json = make_like_json(
            "https://mastic.social/users/alice",
            "https://mastic.social/users/rey_canisteryo/statuses/9999",
        );
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: json,
            }),
            ReceiveActivityResponse::Ok
        );
    }

    #[test]
    fn test_should_saturate_undo_like_at_zero() {
        setup();
        crate::test_utils::insert_status(42, "Hello", did::common::Visibility::Public, 1_000_000);

        let json = make_undo_like_json(
            "https://mastic.social/users/alice",
            "https://mastic.social/users/rey_canisteryo/statuses/42",
        );
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: json,
            }),
            ReceiveActivityResponse::Ok
        );

        let status = StatusRepository::oneshot()
            .find_by_id(42)
            .expect("should query")
            .expect("should find");
        assert_eq!(status.like_count.0, 0);
    }

    #[test]
    fn test_should_fail_like_with_missing_object() {
        setup();

        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Like,
                ..Default::default()
            },
            actor: Some("https://mastic.social/users/alice".to_string()),
            object: None,
            target: None,
            result: None,
            origin: None,
            instrument: None,
        };
        let json = serde_json::to_string(&activity).unwrap();

        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: json,
            }),
            ReceiveActivityResponse::Err(ReceiveActivityError::ProcessingFailed)
        );
    }
}
