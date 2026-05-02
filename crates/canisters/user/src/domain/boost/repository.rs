use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::{Database, DeleteBehavior, Filter, Nullable, Query, Value};

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{
    Boost, BoostInsertRequest, BoostRecord, FeedEntry, FeedEntryInsertRequest, FeedSource, Schema,
    Status, StatusInsertRequest, Visibility,
};

/// Repository for managing [`Boost`] records in the database.
pub struct BoostRepository;

impl BoostRepository {
    /// Inserts a boost into the database.
    pub fn boost_status(id: u64, status_id: u64, original_status_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT
            .with(|ctx| {
                let db = WasmDbmsDatabase::oneshot(ctx, Schema);

                db.insert::<Boost>(BoostInsertRequest {
                    id: id.into(),
                    status_id: status_id.into(),
                    original_status_uri: original_status_uri.into(),
                    created_at: ic_utils::now().into(),
                })
            })
            .map_err(CanisterError::from)
    }

    /// Transactionally insert a wrapper [`Status`] row, the corresponding
    /// [`Boost`] row, and a `feed` entry with `source = Outbox`. The same
    /// `snowflake_id` is used as the primary key in all three tables — this
    /// makes the wrapper status URL `<actor>/statuses/<snowflake>` also serve
    /// as the `Announce` activity id.
    pub fn insert_boost_with_wrapper(
        snowflake_id: u64,
        original_status_uri: &str,
        content: &str,
        visibility: Visibility,
        spoiler_text: Option<&str>,
        sensitive: bool,
        created_at: u64,
    ) -> CanisterResult<()> {
        DBMS_CONTEXT
            .with(|ctx| {
                let tx_id = ctx
                    .begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
                let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);

                db.insert::<Status>(StatusInsertRequest {
                    id: snowflake_id.into(),
                    content: content.into(),
                    visibility,
                    like_count: 0u64.into(),
                    boost_count: 0u64.into(),
                    in_reply_to_uri: Nullable::Null,
                    spoiler_text: spoiler_text
                        .map_or(Nullable::Null, |s| Nullable::Value(s.into())),
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

                db.commit()
            })
            .map_err(CanisterError::from)
    }

    /// Transactionally delete the [`Boost`] row, the wrapper [`Status`] row,
    /// and the corresponding `feed` entry sharing the same `snowflake_id`.
    ///
    /// Order: child (`boosts`) → `feed` → parent (`statuses`) so the FK
    /// `boosts.status_id → statuses.id` (Restrict) is satisfied.
    pub fn delete_boost_with_wrapper(snowflake_id: u64) -> CanisterResult<()> {
        DBMS_CONTEXT
            .with(|ctx| {
                let tx_id = ctx
                    .begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
                let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);

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
                db.commit()
            })
            .map_err(CanisterError::from)
    }

    /// Deletes a boost from the database, identified by the boosted status URI.
    pub fn unboost_status(original_status_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT
            .with(|ctx| {
                let db = WasmDbmsDatabase::oneshot(ctx, Schema);

                db.delete::<Boost>(
                    DeleteBehavior::Cascade,
                    Some(Filter::eq(
                        "original_status_uri",
                        original_status_uri.into(),
                    )),
                )
            })
            .map(|_| ())
            .map_err(CanisterError::from)
    }

    /// Checks whether the user has boosted the given status.
    pub fn is_boosted(original_status_uri: &str) -> CanisterResult<bool> {
        DBMS_CONTEXT
            .with(|ctx| {
                let db = WasmDbmsDatabase::oneshot(ctx, Schema);

                db.select::<Boost>(
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
    pub fn find_by_original_uri(uri: &str) -> CanisterResult<Option<BoostRecord>> {
        DBMS_CONTEXT
            .with(|ctx| {
                let db = WasmDbmsDatabase::oneshot(ctx, Schema);

                db.select::<Boost>(
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

    /// Returns the URIs of statuses boosted by the user.
    pub fn get_boosts(offset: usize, limit: usize) -> CanisterResult<Vec<String>> {
        DBMS_CONTEXT
            .with(|ctx| {
                let db = WasmDbmsDatabase::oneshot(ctx, Schema);

                db.select::<Boost>(Query::builder().all().offset(offset).limit(limit).build())
            })
            .map(|records| {
                records
                    .into_iter()
                    .map(|record| record.original_status_uri.unwrap_or_default().0)
                    .collect()
            })
            .map_err(CanisterError::from)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::{insert_status, setup};

    const STATUS_URI_A: &str = "https://example.com/users/alice/statuses/1";
    const STATUS_URI_B: &str = "https://example.com/users/alice/statuses/2";
    const STATUS_URI_C: &str = "https://example.com/users/alice/statuses/3";

    fn seed_status(id: u64) {
        insert_status(id, "Hello", did::common::Visibility::Public, 1_000_000);
    }

    #[test]
    fn test_should_insert_and_check_boost() {
        setup();
        seed_status(10);

        BoostRepository::boost_status(100, 10, STATUS_URI_A).expect("should insert boost");

        assert!(BoostRepository::is_boosted(STATUS_URI_A).expect("should query"));
    }

    #[test]
    fn test_should_return_false_for_missing_boost() {
        setup();

        assert!(!BoostRepository::is_boosted(STATUS_URI_A).expect("should query"));
    }

    #[test]
    fn test_should_unboost_status() {
        setup();
        seed_status(10);

        BoostRepository::boost_status(100, 10, STATUS_URI_A).expect("should insert boost");
        assert!(BoostRepository::is_boosted(STATUS_URI_A).expect("should query"));

        BoostRepository::unboost_status(STATUS_URI_A).expect("should delete boost");

        assert!(!BoostRepository::is_boosted(STATUS_URI_A).expect("should query"));
    }

    #[test]
    fn test_should_unboost_missing_without_error() {
        setup();

        BoostRepository::unboost_status(STATUS_URI_A).expect("should delete missing boost");
    }

    #[test]
    fn test_should_get_boosts_paginated() {
        setup();
        seed_status(10);
        seed_status(20);
        seed_status(30);

        BoostRepository::boost_status(100, 10, STATUS_URI_A).expect("should insert boost");
        BoostRepository::boost_status(200, 20, STATUS_URI_B).expect("should insert boost");
        BoostRepository::boost_status(300, 30, STATUS_URI_C).expect("should insert boost");

        let all = BoostRepository::get_boosts(0, 10).expect("should query");
        assert_eq!(all.len(), 3);
        assert!(all.contains(&STATUS_URI_A.to_string()));
        assert!(all.contains(&STATUS_URI_B.to_string()));
        assert!(all.contains(&STATUS_URI_C.to_string()));

        let first = BoostRepository::get_boosts(0, 2).expect("should query");
        assert_eq!(first.len(), 2);

        let second = BoostRepository::get_boosts(2, 2).expect("should query");
        assert_eq!(second.len(), 1);
    }

    #[test]
    fn test_should_return_empty_when_no_boosts() {
        setup();

        let boosts = BoostRepository::get_boosts(0, 10).expect("should query");
        assert!(boosts.is_empty());
    }

    #[test]
    fn test_should_find_by_original_uri() {
        setup();
        seed_status(10);
        BoostRepository::boost_status(100, 10, STATUS_URI_A).expect("should insert boost");

        let found = BoostRepository::find_by_original_uri(STATUS_URI_A)
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
        let found = BoostRepository::find_by_original_uri(STATUS_URI_A).expect("should query");
        assert!(found.is_none());
    }

    #[test]
    fn test_should_delete_boost_with_wrapper_in_one_tx() {
        setup();

        BoostRepository::insert_boost_with_wrapper(
            77,
            STATUS_URI_A,
            "boosted",
            crate::schema::Visibility::from(did::common::Visibility::Public),
            None,
            false,
            1_000,
        )
        .expect("insert");

        BoostRepository::delete_boost_with_wrapper(77).expect("delete");

        assert!(!BoostRepository::is_boosted(STATUS_URI_A).expect("query"));
        assert!(
            crate::domain::status::StatusRepository::find_by_id(77)
                .expect("query")
                .is_none(),
            "wrapper status row removed"
        );
    }

    #[test]
    fn test_should_insert_boost_with_wrapper_in_one_tx() {
        setup();

        BoostRepository::insert_boost_with_wrapper(
            42,
            STATUS_URI_A,
            "boosted text",
            crate::schema::Visibility::from(did::common::Visibility::Public),
            Some("cw"),
            true,
            1_000_000,
        )
        .expect("insert");

        // Wrapper Status row
        let wrapper = crate::domain::status::StatusRepository::find_by_id(42)
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
        let boost = BoostRepository::find_by_original_uri(STATUS_URI_A)
            .expect("query")
            .expect("boost exists");
        assert_eq!(boost.id.expect("id").0, 42);
        assert_eq!(
            boost.original_status_uri.expect("uri").0,
            STATUS_URI_A.to_string()
        );

        // Idempotency check picks it up
        assert!(BoostRepository::is_boosted(STATUS_URI_A).expect("query"));
    }
}
