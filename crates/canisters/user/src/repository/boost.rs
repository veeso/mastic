use db_utils::repository::Repository;
use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms_api::prelude::{Database, DeleteBehavior, Filter, Query, TransactionId, Value};

use crate::error::CanisterResult;
use crate::schema::{Boost, BoostInsertRequest, BoostRecord, Schema};

/// Repository for managing [`Boost`] records in the database.
pub struct BoostRepository {
    tx: Option<TransactionId>,
}

impl BoostRepository {
    /// Insert a [`Boost`] row. The same `snowflake_id` is reused as the
    /// primary key in the wrapper `statuses` and `feed` tables — that
    /// orchestration is owned by the boost domain helper, not this repo.
    pub fn insert(
        &self,
        snowflake_id: u64,
        original_status_uri: &str,
        created_at: u64,
    ) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx).insert::<Boost>(BoostInsertRequest {
                id: snowflake_id.into(),
                status_id: snowflake_id.into(),
                original_status_uri: original_status_uri.into(),
                created_at: created_at.into(),
            })?;
            Ok(())
        })
    }

    /// Delete the [`Boost`] row whose primary key matches `snowflake_id`.
    ///
    /// Uses [`DeleteBehavior::Restrict`] — callers must drop the wrapper
    /// `statuses` and `feed` rows in the right order so referential integrity
    /// is preserved.
    pub fn delete_by_id(&self, snowflake_id: u64) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx).delete::<Boost>(
                DeleteBehavior::Restrict,
                Some(Filter::eq("id", Value::from(snowflake_id))),
            )?;
            Ok(())
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
            .map_err(crate::error::CanisterError::from)
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
            .map_err(crate::error::CanisterError::from)
    }
}

impl Repository for BoostRepository {
    type Schema = Schema;

    fn schema() -> Self::Schema {
        Schema
    }

    fn oneshot() -> Self {
        Self { tx: None }
    }

    fn with_transaction(tx: TransactionId) -> Self {
        Self { tx: Some(tx) }
    }

    fn tx(&self) -> Option<TransactionId> {
        self.tx
    }
}

#[cfg(test)]
mod tests {

    use did::common::Visibility;

    use super::*;
    use crate::test_utils::{insert_status, setup};

    const STATUS_URI_A: &str = "https://example.com/users/alice/statuses/1";

    #[test]
    fn test_insert_should_persist_boost_row() {
        setup();
        // The boost row has a FK to a wrapper `statuses` row sharing the same
        // primary key — insert it first so the FK constraint holds.
        insert_status(10, "wrapper", Visibility::Public, 1_000_000);

        BoostRepository::oneshot()
            .insert(10, STATUS_URI_A, 1_000_000)
            .expect("insert");

        let row = BoostRepository::oneshot()
            .find_by_original_uri(STATUS_URI_A)
            .expect("query")
            .expect("row exists");
        assert_eq!(row.id.expect("id").0, 10);
        assert_eq!(
            row.original_status_uri.expect("uri").0,
            STATUS_URI_A.to_string()
        );
        assert_eq!(row.created_at.expect("created_at").0, 1_000_000);
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
        insert_status(100, "wrapper", Visibility::Public, 1_000_000);
        BoostRepository::oneshot()
            .insert(100, STATUS_URI_A, 1_000_000)
            .expect("insert");

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
    fn test_delete_by_id_should_remove_boost_row() {
        setup();
        insert_status(11, "wrapper", Visibility::Public, 1_000);
        BoostRepository::oneshot()
            .insert(11, STATUS_URI_A, 1_000)
            .expect("insert");

        BoostRepository::oneshot()
            .delete_by_id(11)
            .expect("should delete");

        assert!(
            !BoostRepository::oneshot()
                .is_boosted(STATUS_URI_A)
                .expect("query")
        );
    }

    #[test]
    fn test_delete_by_id_should_be_noop_when_missing() {
        setup();

        BoostRepository::oneshot()
            .delete_by_id(999)
            .expect("delete missing must succeed");
    }
}
