//! Status repository

use did::common::Visibility;
use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::*;

use crate::domain::snowflake::Snowflake;
use crate::error::CanisterResult;
use crate::schema::{
    FeedEntry, FeedEntryInsertRequest, FeedSource, Schema, Status, StatusInsertRequest,
    StatusRecord,
};

/// Interface for the [`Status`] repository.
pub struct StatusRepository;

impl StatusRepository {
    /// Create a new [`Status`] with the given content, timestamp and visibility.
    ///
    /// A new [`Snowflake`] ID is generated for the status, and the current timestamp is used as the creation time.
    ///
    /// Both the `statuses` row and the corresponding `feed` entry are
    /// inserted inside a single transaction.
    ///
    /// In case of success the [`Snowflake`] ID of the newly created status is returned.
    pub fn create(
        content: String,
        visibility: Visibility,
        created_at: u64,
    ) -> CanisterResult<Snowflake> {
        let snowflake_id = Snowflake::new();

        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);

            db.insert::<Status>(StatusInsertRequest {
                id: snowflake_id.into(),
                content: content.into(),
                visibility: visibility.into(),
                created_at: created_at.into(),
            })?;

            db.insert::<FeedEntry>(FeedEntryInsertRequest {
                id: snowflake_id.into(),
                source: FeedSource::Outbox,
                created_at: created_at.into(),
            })?;

            db.commit()
        })?;

        Ok(snowflake_id)
    }

    /// Get a paginated list of [`Status`]es with the given visibility, ordered from the most recent to the oldest.
    ///
    /// This function is used to implement the `get_statuses` API, and the returned statuses are filtered based on the relationship of the caller with the user (owner, follower, or other).
    pub fn get_paginated_by_visibility(
        visibility: &[Visibility],
        offset: usize,
        limit: usize,
    ) -> CanisterResult<Vec<Status>> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            let mut query = Query::builder().all().limit(limit).offset(offset);
            for v in visibility {
                let visibility = crate::schema::Visibility::from(*v);
                query = query.and_where(Filter::eq("visibility", visibility.into()));
            }

            let results = db.select::<Status>(query.build())?;
            Ok(results.into_iter().map(Self::record_to_status).collect())
        })
    }

    fn record_to_status(record: StatusRecord) -> Status {
        Status {
            id: record.id.expect("must have field"),
            content: record.content.expect("must have field"),
            visibility: record.visibility.expect("must have field"),
            created_at: record.created_at.expect("must have field"),
        }
    }
}
