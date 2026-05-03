//! Repository for the `feed` table.
//!
//! The `feed` table is a denormalized timeline that indexes both outbox
//! entries (own statuses and boost wrappers) and inbox entries (received
//! `Create` / `Announce` activities) under a single sorted timeline.
//!
//! Writes go through this repository so the lifecycle is centralised. Reads
//! are still served by [`crate::domain::feed::read_feed`] which performs the
//! full hydration join across `statuses` / `inbox` / `boosts`.

use ic_dbms_canister::prelude::{DBMS_CONTEXT, IcAccessControlList, IcMemoryProvider};
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms::prelude::DbmsContext;
use wasm_dbms_api::prelude::{Database, DeleteBehavior, Filter, TransactionId, Value};

use crate::error::CanisterResult;
use crate::schema::{FeedEntry, FeedEntryInsertRequest, FeedSource, Schema};

/// Repository over the denormalized `feed` table.
pub struct FeedRepository {
    tx: Option<TransactionId>,
}

impl FeedRepository {
    /// Build a repository instance that runs each operation in its own
    /// auto-committed oneshot transaction.
    //
    // `oneshot` callers land with the boost / inbox refactors; for now it is
    // exercised only by the in-module `#[cfg(test)]` suite.
    #[allow(dead_code)]
    pub const fn oneshot() -> Self {
        Self { tx: None }
    }

    /// Build a repository instance that splices its writes into an
    /// externally-driven transaction. Lifecycle (commit/rollback) is owned by
    /// the caller — typically via [`db_utils::transaction::Transaction::run`].
    pub const fn with_transaction(tx: TransactionId) -> Self {
        Self { tx: Some(tx) }
    }

    fn db<'a>(
        &self,
        ctx: &'a DbmsContext<IcMemoryProvider, IcAccessControlList>,
    ) -> WasmDbmsDatabase<'a, IcMemoryProvider, IcAccessControlList> {
        match self.tx {
            Some(id) => WasmDbmsDatabase::from_transaction(ctx, Schema, id),
            None => WasmDbmsDatabase::oneshot(ctx, Schema),
        }
    }

    /// Insert a feed entry tagged as [`FeedSource::Outbox`] — used for own
    /// statuses and boost wrappers.
    pub fn insert_outbox(&self, snowflake_id: u64, created_at: u64) -> CanisterResult<()> {
        self.insert(snowflake_id, FeedSource::Outbox, created_at)
    }

    /// Insert a feed entry tagged as [`FeedSource::Inbox`] — used for received
    /// `Create` / `Announce` activities.
    //
    // Wired in by the inbox refactor; covered by the in-module test suite
    // until then.
    #[allow(dead_code)]
    pub fn insert_inbox(&self, snowflake_id: u64, created_at: u64) -> CanisterResult<()> {
        self.insert(snowflake_id, FeedSource::Inbox, created_at)
    }

    fn insert(&self, snowflake_id: u64, source: FeedSource, created_at: u64) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx).insert::<FeedEntry>(FeedEntryInsertRequest {
                id: snowflake_id.into(),
                source,
                created_at: created_at.into(),
            })?;
            Ok(())
        })
    }

    /// Remove the [`FeedEntry`] row whose primary key matches `snowflake_id`.
    ///
    /// Uses [`DeleteBehavior::Restrict`] — callers must drop the corresponding
    /// `statuses` / `inbox` / `boosts` rows in the right order so referential
    /// integrity is preserved.
    //
    // Wired in by the boost refactor (undo_boost flow); covered by the
    // in-module test suite until then.
    #[allow(dead_code)]
    pub fn delete_by_id(&self, snowflake_id: u64) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx).delete::<FeedEntry>(
                DeleteBehavior::Restrict,
                Some(Filter::eq("id", Value::from(snowflake_id))),
            )?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use db_utils::transaction::Transaction;
    use wasm_dbms_api::prelude::Query;

    use super::*;
    use crate::error::CanisterError;
    use crate::test_utils::setup;

    fn select_all_entries() -> Vec<FeedEntry> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.select::<FeedEntry>(Query::builder().all().build())
                .expect("select feed entries")
                .into_iter()
                .map(|r| FeedEntry {
                    id: r.id.expect("id"),
                    source: r.source.expect("source"),
                    created_at: r.created_at.expect("created_at"),
                })
                .collect()
        })
    }

    #[test]
    fn test_should_insert_outbox_entry() {
        setup();

        FeedRepository::oneshot()
            .insert_outbox(1, 1_000)
            .expect("should insert outbox entry");

        let entries = select_all_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id.0, 1);
        assert_eq!(entries[0].source, FeedSource::Outbox);
        assert_eq!(entries[0].created_at.0, 1_000);
    }

    #[test]
    fn test_should_insert_inbox_entry() {
        setup();

        FeedRepository::oneshot()
            .insert_inbox(42, 2_000)
            .expect("should insert inbox entry");

        let entries = select_all_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id.0, 42);
        assert_eq!(entries[0].source, FeedSource::Inbox);
        assert_eq!(entries[0].created_at.0, 2_000);
    }

    #[test]
    fn test_should_delete_by_id() {
        setup();
        let repo = FeedRepository::oneshot();
        repo.insert_outbox(7, 1_000).expect("insert");
        assert_eq!(select_all_entries().len(), 1);

        repo.delete_by_id(7).expect("delete");

        assert!(select_all_entries().is_empty());
    }

    #[test]
    fn test_should_insert_via_transaction_run() {
        setup();

        Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
            FeedRepository::with_transaction(tx).insert_outbox(99, 5_000)
        })
        .expect("transaction should commit");

        let entries = select_all_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id.0, 99);
        assert_eq!(entries[0].source, FeedSource::Outbox);
        assert_eq!(entries[0].created_at.0, 5_000);
    }

    #[test]
    fn test_rolls_back_when_tx_errors() {
        setup();

        let result: Result<(), CanisterError> = Transaction::run(Schema, |tx| {
            FeedRepository::with_transaction(tx).insert_outbox(123, 9_000)?;
            Err(CanisterError::Internal("boom".to_string()))
        });
        assert!(result.is_err());

        assert!(
            select_all_entries().is_empty(),
            "errored tx must not persist its insert"
        );
    }
}
