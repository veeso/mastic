//! Handle `Announce` (boost) and `Undo(Announce)` activities.

use activitypub::Activity;
use activitypub::activity::ActivityType;
use db_utils::repository::Repository;
use did::user::ReceiveActivityError;
use wasm_dbms_api::prelude::{Database, DeleteBehavior, Filter, Nullable, Query, Value};

use crate::domain::snowflake::Snowflake;
use crate::error::CanisterError;
use crate::repository::status::StatusRepository;
use crate::schema::{
    ActivityType as DbActivityType, FeedEntry, FeedEntryInsertRequest, FeedSource, InboxActivity,
    InboxActivityInsertRequest, Schema,
};

/// Handle an incoming `Announce` (boost) activity.
///
/// Stores the announce in the inbox with `is_boost = true`, indexes it in
/// the `feed` table so the boost shows up in the recipient's timeline, and
/// — if the boosted status is hosted locally — increments the cached
/// `statuses.boost_count` of the original.
pub(super) fn handle_announce(
    activity: &Activity,
    activity_json: &str,
) -> Result<(), ReceiveActivityError> {
    let actor_uri = activity
        .actor
        .as_deref()
        .ok_or(ReceiveActivityError::ProcessingFailed)?;
    let Some(target_uri) = super::extract_object_uri(activity) else {
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

    if let Some((_handle, id)) = super::parse_local_status(&target_uri)?
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
pub(super) fn handle_undo_announce(
    inner: &Activity,
    sender_uri: &str,
) -> Result<(), ReceiveActivityError> {
    let Some(target_uri) = super::extract_object_uri(inner) else {
        ic_utils::log!("handle_incoming: Undo(Announce) missing object URI");
        return Err(ReceiveActivityError::ProcessingFailed);
    };
    ic_utils::log!("handle_incoming: Undo(Announce) from {sender_uri} on {target_uri}");

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

    if let Some((_handle, id)) = super::parse_local_status(&target_uri)?
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

#[cfg(test)]
mod tests {

    use db_utils::repository::Repository;
    use did::user::{ReceiveActivityArgs, ReceiveActivityResponse};

    use super::super::handle_incoming;
    use super::super::test_helpers::{
        LOCAL_STATUS_URI, REMOTE_BOOSTER, make_announce_json, make_undo_announce_json,
    };
    use crate::repository::status::StatusRepository;
    use crate::test_utils::setup;

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
}
