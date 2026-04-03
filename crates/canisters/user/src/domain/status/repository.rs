//! Status repository

use did::common::Visibility;
use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::*;

use crate::domain::snowflake::Snowflake;
use crate::error::CanisterResult;
use crate::schema::{Schema, Status, StatusInsertRequest};

/// Interface for the [`Status`] repository.
pub struct StatusRepository;

impl StatusRepository {
    /// Create a new [`Status`] with the given content, timestamp and visibility.
    ///
    /// A new [`Snowflake`] ID is generated for the status, and the current timestamp is used as the creation time.
    ///
    /// In case of success the [`Snowflake`] ID of the newly created status is returned.
    pub fn create(
        content: String,
        visibility: Visibility,
        created_at: u64,
    ) -> CanisterResult<Snowflake> {
        let snowflake_id = Snowflake::new();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            // insert the new status into the database
            let status_insert_request = StatusInsertRequest {
                id: snowflake_id.into(),
                content: content.into(),
                visibility: visibility.into(),
                created_at: created_at.into(),
            };
            db.insert::<Status>(status_insert_request)
        })?;

        Ok(snowflake_id)
    }
}
