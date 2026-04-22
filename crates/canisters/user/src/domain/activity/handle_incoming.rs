//! Handle incoming activity flow

use activitypub::Activity;
use activitypub::activity::{ActivityObject, ActivityType};
use did::user::{ReceiveActivityArgs, ReceiveActivityError, ReceiveActivityResponse};
use wasm_dbms_api::prelude::{Database, Nullable};

use crate::domain::follow_request::FollowRequestRepository;
use crate::domain::following::FollowingRepository;
use crate::domain::snowflake::Snowflake;
use crate::error::CanisterError;
use crate::schema::{
    ActivityType as DbActivityType, FeedEntry, FeedEntryInsertRequest, FeedSource, FollowStatus,
    InboxActivity, InboxActivityInsertRequest, Schema,
};

/// Handles an incoming [`Activity`] from the federation canister.
///
/// Tries to decode the activity object from JSON into an [`Activity`] struct,
/// then it matches on the activity type and performs the appropriate action based on the type of activity received.
pub fn handle_incoming(
    ReceiveActivityArgs { activity_json }: ReceiveActivityArgs,
) -> ReceiveActivityResponse {
    // Try to decode the activity JSON into an Activity struct
    let Ok(activity) = serde_json::from_str::<Activity>(&activity_json) else {
        return ReceiveActivityResponse::Err(ReceiveActivityError::InvalidActivity);
    };

    let result = match activity.base.kind {
        ActivityType::Create => handle_create(&activity, &activity_json),
        ActivityType::Follow => handle_follow(&activity),
        ActivityType::Accept => handle_accept(&activity),
        ActivityType::Reject => handle_reject(&activity),
        other => {
            // Unknown / not-yet-implemented activity types are silently accepted.
            // ActivityPub receivers should not reject deliveries they can't act
            // on — unknown verbs are absorbed so the sender does not retry.
            ic_utils::log!("handle_incoming: ignoring unsupported activity type: {other:?}");
            Ok(())
        }
    };

    match result {
        Ok(()) => ReceiveActivityResponse::Ok,
        Err(e) => ReceiveActivityResponse::Err(e),
    }
}

/// Handle an incoming `Create` activity (e.g. `Create(Note)`).
///
/// Stores the activity in the inbox and records a feed entry so that the
/// status appears in the owner's chronological feed.
fn handle_create(activity: &Activity, activity_json: &str) -> Result<(), ReceiveActivityError> {
    let actor_uri = activity
        .actor
        .as_deref()
        .ok_or(ReceiveActivityError::ProcessingFailed)?;

    ic_utils::log!("handle_incoming: Create from {actor_uri}");

    let created_at = ic_utils::now();
    let snowflake_id = Snowflake::new();
    let object_data: serde_json::Value = serde_json::from_str(activity_json).map_err(|e| {
        ic_utils::log!("handle_incoming: failed to parse activity JSON: {e}");
        ReceiveActivityError::Internal(e.to_string())
    })?;

    // Insert inbox activity and feed entry in a single transaction
    ic_dbms_canister::prelude::DBMS_CONTEXT
        .with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = wasm_dbms::WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);

            db.insert::<InboxActivity>(InboxActivityInsertRequest {
                id: snowflake_id.into(),
                activity_type: DbActivityType::from(ActivityType::Create),
                actor_uri: actor_uri.into(),
                object_data: object_data.into(),
                is_boost: false.into(),
                original_status_uri: Nullable::Null,
                created_at: created_at.into(),
            })?;

            db.insert::<FeedEntry>(FeedEntryInsertRequest {
                id: snowflake_id.into(),
                source: FeedSource::Inbox,
                created_at: created_at.into(),
            })?;

            db.commit()?;
            Ok(())
        })
        .map_err(|e: CanisterError| {
            ic_utils::log!("handle_incoming: failed to insert inbox activity + feed entry: {e}");
            ReceiveActivityError::Internal(e.to_string())
        })?;

    Ok(())
}

/// Handle an incoming `Follow` activity.
///
/// Extracts the actor URI (the follower) and stores a pending follow request.
/// If a follow request from the same actor already exists, the operation is
/// treated as idempotent and succeeds without inserting a duplicate.
fn handle_follow(activity: &Activity) -> Result<(), ReceiveActivityError> {
    let actor_uri = activity
        .actor
        .as_deref()
        .ok_or(ReceiveActivityError::ProcessingFailed)?;

    ic_utils::log!("handle_incoming: Follow from {actor_uri}");

    // Check for existing follow request to ensure idempotency (AP retries)
    let existing = FollowRequestRepository::find_by_actor_uri(actor_uri).map_err(|e| {
        ic_utils::log!("handle_incoming: failed to look up follow request: {e}");
        ReceiveActivityError::Internal(e.to_string())
    })?;

    if existing.is_some() {
        ic_utils::log!("handle_incoming: follow request from {actor_uri} already exists, skipping");
        return Ok(());
    }

    FollowRequestRepository::insert(actor_uri).map_err(|e| {
        ic_utils::log!("handle_incoming: failed to insert follow request: {e}");
        ReceiveActivityError::Internal(e.to_string())
    })
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

/// Handle an incoming `Accept(Follow)` activity.
///
/// Updates the pending entry in the `following` table to Accepted.
fn handle_accept(activity: &Activity) -> Result<(), ReceiveActivityError> {
    let remote_actor_uri = validate_follow_response(activity)?;

    ic_utils::log!("handle_incoming: accepting following for {remote_actor_uri}");

    FollowingRepository::update_status(remote_actor_uri, FollowStatus::Accepted).map_err(|e| {
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
fn handle_reject(activity: &Activity) -> Result<(), ReceiveActivityError> {
    let remote_actor_uri = validate_follow_response(activity)?;

    ic_utils::log!("handle_incoming: rejecting following for {remote_actor_uri}, removing entry");

    FollowingRepository::delete_by_actor_uri(remote_actor_uri).map_err(|e| {
        ic_utils::log!("handle_incoming: failed to delete following entry: {e}");
        match e {
            CanisterError::Database(_) => ReceiveActivityError::ProcessingFailed,
            _ => ReceiveActivityError::Internal(e.to_string()),
        }
    })
}

#[cfg(test)]
mod tests {

    use activitypub::activity::{Activity, ActivityObject, ActivityType};
    use activitypub::object::BaseObject;
    use did::user::{ReceiveActivityArgs, ReceiveActivityError, ReceiveActivityResponse};

    use super::handle_incoming;
    use crate::domain::follow_request::FollowRequestRepository;
    use crate::domain::following::FollowingRepository;
    use crate::schema::FollowStatus;
    use crate::test_utils::setup;

    fn make_follow_json(follower_actor_uri: &str, target_actor_uri: &str) -> String {
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Follow,
                ..Default::default()
            },
            actor: Some(follower_actor_uri.to_string()),
            object: Some(ActivityObject::Id(target_actor_uri.to_string())),
            target: None,
            result: None,
            origin: None,
            instrument: None,
        };
        serde_json::to_string(&activity).unwrap()
    }

    fn make_accept_follow_json(acceptor_actor_uri: &str, follower_actor_uri: &str) -> String {
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Accept,
                ..Default::default()
            },
            actor: Some(acceptor_actor_uri.to_string()),
            object: Some(ActivityObject::Activity(Box::new(Activity {
                base: BaseObject {
                    kind: ActivityType::Follow,
                    ..Default::default()
                },
                actor: Some(follower_actor_uri.to_string()),
                object: Some(ActivityObject::Id(acceptor_actor_uri.to_string())),
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
        serde_json::to_string(&activity).unwrap()
    }

    fn make_reject_follow_json(rejector_actor_uri: &str, follower_actor_uri: &str) -> String {
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Reject,
                ..Default::default()
            },
            actor: Some(rejector_actor_uri.to_string()),
            object: Some(ActivityObject::Activity(Box::new(Activity {
                base: BaseObject {
                    kind: ActivityType::Follow,
                    ..Default::default()
                },
                actor: Some(follower_actor_uri.to_string()),
                object: Some(ActivityObject::Id(rejector_actor_uri.to_string())),
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
        serde_json::to_string(&activity).unwrap()
    }

    #[test]
    fn test_should_return_invalid_activity_for_bad_json() {
        setup();

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: "not valid json".to_string(),
        });

        assert_eq!(
            response,
            ReceiveActivityResponse::Err(ReceiveActivityError::InvalidActivity)
        );
    }

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

        let request =
            FollowRequestRepository::find_by_actor_uri("https://mastic.social/users/alice")
                .expect("should query")
                .expect("should find follow request");
        assert_eq!(request.actor_uri.0, "https://mastic.social/users/alice");
    }

    #[test]
    fn test_should_update_following_to_accepted_on_accept() {
        setup();

        // first, create a pending following entry (simulates follow_user having run)
        FollowingRepository::insert_pending("https://mastic.social/users/bob")
            .expect("should insert pending");

        let json = make_accept_follow_json(
            "https://mastic.social/users/bob",
            "https://mastic.social/users/rey_canisteryo",
        );

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });

        assert_eq!(response, ReceiveActivityResponse::Ok);

        let entry = FollowingRepository::find_by_actor_uri("https://mastic.social/users/bob")
            .expect("should query")
            .expect("should find following entry");
        assert_eq!(entry.status, FollowStatus::Accepted);
    }

    #[test]
    fn test_should_delete_following_on_reject() {
        setup();

        // first, create a pending following entry
        FollowingRepository::insert_pending("https://mastic.social/users/bob")
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
        let entry = FollowingRepository::find_by_actor_uri("https://mastic.social/users/bob")
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
        let request =
            FollowRequestRepository::find_by_actor_uri("https://mastic.social/users/alice")
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
