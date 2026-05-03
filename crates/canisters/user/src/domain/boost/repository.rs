use ic_dbms_canister::prelude::{DBMS_CONTEXT, IcAccessControlList, IcMemoryProvider};
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms::prelude::DbmsContext;
use wasm_dbms_api::prelude::{
    Database, DeleteBehavior, Filter, Nullable, Query, TransactionId, Value,
};

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{
    Boost, BoostInsertRequest, BoostRecord, FeedEntry, FeedEntryInsertRequest, FeedSource, Schema,
    Status, StatusInsertRequest, Visibility,
};

/// Repository for managing [`Boost`] records in the database.
pub struct BoostRepository {
    tx: Option<TransactionId>,
}

impl BoostRepository {
    pub const fn oneshot() -> Self {
        Self { tx: None }
    }

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

    /// Insert a wrapper [`Status`] row, the corresponding [`Boost`] row, and a
    /// `feed` entry with `source = Outbox`. The same `snowflake_id` is used as
    /// the primary key in all three tables — this makes the wrapper status URL
    /// `<actor>/statuses/<snowflake>` also serve as the `Announce` activity id.
    ///
    /// Transaction lifecycle is owned by the caller — typically via
    /// `Transaction::run` — so the three writes are atomic.
    #[allow(clippy::too_many_arguments)]
    pub fn insert_boost_with_wrapper(
        &self,
        snowflake_id: u64,
        original_status_uri: &str,
        content: &str,
        visibility: Visibility,
        spoiler_text: Option<&str>,
        sensitive: bool,
        created_at: u64,
    ) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            let db = self.db(ctx);

            db.insert::<Status>(StatusInsertRequest {
                id: snowflake_id.into(),
                content: content.into(),
                visibility,
                like_count: 0u64.into(),
                boost_count: 0u64.into(),
                in_reply_to_uri: Nullable::Null,
                spoiler_text: spoiler_text.map_or(Nullable::Null, |s| Nullable::Value(s.into())),
                sensitive: sensitive.into(),
                edited_at: Nullable::Null,
                created_at: created_at.into(),
            })?;

            db.insert::<Boost>(BoostInsertRequest {
                id: snowflake_id.into(),
                status_id: snowflake_id.into(),
                original_status_uri: original_status_uri.into(),
                created_at: created_at.into(),
            })?;

            db.insert::<FeedEntry>(FeedEntryInsertRequest {
                id: snowflake_id.into(),
                source: FeedSource::Outbox,
                created_at: created_at.into(),
            })?;

            Ok::<(), CanisterError>(())
        })
    }

    /// Delete the [`Boost`] row, the wrapper [`Status`] row, and the
    /// corresponding `feed` entry sharing the same `snowflake_id`.
    ///
    /// Order: child (`boosts`) → `feed` → parent (`statuses`) so the FK
    /// `boosts.status_id → statuses.id` (Restrict) is satisfied.
    ///
    /// Transaction lifecycle is owned by the caller — typically via
    /// `Transaction::run` — so the three deletes are atomic.
    pub fn delete_boost_with_wrapper(&self, snowflake_id: u64) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            let db = self.db(ctx);

            db.delete::<Boost>(
                DeleteBehavior::Restrict,
                Some(Filter::eq("id", Value::from(snowflake_id))),
            )?;
            db.delete::<FeedEntry>(
                DeleteBehavior::Restrict,
                Some(Filter::eq("id", Value::from(snowflake_id))),
            )?;
            db.delete::<Status>(
                DeleteBehavior::Restrict,
                Some(Filter::eq("id", Value::from(snowflake_id))),
            )?;

            Ok::<(), CanisterError>(())
        })
    }

    /// Checks whether the user has boosted the given status.
    pub fn is_boosted(&self, original_status_uri: &str) -> CanisterResult<bool> {
        DBMS_CONTEXT
            .with(|ctx| {
                self.db(ctx).select::<Boost>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "original_status_uri",
                            original_status_uri.into(),
                        ))
                        .limit(1)
                        .build(),
                )
            })
            .map(|rows| !rows.is_empty())
            .map_err(CanisterError::from)
    }

    /// Looks up a boost record by the URI of the original (boosted) status.
    ///
    /// Returns `Ok(None)` when no matching row exists.
    pub fn find_by_original_uri(&self, uri: &str) -> CanisterResult<Option<BoostRecord>> {
        DBMS_CONTEXT
            .with(|ctx| {
                self.db(ctx).select::<Boost>(
                    Query::builder()
                        .all()
                        .and_where(Filter::eq("original_status_uri", uri.into()))
                        .limit(1)
                        .build(),
                )
            })
            .map(|rows| rows.into_iter().next())
            .map_err(CanisterError::from)
    }
}

#[cfg(test)]
mod tests {

    use db_utils::transaction::Transaction;

    use super::*;
    use crate::test_utils::setup;

    const STATUS_URI_A: &str = "https://example.com/users/alice/statuses/1";

    fn insert_boost(snowflake_id: u64, original_status_uri: &str) {
        Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
            BoostRepository::with_transaction(tx).insert_boost_with_wrapper(
                snowflake_id,
                original_status_uri,
                "boosted",
                crate::schema::Visibility::from(did::common::Visibility::Public),
                None,
                false,
                1_000_000,
            )
        })
        .expect("insert boost");
    }

    #[test]
    fn test_should_insert_and_check_boost() {
        setup();
        insert_boost(10, STATUS_URI_A);

        assert!(
            BoostRepository::oneshot()
                .is_boosted(STATUS_URI_A)
                .expect("should query")
        );
    }

    #[test]
    fn test_should_return_false_for_missing_boost() {
        setup();

        assert!(
            !BoostRepository::oneshot()
                .is_boosted(STATUS_URI_A)
                .expect("should query")
        );
    }

    #[test]
    fn test_should_find_by_original_uri() {
        setup();
        insert_boost(100, STATUS_URI_A);

        let found = BoostRepository::oneshot()
            .find_by_original_uri(STATUS_URI_A)
            .expect("should query")
            .expect("should find row");
        assert_eq!(found.id.expect("id").0, 100);
        assert_eq!(
            found.original_status_uri.expect("uri").0,
            STATUS_URI_A.to_string()
        );
    }

    #[test]
    fn test_find_by_original_uri_returns_none_when_missing() {
        setup();
        let found = BoostRepository::oneshot()
            .find_by_original_uri(STATUS_URI_A)
            .expect("should query");
        assert!(found.is_none());
    }

    #[test]
    fn test_should_delete_boost_with_wrapper_in_one_tx() {
        setup();

        Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
            BoostRepository::with_transaction(tx).insert_boost_with_wrapper(
                77,
                STATUS_URI_A,
                "boosted",
                crate::schema::Visibility::from(did::common::Visibility::Public),
                None,
                false,
                1_000,
            )
        })
        .expect("insert");

        Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
            BoostRepository::with_transaction(tx).delete_boost_with_wrapper(77)
        })
        .expect("delete");

        assert!(
            !BoostRepository::oneshot()
                .is_boosted(STATUS_URI_A)
                .expect("query")
        );
        assert!(
            crate::domain::status::StatusRepository::oneshot()
                .find_by_id(77)
                .expect("query")
                .is_none(),
            "wrapper status row removed"
        );
    }

    #[test]
    fn test_should_insert_boost_with_wrapper_in_one_tx() {
        setup();

        Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
            BoostRepository::with_transaction(tx).insert_boost_with_wrapper(
                42,
                STATUS_URI_A,
                "boosted text",
                crate::schema::Visibility::from(did::common::Visibility::Public),
                Some("cw"),
                true,
                1_000_000,
            )
        })
        .expect("insert");

        // Wrapper Status row
        let wrapper = crate::domain::status::StatusRepository::oneshot()
            .find_by_id(42)
            .expect("query")
            .expect("wrapper exists");
        assert_eq!(wrapper.content.0, "boosted text");
        assert_eq!(
            wrapper
                .spoiler_text
                .clone()
                .into_opt()
                .expect("spoiler value")
                .0,
            "cw"
        );
        assert!(wrapper.sensitive.0);

        // Boost row
        let boost = BoostRepository::oneshot()
            .find_by_original_uri(STATUS_URI_A)
            .expect("query")
            .expect("boost exists");
        assert_eq!(boost.id.expect("id").0, 42);
        assert_eq!(
            boost.original_status_uri.expect("uri").0,
            STATUS_URI_A.to_string()
        );

        // Idempotency check picks it up
        assert!(
            BoostRepository::oneshot()
                .is_boosted(STATUS_URI_A)
                .expect("query")
        );
    }

    #[test]
    fn test_insert_boost_with_wrapper_rolls_back_on_error() {
        setup();

        let result: Result<(), CanisterError> = Transaction::run(Schema, |tx| {
            BoostRepository::with_transaction(tx).insert_boost_with_wrapper(
                55,
                STATUS_URI_A,
                "rolled back",
                crate::schema::Visibility::from(did::common::Visibility::Public),
                None,
                false,
                500,
            )?;
            Err(CanisterError::Internal("boom".to_string()))
        });
        assert!(result.is_err());

        assert!(
            !BoostRepository::oneshot()
                .is_boosted(STATUS_URI_A)
                .expect("query")
        );
        assert!(
            crate::domain::status::StatusRepository::oneshot()
                .find_by_id(55)
                .expect("query")
                .is_none(),
            "wrapper status must not persist on rollback"
        );
    }
}
