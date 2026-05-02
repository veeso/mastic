//! Follower repository.

use ic_dbms_canister::prelude::{DBMS_CONTEXT, IcAccessControlList, IcMemoryProvider};
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms::prelude::DbmsContext;
use wasm_dbms_api::prelude::*;

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{Follower, FollowerInsertRequest, FollowerRecord, Schema};

/// Interface to access [`Follower`]s data.
pub struct FollowerRepository {
    tx: Option<TransactionId>,
}

impl FollowerRepository {
    pub const fn oneshot() -> Self {
        Self { tx: None }
    }

    // Reserved for future cross-repo atomic flows that need to splice follower
    // reads/writes into an externally-driven transaction. Not yet wired up.
    #[allow(dead_code)]
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

    /// Insert a new follower with the given actor URI.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "will be used by receive_activity handler")
    )]
    pub fn insert(&self, actor_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx)
                .insert::<Follower>(FollowerInsertRequest {
                    actor_uri: actor_uri.into(),
                    created_at: ic_utils::now().into(),
                })
                .map_err(CanisterError::from)
        })
    }

    /// Get the list of [`Follower`]s of the user.
    pub fn get_followers(&self) -> CanisterResult<Vec<Follower>> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx)
                .select::<Follower>(Query::builder().all().build())
                .map(|records| records.into_iter().map(Self::record_to_follower).collect())
                .map_err(CanisterError::from)
        })
    }

    /// Checks if the given actor URI is a [`Follower`] of the user.
    pub fn is_follower(&self, actor_uri: &str) -> CanisterResult<bool> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx)
                .select::<Follower>(
                    Query::builder()
                        .all()
                        .and_where(Filter::eq("actor_uri", actor_uri.into()))
                        .limit(1)
                        .build(),
                )
                .map(|records| !records.is_empty())
                .map_err(CanisterError::from)
        })
    }

    /// Delete a follower entry by actor URI.
    ///
    /// Returns `true` if an entry was deleted, `false` if no entry was found
    /// with the given actor URI.
    pub fn delete_by_actor_uri(&self, actor_uri: &str) -> CanisterResult<bool> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx)
                .delete::<Follower>(
                    DeleteBehavior::Restrict,
                    Some(Filter::eq("actor_uri", Value::from(actor_uri.to_string()))),
                )
                .map(|entries| entries > 0)
                .map_err(CanisterError::from)
        })
    }

    /// Get the list of [`Follower`]s of the user.
    pub fn get_paginated(&self, offset: usize, limit: usize) -> CanisterResult<Vec<Follower>> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx)
                .select::<Follower>(Query::builder().all().offset(offset).limit(limit).build())
                .map(|records| records.into_iter().map(Self::record_to_follower).collect())
                .map_err(CanisterError::from)
        })
    }

    fn record_to_follower(record: FollowerRecord) -> Follower {
        Follower {
            actor_uri: record.actor_uri.expect("must have field"),
            created_at: record.created_at.expect("must have field"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::setup;

    #[test]
    fn test_should_insert_follower() {
        setup();

        FollowerRepository::oneshot()
            .insert("https://mastic.social/users/alice")
            .expect("should insert");

        let followers = FollowerRepository::oneshot()
            .get_followers()
            .expect("should query");
        assert_eq!(followers.len(), 1);
        assert_eq!(
            followers[0].actor_uri.0,
            "https://mastic.social/users/alice"
        );
    }
}
