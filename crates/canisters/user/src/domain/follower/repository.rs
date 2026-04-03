//! Follower repository.

use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::*;

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{Follower, FollowerRecord, Schema};

/// Interface to access [`Follower`]s data.
pub struct FollowerRepository;

impl FollowerRepository {
    /// Get the list of [`Follower`]s of the user.
    pub fn get_followers() -> CanisterResult<Vec<Follower>> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            db.select::<Follower>(Query::builder().all().build())
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
