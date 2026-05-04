//! Handle `Follow`, `Accept(Follow)` and `Reject(Follow)` activities.

use activitypub::Activity;
use activitypub::activity::{ActivityObject, ActivityType};
use db_utils::repository::Repository;
use db_utils::transaction::Transaction;
use did::user::ReceiveActivityError;

use crate::domain::follow_request::FollowRequestRepository;
use crate::domain::following::FollowingRepository;
use crate::error::CanisterError;
use crate::schema::{FollowStatus, Schema};

/// Handle an incoming `Follow` activity.
///
/// Extracts the actor URI (the follower) and stores a pending follow request.
/// If a follow request from the same actor already exists, the operation is
/// treated as idempotent and succeeds without inserting a duplicate.
pub(super) fn handle_follow(activity: &Activity) -> Result<(), ReceiveActivityError> {
    let actor_uri = activity
        .actor
        .as_deref()
        .ok_or(ReceiveActivityError::ProcessingFailed)?;

    ic_utils::log!("handle_incoming: Follow from {actor_uri}");

    // Check for existing follow request to ensure idempotency (AP retries)
    let existing = FollowRequestRepository::oneshot()
        .find_by_actor_uri(actor_uri)
        .map_err(|e| {
            ic_utils::log!("handle_incoming: failed to look up follow request: {e}");
            ReceiveActivityError::Internal(e.to_string())
        })?;

    if existing.is_some() {
        ic_utils::log!("handle_incoming: follow request from {actor_uri} already exists, skipping");
        return Ok(());
    }

    FollowRequestRepository::oneshot()
        .insert(actor_uri)
        .map_err(|e| {
            ic_utils::log!("handle_incoming: failed to insert follow request: {e}");
            ReceiveActivityError::Internal(e.to_string())
        })
}

/// Handle an incoming `Accept(Follow)` activity.
///
/// Updates the pending entry in the `following` table to Accepted.
pub(super) fn handle_accept(activity: &Activity) -> Result<(), ReceiveActivityError> {
    let remote_actor_uri = validate_follow_response(activity)?;

    ic_utils::log!("handle_incoming: accepting following for {remote_actor_uri}");

    Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
        FollowingRepository::with_transaction(tx)
            .update_status(remote_actor_uri, FollowStatus::Accepted)
    })
    .map_err(|e| {
        ic_utils::log!("handle_incoming: failed to update following status: {e}");
        match e {
            CanisterError::Database(_) => ReceiveActivityError::ProcessingFailed,
            _ => ReceiveActivityError::Internal(e.to_string()),
        }
    })
}

/// Handle an incoming `Reject(Follow)` activity.
///
/// Deletes the pending entry from the `following` table so the user can
/// re-issue a follow request in the future.
pub(super) fn handle_reject(activity: &Activity) -> Result<(), ReceiveActivityError> {
    let remote_actor_uri = validate_follow_response(activity)?;

    ic_utils::log!("handle_incoming: rejecting following for {remote_actor_uri}, removing entry");

    FollowingRepository::oneshot()
        .delete_by_actor_uri(remote_actor_uri)
        .map_err(|e| {
            ic_utils::log!("handle_incoming: failed to delete following entry: {e}");
            match e {
                CanisterError::Database(_) => ReceiveActivityError::ProcessingFailed,
                _ => ReceiveActivityError::Internal(e.to_string()),
            }
        })
        .map(|_| ())
}

/// Validate that the activity wraps an inner `Follow` activity and extract
/// the remote actor URI (the user who accepted/rejected our follow request).
fn validate_follow_response(activity: &Activity) -> Result<&str, ReceiveActivityError> {
    let remote_actor_uri = activity
        .actor
        .as_deref()
        .ok_or(ReceiveActivityError::ProcessingFailed)?;

    let Some(ActivityObject::Activity(inner)) = &activity.object else {
        ic_utils::log!("handle_incoming: Accept/Reject missing inner Follow activity");
        return Err(ReceiveActivityError::ProcessingFailed);
    };

    if inner.base.kind != ActivityType::Follow {
        ic_utils::log!(
            "handle_incoming: Accept/Reject inner activity is not Follow: {:?}",
            inner.base.kind
        );
        return Err(ReceiveActivityError::ProcessingFailed);
    }

    Ok(remote_actor_uri)
}

#[cfg(test)]
mod tests {

    use activitypub::activity::{Activity, ActivityObject, ActivityType};
    use activitypub::object::BaseObject;
    use db_utils::repository::Repository;
    use did::user::{ReceiveActivityArgs, ReceiveActivityError, ReceiveActivityResponse};

    use super::super::handle_incoming;
    use super::super::test_helpers::{
        make_accept_follow_json, make_follow_json, make_reject_follow_json,
    };
    use crate::domain::follow_request::FollowRequestRepository;
    use crate::domain::following::FollowingRepository;
    use crate::schema::FollowStatus;
    use crate::test_utils::setup;

    #[test]
    fn test_should_store_follow_as_pending_request() {
        setup();

        let json = make_follow_json(
            "https://mastic.social/users/alice",
            "https://mastic.social/users/rey_canisteryo",
        );

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });

        assert_eq!(response, ReceiveActivityResponse::Ok);

        let request = FollowRequestRepository::oneshot()
            .find_by_actor_uri("https://mastic.social/users/alice")
            .expect("should query")
            .expect("should find follow request");
        assert_eq!(request.actor_uri.0, "https://mastic.social/users/alice");
    }

    #[test]
    fn test_should_update_following_to_accepted_on_accept() {
        setup();

        // first, create a pending following entry (simulates follow_user having run)
        FollowingRepository::oneshot()
            .insert_pending("https://mastic.social/users/bob")
            .expect("should insert pending");

        let json = make_accept_follow_json(
            "https://mastic.social/users/bob",
            "https://mastic.social/users/rey_canisteryo",
        );

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });

        assert_eq!(response, ReceiveActivityResponse::Ok);

        let entry = FollowingRepository::oneshot()
            .find_by_actor_uri("https://mastic.social/users/bob")
            .expect("should query")
            .expect("should find following entry");
        assert_eq!(entry.status, FollowStatus::Accepted);
    }

    #[test]
    fn test_should_delete_following_on_reject() {
        setup();

        // first, create a pending following entry
        FollowingRepository::oneshot()
            .insert_pending("https://mastic.social/users/bob")
            .expect("should insert pending");

        let json = make_reject_follow_json(
            "https://mastic.social/users/bob",
            "https://mastic.social/users/rey_canisteryo",
        );

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });

        assert_eq!(response, ReceiveActivityResponse::Ok);

        // entry should be deleted, not updated to rejected
        let entry = FollowingRepository::oneshot()
            .find_by_actor_uri("https://mastic.social/users/bob")
            .expect("should query");
        assert!(
            entry.is_none(),
            "following entry should be deleted on reject"
        );
    }

    #[test]
    fn test_should_fail_accept_when_no_pending_following() {
        setup();

        let json = make_accept_follow_json(
            "https://mastic.social/users/bob",
            "https://mastic.social/users/rey_canisteryo",
        );

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });

        assert_eq!(
            response,
            ReceiveActivityResponse::Err(ReceiveActivityError::ProcessingFailed)
        );
    }

    #[test]
    fn test_should_succeed_reject_when_no_pending_following() {
        setup();

        // Reject for a non-existent entry is idempotent (no-op delete)
        let json = make_reject_follow_json(
            "https://mastic.social/users/bob",
            "https://mastic.social/users/rey_canisteryo",
        );

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });

        assert_eq!(response, ReceiveActivityResponse::Ok);
    }

    #[test]
    fn test_should_handle_duplicate_follow_idempotently() {
        setup();

        let json = make_follow_json(
            "https://mastic.social/users/alice",
            "https://mastic.social/users/rey_canisteryo",
        );

        let first = handle_incoming(ReceiveActivityArgs {
            activity_json: json.clone(),
        });
        assert_eq!(first, ReceiveActivityResponse::Ok);

        // sending the same Follow again should succeed (idempotent)
        let second = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });
        assert_eq!(second, ReceiveActivityResponse::Ok);

        // only one follow request should exist
        let request = FollowRequestRepository::oneshot()
            .find_by_actor_uri("https://mastic.social/users/alice")
            .expect("should query")
            .expect("should find follow request");
        assert_eq!(request.actor_uri.0, "https://mastic.social/users/alice");
    }

    #[test]
    fn test_should_fail_accept_when_missing_inner_activity() {
        setup();

        // Accept with a plain URI object instead of nested Follow activity
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Accept,
                ..Default::default()
            },
            actor: Some("https://mastic.social/users/bob".to_string()),
            object: Some(ActivityObject::Id(
                "https://mastic.social/users/bob".to_string(),
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
}
