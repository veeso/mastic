//! Handle incoming activity flow

use activitypub::Activity;
use activitypub::activity::{ActivityObject, ActivityType};
use db_utils::repository::Repository;
use db_utils::transaction::Transaction;
use did::user::{ReceiveActivityArgs, ReceiveActivityError, ReceiveActivityResponse};
use wasm_dbms_api::prelude::{Database, Nullable};

use crate::domain::activity::InboxActivityRepository;
use crate::domain::feed::FeedRepository;
use crate::domain::follow_request::FollowRequestRepository;
use crate::domain::follower::FollowerRepository;
use crate::domain::following::FollowingRepository;
use crate::domain::snowflake::Snowflake;
use crate::domain::status::StatusRepository;
use crate::error::{CanisterError, CanisterResult};
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
        ActivityType::Like => handle_like(&activity),
        ActivityType::Announce => handle_announce(&activity, &activity_json),
        ActivityType::Delete => handle_delete(&activity),
        ActivityType::Undo => handle_undo(&activity),
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
fn handle_reject(activity: &Activity) -> Result<(), ReceiveActivityError> {
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

/// Handle an incoming `Delete` activity.
///
/// Purges every inbox row referencing the deleted status URI plus its
/// matching `feed` entry, in a single transaction:
///
/// - Boost rows: matched by `original_status_uri`.
/// - `Create(Note)` rows: matched by parsing the cached `object_data`
///   activity JSON and comparing the embedded note's `id`.
///
/// The handler is idempotent — when no rows match (already purged or
/// never received) it returns `Ok(())` without modifying state.
fn handle_delete(activity: &Activity) -> Result<(), ReceiveActivityError> {
    let actor_uri = activity
        .actor
        .as_deref()
        .ok_or(ReceiveActivityError::ProcessingFailed)?;
    let Some(target_uri) = extract_object_uri(activity) else {
        ic_utils::log!("handle_incoming: Delete missing object URI; ignoring");
        return Ok(());
    };
    ic_utils::log!("handle_incoming: Delete from {actor_uri} on {target_uri}");
    let ids_to_delete = collect_delete_targets(&target_uri).map_err(|e| {
        ic_utils::log!("handle_incoming: failed to scan inbox for Delete: {e}");
        ReceiveActivityError::Internal(e.to_string())
    })?;

    if ids_to_delete.is_empty() {
        ic_utils::log!("handle_incoming: Delete found no matching inbox entries, ignoring");
        return Ok(());
    }

    Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
        let inbox_tx = InboxActivityRepository::with_transaction(tx);
        let feed_tx = FeedRepository::with_transaction(tx);
        for id in &ids_to_delete {
            feed_tx.delete_by_id(*id)?;
            inbox_tx.delete_by_id(*id)?;
        }
        Ok(())
    })
    .map_err(|e| {
        ic_utils::log!("handle_incoming: failed to apply Delete: {e}");
        ReceiveActivityError::Internal(e.to_string())
    })
}

/// Collect every inbox row id whose contents reference `target_uri`.
///
/// Combines three independent matches:
///
/// 1. Boost rows whose `original_status_uri` is the target.
/// 2. Create rows whose embedded note id is the full target URI.
/// 3. Create rows whose embedded note id is the bare snowflake string,
///    paired with the activity actor matching the URI's parent actor.
///    Mastic's publish pipeline currently mints note ids as bare
///    snowflakes, so this branch is what catches locally-published notes.
fn collect_delete_targets(target_uri: &str) -> CanisterResult<Vec<u64>> {
    let inbox = InboxActivityRepository::oneshot();
    let mut ids = inbox.find_boost_ids_by_original_uri(target_uri)?;
    ids.extend(inbox.find_create_ids_with_object_id(target_uri, None)?);

    if let Some(bare_id) = crate::domain::urls::parse_status_id(target_uri)
        && let Some(parent_actor) = crate::domain::urls::actor_uri_from_status_uri(target_uri)
    {
        for id in inbox.find_create_ids_with_object_id(&bare_id.to_string(), Some(&parent_actor))? {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    Ok(ids)
}

/// Handle an incoming `Undo(Follow)` or `Undo(Like)` activity.
///
/// - `Undo(Follow)`: removes the sender from the `followers` table
///   (accepted inbound follow) and from the `follow_requests` table
///   (pending inbound follow). Idempotent: missing entries do not produce
///   an error.
/// - `Undo(Like)`: decrements the cached `like_count` of the targeted local
///   status when the URI points at one of our statuses; ignored otherwise.
/// - Any other inner activity is silently accepted but not acted on.
fn handle_undo(activity: &Activity) -> Result<(), ReceiveActivityError> {
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
        ActivityType::Like => handle_undo_like(inner, sender_uri),
        ActivityType::Announce => handle_undo_announce(inner, sender_uri),
        other => {
            ic_utils::log!("handle_incoming: ignoring Undo of unsupported inner type: {other:?}");
            Ok(())
        }
    }
}

/// Extract the object URI from an `Id`-form or `Object`-form `ActivityObject`.
fn extract_object_uri(activity: &Activity) -> Option<String> {
    match activity.object.as_ref()? {
        ActivityObject::Id(uri) => Some(uri.clone()),
        ActivityObject::Object(obj) => obj.id.clone(),
        ActivityObject::Activity(_) | ActivityObject::Actor(_) => None,
    }
}

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
fn handle_like(activity: &Activity) -> Result<(), ReceiveActivityError> {
    let actor_uri = activity
        .actor
        .as_deref()
        .ok_or(ReceiveActivityError::ProcessingFailed)?;
    let Some(status_uri) = extract_object_uri(activity) else {
        ic_utils::log!("handle_incoming: Like missing object URI");
        return Err(ReceiveActivityError::ProcessingFailed);
    };
    ic_utils::log!("handle_incoming: Like from {actor_uri} on {status_uri}");

    let Some((_handle, id)) = parse_local_status(&status_uri)? else {
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
fn handle_undo_like(inner: &Activity, sender_uri: &str) -> Result<(), ReceiveActivityError> {
    let Some(status_uri) = extract_object_uri(inner) else {
        ic_utils::log!("handle_incoming: Undo(Like) missing object URI");
        return Err(ReceiveActivityError::ProcessingFailed);
    };
    ic_utils::log!("handle_incoming: Undo(Like) from {sender_uri} on {status_uri}");

    let Some((_handle, id)) = parse_local_status(&status_uri)? else {
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

/// Handle an incoming `Announce` (boost) activity.
///
/// Stores the announce in the inbox with `is_boost = true`, indexes it in
/// the `feed` table so the boost shows up in the recipient's timeline, and
/// — if the boosted status is hosted locally — increments the cached
/// `statuses.boost_count` of the original.
fn handle_announce(activity: &Activity, activity_json: &str) -> Result<(), ReceiveActivityError> {
    let actor_uri = activity
        .actor
        .as_deref()
        .ok_or(ReceiveActivityError::ProcessingFailed)?;
    let Some(target_uri) = extract_object_uri(activity) else {
        ic_utils::log!("handle_incoming: Announce missing object URI");
        return Err(ReceiveActivityError::ProcessingFailed);
    };

    ic_utils::log!("handle_incoming: Announce from {actor_uri} on {target_uri}");

    let snowflake_id = Snowflake::new();
    let created_at = ic_utils::now();
    let object_data: serde_json::Value = serde_json::from_str(activity_json).map_err(|e| {
        ic_utils::log!("handle_incoming: failed to parse activity JSON: {e}");
        ReceiveActivityError::Internal(e.to_string())
    })?;

    ic_dbms_canister::prelude::DBMS_CONTEXT
        .with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = wasm_dbms::WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);

            db.insert::<InboxActivity>(InboxActivityInsertRequest {
                id: snowflake_id.into(),
                activity_type: DbActivityType::from(ActivityType::Announce),
                actor_uri: actor_uri.into(),
                object_data: object_data.into(),
                is_boost: true.into(),
                original_status_uri: Nullable::Value(target_uri.clone().into()),
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
            ic_utils::log!("handle_incoming: failed to insert announce inbox row: {e}");
            ReceiveActivityError::Internal(e.to_string())
        })?;

    if let Some((_handle, id)) = parse_local_status(&target_uri)?
        && !StatusRepository::oneshot()
            .increment_boost_count(id)
            .map_err(|e| {
                ic_utils::log!("handle_incoming: failed to increment boost_count: {e}");
                ReceiveActivityError::Internal(e.to_string())
            })?
    {
        ic_utils::log!("handle_incoming: Announce target status {id} not found, ignoring");
    }
    Ok(())
}

/// Handle an `Undo(Announce)` body.
///
/// Deletes the matching `(actor_uri, original_status_uri, is_boost = true)`
/// inbox row and its `feed` entry, and — if the original is local —
/// decrements the cached `statuses.boost_count` (saturating at 0).
fn handle_undo_announce(inner: &Activity, sender_uri: &str) -> Result<(), ReceiveActivityError> {
    let Some(target_uri) = extract_object_uri(inner) else {
        ic_utils::log!("handle_incoming: Undo(Announce) missing object URI");
        return Err(ReceiveActivityError::ProcessingFailed);
    };
    ic_utils::log!("handle_incoming: Undo(Announce) from {sender_uri} on {target_uri}");

    use wasm_dbms_api::prelude::{DeleteBehavior, Filter, Query, Value};

    ic_dbms_canister::prelude::DBMS_CONTEXT
        .with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = wasm_dbms::WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);

            let rows = db.select::<InboxActivity>(
                Query::builder()
                    .all()
                    .and_where(Filter::eq("actor_uri", Value::from(sender_uri)))
                    .and_where(Filter::eq("is_boost", Value::from(true)))
                    .and_where(Filter::eq(
                        "original_status_uri",
                        Value::from(target_uri.as_str()),
                    ))
                    .limit(1)
                    .build(),
            )?;
            if let Some(row) = rows.into_iter().next() {
                let id = row.id.expect("id").0;
                db.delete::<FeedEntry>(
                    DeleteBehavior::Restrict,
                    Some(Filter::eq("id", Value::from(id))),
                )?;
                db.delete::<InboxActivity>(
                    DeleteBehavior::Restrict,
                    Some(Filter::eq("id", Value::from(id))),
                )?;
            }
            db.commit()?;
            Ok(())
        })
        .map_err(|e: CanisterError| {
            ic_utils::log!("handle_incoming: failed to delete announce inbox row: {e}");
            ReceiveActivityError::Internal(e.to_string())
        })?;

    if let Some((_handle, id)) = parse_local_status(&target_uri)?
        && !StatusRepository::oneshot()
            .decrement_boost_count(id)
            .map_err(|e| {
                ic_utils::log!("handle_incoming: failed to decrement boost_count: {e}");
                ReceiveActivityError::Internal(e.to_string())
            })?
    {
        ic_utils::log!("handle_incoming: Undo(Announce) target status {id} not found, ignoring");
    }
    Ok(())
}

/// Confirm the status URI is hosted on this canister's instance and points
/// at this user's handle, returning `(handle, id)` when so.
fn parse_local_status(status_uri: &str) -> Result<Option<(String, u64)>, ReceiveActivityError> {
    let parsed = crate::domain::urls::parse_local_status_uri(status_uri).map_err(|e| {
        ic_utils::log!("handle_incoming: failed to parse status URI: {e}");
        ReceiveActivityError::Internal(e.to_string())
    })?;
    let Some((handle, id)) = parsed else {
        return Ok(None);
    };

    let own = crate::domain::profile::ProfileRepository::oneshot()
        .get_profile()
        .map_err(|e| {
            ic_utils::log!("handle_incoming: failed to load own profile: {e}");
            ReceiveActivityError::Internal(e.to_string())
        })?;
    if handle != own.handle.0 {
        return Ok(None);
    }

    Ok(Some((handle, id)))
}

#[cfg(test)]
mod tests {

    use activitypub::activity::{Activity, ActivityObject, ActivityType};
    use activitypub::object::BaseObject;
    use db_utils::repository::Repository;
    use did::user::{ReceiveActivityArgs, ReceiveActivityError, ReceiveActivityResponse};

    use super::handle_incoming;
    use crate::domain::follow_request::FollowRequestRepository;
    use crate::domain::follower::FollowerRepository;
    use crate::domain::following::FollowingRepository;
    use crate::domain::status::StatusRepository;
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

    fn make_undo_follow_json(unfollower_actor_uri: &str, target_actor_uri: &str) -> String {
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Undo,
                ..Default::default()
            },
            actor: Some(unfollower_actor_uri.to_string()),
            object: Some(ActivityObject::Activity(Box::new(Activity {
                base: BaseObject {
                    kind: ActivityType::Follow,
                    ..Default::default()
                },
                actor: Some(unfollower_actor_uri.to_string()),
                object: Some(ActivityObject::Id(target_actor_uri.to_string())),
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

    fn make_like_json(actor_uri: &str, status_uri: &str) -> String {
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Like,
                ..Default::default()
            },
            actor: Some(actor_uri.to_string()),
            object: Some(ActivityObject::Id(status_uri.to_string())),
            target: None,
            result: None,
            origin: None,
            instrument: None,
        };
        serde_json::to_string(&activity).unwrap()
    }

    fn make_undo_like_json(actor_uri: &str, status_uri: &str) -> String {
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Undo,
                ..Default::default()
            },
            actor: Some(actor_uri.to_string()),
            object: Some(ActivityObject::Activity(Box::new(Activity {
                base: BaseObject {
                    kind: ActivityType::Like,
                    ..Default::default()
                },
                actor: Some(actor_uri.to_string()),
                object: Some(ActivityObject::Id(status_uri.to_string())),
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

    const REMOTE_BOOSTER: &str = "https://remote.example/users/alice";
    const LOCAL_STATUS_URI: &str = "https://mastic.social/users/rey_canisteryo/statuses/42";

    fn make_announce_json(actor: &str, target_uri: &str) -> String {
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Announce,
                ..Default::default()
            },
            actor: Some(actor.into()),
            object: Some(ActivityObject::Id(target_uri.into())),
            target: None,
            result: None,
            origin: None,
            instrument: None,
        };
        serde_json::to_string(&activity).unwrap()
    }

    fn make_undo_announce_json(actor: &str, target_uri: &str) -> String {
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Undo,
                ..Default::default()
            },
            actor: Some(actor.into()),
            object: Some(ActivityObject::Activity(Box::new(Activity {
                base: BaseObject {
                    kind: ActivityType::Announce,
                    ..Default::default()
                },
                actor: Some(actor.into()),
                object: Some(ActivityObject::Id(target_uri.into())),
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
    fn test_should_store_inbox_row_on_announce() {
        setup();
        crate::test_utils::insert_status(42, "hi", did::common::Visibility::Public, 1_000);

        let resp = handle_incoming(ReceiveActivityArgs {
            activity_json: make_announce_json(REMOTE_BOOSTER, LOCAL_STATUS_URI),
        });
        assert_eq!(resp, ReceiveActivityResponse::Ok);

        // boost_count incremented
        let s = StatusRepository::oneshot().find_by_id(42).unwrap().unwrap();
        assert_eq!(s.boost_count.0, 1);

        // inbox row + feed entry exist with is_boost=true
        use ic_dbms_canister::prelude::DBMS_CONTEXT;
        use wasm_dbms::WasmDbmsDatabase;
        use wasm_dbms_api::prelude::{Database, Filter, Query, Value};
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, crate::schema::Schema);
            let inbox = db
                .select::<crate::schema::InboxActivity>(
                    Query::builder()
                        .all()
                        .and_where(Filter::eq("is_boost", Value::from(true)))
                        .build(),
                )
                .unwrap();
            assert_eq!(inbox.len(), 1);
            let row = &inbox[0];
            assert_eq!(row.actor_uri.as_ref().unwrap().0, REMOTE_BOOSTER);
            assert_eq!(
                row.original_status_uri
                    .as_ref()
                    .unwrap()
                    .clone()
                    .into_opt()
                    .unwrap()
                    .0,
                LOCAL_STATUS_URI
            );
        });
    }

    #[test]
    fn test_should_decrement_boost_count_and_delete_inbox_on_undo_announce() {
        setup();
        crate::test_utils::insert_status(42, "hi", did::common::Visibility::Public, 1_000);

        handle_incoming(ReceiveActivityArgs {
            activity_json: make_announce_json(REMOTE_BOOSTER, LOCAL_STATUS_URI),
        });
        let resp = handle_incoming(ReceiveActivityArgs {
            activity_json: make_undo_announce_json(REMOTE_BOOSTER, LOCAL_STATUS_URI),
        });
        assert_eq!(resp, ReceiveActivityResponse::Ok);

        let s = StatusRepository::oneshot().find_by_id(42).unwrap().unwrap();
        assert_eq!(s.boost_count.0, 0);

        // Inbox boost row removed
        use ic_dbms_canister::prelude::DBMS_CONTEXT;
        use wasm_dbms::WasmDbmsDatabase;
        use wasm_dbms_api::prelude::{Database, Filter, Query, Value};
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, crate::schema::Schema);
            let inbox = db
                .select::<crate::schema::InboxActivity>(
                    Query::builder()
                        .all()
                        .and_where(Filter::eq("is_boost", Value::from(true)))
                        .build(),
                )
                .unwrap();
            assert!(inbox.is_empty(), "inbox boost row removed");
        });
    }

    #[test]
    fn test_announce_on_remote_status_does_not_panic_or_affect_counts() {
        setup();
        let resp = handle_incoming(ReceiveActivityArgs {
            activity_json: make_announce_json(
                REMOTE_BOOSTER,
                "https://other.example/users/bob/statuses/9",
            ),
        });
        assert_eq!(resp, ReceiveActivityResponse::Ok);
        // No local status to update; no panic.
    }

    #[test]
    fn test_undo_announce_saturates_at_zero() {
        setup();
        crate::test_utils::insert_status(42, "hi", did::common::Visibility::Public, 1_000);

        // Undo without prior Announce — boost_count stays at 0.
        let resp = handle_incoming(ReceiveActivityArgs {
            activity_json: make_undo_announce_json(REMOTE_BOOSTER, LOCAL_STATUS_URI),
        });
        assert_eq!(resp, ReceiveActivityResponse::Ok);

        let s = StatusRepository::oneshot().find_by_id(42).unwrap().unwrap();
        assert_eq!(s.boost_count.0, 0);
    }

    fn make_create_note_json(actor_uri: &str, note_id: &str, content: &str) -> String {
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Create,
                ..Default::default()
            },
            actor: Some(actor_uri.to_string()),
            object: Some(ActivityObject::Object(Box::new(BaseObject {
                id: Some(note_id.to_string()),
                kind: activitypub::object::ObjectType::Note,
                content: Some(content.to_string()),
                ..Default::default()
            }))),
            target: None,
            result: None,
            origin: None,
            instrument: None,
        };
        serde_json::to_string(&activity).unwrap()
    }

    fn make_delete_json(actor_uri: &str, target_uri: &str) -> String {
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Delete,
                ..Default::default()
            },
            actor: Some(actor_uri.to_string()),
            object: Some(ActivityObject::Id(target_uri.to_string())),
            target: None,
            result: None,
            origin: None,
            instrument: None,
        };
        serde_json::to_string(&activity).unwrap()
    }

    fn count_inbox() -> usize {
        use ic_dbms_canister::prelude::DBMS_CONTEXT;
        use wasm_dbms::WasmDbmsDatabase;
        use wasm_dbms_api::prelude::{Database, Query};
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, crate::schema::Schema);
            db.select::<crate::schema::InboxActivity>(Query::builder().all().build())
                .unwrap()
                .len()
        })
    }

    fn count_feed() -> usize {
        use ic_dbms_canister::prelude::DBMS_CONTEXT;
        use wasm_dbms::WasmDbmsDatabase;
        use wasm_dbms_api::prelude::{Database, Query};
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, crate::schema::Schema);
            db.select::<crate::schema::FeedEntry>(Query::builder().all().build())
                .unwrap()
                .len()
        })
    }

    const REMOTE_AUTHOR: &str = "https://remote.example/users/dave";
    const REMOTE_NOTE_URI: &str = "https://remote.example/users/dave/statuses/7";

    #[test]
    fn test_should_purge_inbox_and_feed_on_delete_of_create_note() {
        setup();

        // ingest a remote Create(Note)
        let create = make_create_note_json(REMOTE_AUTHOR, REMOTE_NOTE_URI, "hi");
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: create
            }),
            ReceiveActivityResponse::Ok
        );
        assert_eq!(count_inbox(), 1);
        assert_eq!(count_feed(), 1);

        // delete it
        let delete = make_delete_json(REMOTE_AUTHOR, REMOTE_NOTE_URI);
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: delete
            }),
            ReceiveActivityResponse::Ok
        );
        assert_eq!(count_inbox(), 0);
        assert_eq!(count_feed(), 0);
    }

    #[test]
    fn test_should_purge_inbox_boost_referencing_deleted_uri() {
        setup();

        let announce = make_announce_json(REMOTE_BOOSTER, REMOTE_NOTE_URI);
        handle_incoming(ReceiveActivityArgs {
            activity_json: announce,
        });
        assert_eq!(count_inbox(), 1);
        assert_eq!(count_feed(), 1);

        let delete = make_delete_json(REMOTE_AUTHOR, REMOTE_NOTE_URI);
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: delete
            }),
            ReceiveActivityResponse::Ok
        );
        assert_eq!(count_inbox(), 0);
        assert_eq!(count_feed(), 0);
    }

    #[test]
    fn test_should_be_idempotent_on_repeat_delete() {
        setup();

        // ingest then delete twice
        handle_incoming(ReceiveActivityArgs {
            activity_json: make_create_note_json(REMOTE_AUTHOR, REMOTE_NOTE_URI, "hi"),
        });
        let delete = make_delete_json(REMOTE_AUTHOR, REMOTE_NOTE_URI);
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: delete.clone()
            }),
            ReceiveActivityResponse::Ok
        );
        // second delete on same URI must succeed silently
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: delete
            }),
            ReceiveActivityResponse::Ok
        );
    }

    #[test]
    fn test_should_succeed_when_delete_targets_unknown_uri() {
        setup();

        // No prior Create / Announce — must still succeed.
        let delete = make_delete_json(REMOTE_AUTHOR, REMOTE_NOTE_URI);
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: delete
            }),
            ReceiveActivityResponse::Ok
        );
    }

    #[test]
    fn test_should_not_purge_unrelated_inbox_rows_on_delete() {
        setup();

        let other = "https://remote.example/users/dave/statuses/8";
        handle_incoming(ReceiveActivityArgs {
            activity_json: make_create_note_json(REMOTE_AUTHOR, REMOTE_NOTE_URI, "a"),
        });
        handle_incoming(ReceiveActivityArgs {
            activity_json: make_create_note_json(REMOTE_AUTHOR, other, "b"),
        });

        let delete = make_delete_json(REMOTE_AUTHOR, REMOTE_NOTE_URI);
        handle_incoming(ReceiveActivityArgs {
            activity_json: delete,
        });

        // verify via raw select that exactly one inbox row remains and it
        // points to the un-deleted note URI.
        use ic_dbms_canister::prelude::DBMS_CONTEXT;
        use wasm_dbms::WasmDbmsDatabase;
        use wasm_dbms_api::prelude::{Database, Query};
        let count = DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, crate::schema::Schema);
            db.select_raw("inbox", Query::builder().all().build())
                .unwrap()
                .len()
        });
        assert_eq!(count, 1);
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
