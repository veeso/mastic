use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::{Database, DeleteBehavior, Filter, Query};

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{Boost, BoostInsertRequest, Schema};

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
}
