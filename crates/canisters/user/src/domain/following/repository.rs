//! Following repository for managing follow relationships.

use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::*;

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{FollowStatus, Following, FollowingInsertRequest, FollowingRecord, Schema};

/// Interface to access [`Following`] data.
pub struct FollowingRepository;

impl FollowingRepository {
    /// Insert a new pending follow entry for the given actor URI.
    pub fn insert_pending(actor_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            db.insert::<Following>(FollowingInsertRequest {
                actor_uri: actor_uri.into(),
                status: FollowStatus::Pending,
                created_at: ic_utils::now().into(),
            })
            .map_err(CanisterError::from)
        })
    }

    /// Find a following entry by actor URI.
    pub fn find_by_actor_uri(actor_uri: &str) -> CanisterResult<Option<Following>> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            let records = db
                .select::<Following>(
                    Query::builder()
                        .all()
                        .and_where(Filter::eq("actor_uri", Value::from(actor_uri)))
                        .build(),
                )
                .map_err(CanisterError::from)?;

            Ok(records.into_iter().next().map(Self::record_to_following))
        })
    }

    /// Get the list of [`Following`]s of the user.
    pub fn get_paginated(offset: usize, limit: usize) -> CanisterResult<Vec<Following>> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            db.select::<Following>(Query::builder().all().offset(offset).limit(limit).build())
                .map(|records| records.into_iter().map(Self::record_to_following).collect())
                .map_err(CanisterError::from)
        })
    }

    /// Delete a following entry by actor URI.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "will be used by unfollow handler")
    )]
    pub fn delete_by_actor_uri(actor_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            db.delete::<Following>(
                DeleteBehavior::Restrict,
                Some(Filter::eq("actor_uri", Value::from(actor_uri.to_string()))),
            )
            .map(|_| ())
            .map_err(CanisterError::from)
        })
    }

    /// Update the follow status for a given actor URI.
    ///
    /// Implemented as delete + re-insert because wasm-dbms does not support
    /// in-place updates. Runs inside a transaction to maintain atomicity.
    pub fn update_status(actor_uri: &str, new_status: FollowStatus) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);

            // read the existing entry
            let records = db.select::<Following>(
                Query::builder()
                    .all()
                    .and_where(Filter::eq("actor_uri", Value::from(actor_uri)))
                    .build(),
            )?;

            let existing = records
                .into_iter()
                .next()
                .map(Self::record_to_following)
                .ok_or(CanisterError::Database(DbmsError::Query(
                    QueryError::RecordNotFound,
                )))?;

            // delete old entry
            db.delete::<Following>(
                DeleteBehavior::Restrict,
                Some(Filter::eq("actor_uri", Value::from(actor_uri.to_string()))),
            )?;

            // re-insert with new status, preserving created_at
            db.insert::<Following>(FollowingInsertRequest {
                actor_uri: existing.actor_uri,
                status: new_status,
                created_at: existing.created_at,
            })?;

            db.commit()?;

            Ok(())
        })
    }

    fn record_to_following(record: FollowingRecord) -> Following {
        Following {
            actor_uri: record.actor_uri.expect("must have field"),
            status: record.status.expect("must have field"),
            created_at: record.created_at.expect("must have field"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::setup;

    #[test]
    fn test_should_delete_following_by_actor_uri() {
        setup();

        FollowingRepository::insert_pending("https://mastic.social/users/alice")
            .expect("should insert");

        FollowingRepository::delete_by_actor_uri("https://mastic.social/users/alice")
            .expect("should delete");

        let found = FollowingRepository::find_by_actor_uri("https://mastic.social/users/alice")
            .expect("should query");
        assert!(found.is_none());
    }

    #[test]
    fn test_should_update_status_pending_to_accepted() {
        setup();

        FollowingRepository::insert_pending("https://mastic.social/users/alice")
            .expect("should insert");

        FollowingRepository::update_status(
            "https://mastic.social/users/alice",
            FollowStatus::Accepted,
        )
        .expect("should update");

        let entry = FollowingRepository::find_by_actor_uri("https://mastic.social/users/alice")
            .expect("should query")
            .expect("should find entry");
        assert_eq!(entry.status, FollowStatus::Accepted);
    }

    #[test]
    fn test_should_update_status_pending_to_rejected() {
        setup();

        FollowingRepository::insert_pending("https://mastic.social/users/alice")
            .expect("should insert");

        FollowingRepository::update_status(
            "https://mastic.social/users/alice",
            FollowStatus::Rejected,
        )
        .expect("should update");

        let entry = FollowingRepository::find_by_actor_uri("https://mastic.social/users/alice")
            .expect("should query")
            .expect("should find entry");
        assert_eq!(entry.status, FollowStatus::Rejected);
    }

    #[test]
    fn test_update_status_should_fail_for_missing_entry() {
        setup();

        let result = FollowingRepository::update_status(
            "https://mastic.social/users/nobody",
            FollowStatus::Accepted,
        );
        assert!(result.is_err());
    }
}
