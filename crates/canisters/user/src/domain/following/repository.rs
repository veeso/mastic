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

    fn record_to_following(record: FollowingRecord) -> Following {
        Following {
            actor_uri: record.actor_uri.expect("must have field"),
            status: record.status.expect("must have field"),
            created_at: record.created_at.expect("must have field"),
        }
    }
}
