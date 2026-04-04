//! Follower repository.

use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::*;

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{Follower, FollowerInsertRequest, FollowerRecord, Schema};

/// Interface to access [`Follower`]s data.
pub struct FollowerRepository;

impl FollowerRepository {
    /// Insert a new follower with the given actor URI.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "will be used by receive_activity handler")
    )]
    pub fn insert(actor_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            db.insert::<Follower>(FollowerInsertRequest {
                actor_uri: actor_uri.into(),
                created_at: ic_utils::now().into(),
            })
            .map_err(CanisterError::from)
        })
    }

    /// Get the list of [`Follower`]s of the user.
    pub fn get_followers() -> CanisterResult<Vec<Follower>> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            db.select::<Follower>(Query::builder().all().build())
                .map(|records| records.into_iter().map(Self::record_to_follower).collect())
                .map_err(CanisterError::from)
        })
    }

    /// Get the list of [`Follower`]s of the user.
    pub fn get_paginated(offset: usize, limit: usize) -> CanisterResult<Vec<Follower>> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            db.select::<Follower>(Query::builder().all().offset(offset).limit(limit).build())
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

        FollowerRepository::insert("https://mastic.social/users/alice").expect("should insert");

        let followers = FollowerRepository::get_followers().expect("should query");
        assert_eq!(followers.len(), 1);
        assert_eq!(
            followers[0].actor_uri.0,
            "https://mastic.social/users/alice"
        );
    }
}
