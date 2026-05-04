//! Handle `Create` activity.

use activitypub::Activity;
use activitypub::activity::ActivityType;
use did::user::ReceiveActivityError;
use wasm_dbms_api::prelude::{Database, Nullable};

use crate::domain::snowflake::Snowflake;
use crate::error::CanisterError;
use crate::schema::{
    ActivityType as DbActivityType, FeedEntry, FeedEntryInsertRequest, FeedSource, InboxActivity,
    InboxActivityInsertRequest, Schema,
};

/// Handle an incoming `Create` activity (e.g. `Create(Note)`).
///
/// Stores the activity in the inbox and records a feed entry so that the
/// status appears in the owner's chronological feed.
pub(super) fn handle_create(
    activity: &Activity,
    activity_json: &str,
) -> Result<(), ReceiveActivityError> {
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
