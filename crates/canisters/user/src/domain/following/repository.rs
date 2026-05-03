//! Following repository for managing follow relationships.

use db_utils::repository::Repository;
use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms_api::prelude::*;

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{FollowStatus, Following, FollowingInsertRequest, FollowingRecord, Schema};

/// Interface to access [`Following`] data.
pub struct FollowingRepository {
    tx: Option<TransactionId>,
}

impl FollowingRepository {
    /// Insert a new pending follow entry for the given actor URI.
    pub fn insert_pending(&self, actor_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx)
                .insert::<Following>(FollowingInsertRequest {
                    actor_uri: actor_uri.into(),
                    status: FollowStatus::Pending,
                    created_at: ic_utils::now().into(),
                })
                .map_err(CanisterError::from)
        })
    }

    /// Find a following entry by actor URI.
    pub fn find_by_actor_uri(&self, actor_uri: &str) -> CanisterResult<Option<Following>> {
        DBMS_CONTEXT.with(|ctx| {
            let records = self
                .db(ctx)
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

    /// Get the list of accepted [`Following`]s of the user.
    ///
    /// Only returns entries with [`FollowStatus::Accepted`].
    pub fn get_accepted_following(
        &self,
        offset: usize,
        limit: usize,
    ) -> CanisterResult<Vec<Following>> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx)
                .select::<Following>(
                    Query::builder()
                        .all()
                        .and_where(Filter::eq("status", Value::from(FollowStatus::Accepted)))
                        .offset(offset)
                        .limit(limit)
                        .build(),
                )
                .map(|records| records.into_iter().map(Self::record_to_following).collect())
                .map_err(CanisterError::from)
        })
    }

    /// Delete a following entry by actor URI.
    ///
    /// Returns `true` if an entry was deleted, `false` if no entry was found with the given actor URI.
    pub fn delete_by_actor_uri(&self, actor_uri: &str) -> CanisterResult<bool> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx)
                .delete::<Following>(
                    DeleteBehavior::Restrict,
                    Some(Filter::eq("actor_uri", Value::from(actor_uri.to_string()))),
                )
                .map(|entries| entries > 0)
                .map_err(CanisterError::from)
        })
    }

    /// Update the follow status for a given actor URI.
    ///
    /// Implemented as delete + re-insert because wasm-dbms does not support
    /// in-place updates. Callers must wrap this call in a `Transaction::run`
    /// to preserve atomicity across the underlying delete + insert.
    pub fn update_status(&self, actor_uri: &str, new_status: FollowStatus) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            let db = self.db(ctx);

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

impl Repository for FollowingRepository {
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

    use db_utils::transaction::Transaction;

    use super::*;
    use crate::test_utils::setup;

    #[test]
    fn test_should_delete_following_by_actor_uri() {
        setup();

        FollowingRepository::oneshot()
            .insert_pending("https://mastic.social/users/alice")
            .expect("should insert");

        FollowingRepository::oneshot()
            .delete_by_actor_uri("https://mastic.social/users/alice")
            .expect("should delete");

        let found = FollowingRepository::oneshot()
            .find_by_actor_uri("https://mastic.social/users/alice")
            .expect("should query");
        assert!(found.is_none());
    }

    #[test]
    fn test_should_update_status_pending_to_accepted() {
        setup();

        FollowingRepository::oneshot()
            .insert_pending("https://mastic.social/users/alice")
            .expect("should insert");

        Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
            FollowingRepository::with_transaction(tx)
                .update_status("https://mastic.social/users/alice", FollowStatus::Accepted)
        })
        .expect("should update");

        let entry = FollowingRepository::oneshot()
            .find_by_actor_uri("https://mastic.social/users/alice")
            .expect("should query")
            .expect("should find entry");
        assert_eq!(entry.status, FollowStatus::Accepted);
    }

    #[test]
    fn test_update_status_should_fail_for_missing_entry() {
        setup();

        let result = Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
            FollowingRepository::with_transaction(tx)
                .update_status("https://mastic.social/users/nobody", FollowStatus::Accepted)
        });
        assert!(result.is_err());
    }
}
