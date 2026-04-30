use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::{Database, DeleteBehavior, Filter, Query, TableSchema};

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{Liked, LikedInsertRequest, Schema};

pub struct LikedRepository;

impl LikedRepository {
    /// Inserts a liked status into the database.
    pub fn like_status(status_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT
            .with(|ctx| {
                let db = WasmDbmsDatabase::oneshot(ctx, Schema);

                db.insert::<Liked>(LikedInsertRequest {
                    status_uri: status_uri.into(),
                    created_at: ic_utils::now().into(),
                })
            })
            .map_err(CanisterError::from)
    }

    /// Deletes a liked status from the database.
    pub fn unlike_status(status_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT
            .with(|ctx| {
                let db = WasmDbmsDatabase::oneshot(ctx, Schema);

                db.delete::<Liked>(
                    DeleteBehavior::Cascade,
                    Some(Filter::eq(Liked::primary_key(), status_uri.into())),
                )
            })
            .map(|_| ())
            .map_err(CanisterError::from)
    }

    /// Checks if a status is liked by the user.
    pub fn is_liked(status_uri: &str) -> CanisterResult<bool> {
        DBMS_CONTEXT
            .with(|ctx| {
                let db = WasmDbmsDatabase::oneshot(ctx, Schema);

                db.select::<Liked>(
                    Query::builder()
                        .and_where(Filter::eq(Liked::primary_key(), status_uri.into()))
                        .limit(1)
                        .build(),
                )
            })
            .map(|count| !count.is_empty())
            .map_err(CanisterError::from)
    }

    pub fn get_liked(offset: usize, limit: usize) -> CanisterResult<Vec<String>> {
        DBMS_CONTEXT
            .with(|ctx| {
                let db = WasmDbmsDatabase::oneshot(ctx, Schema);

                db.select::<Liked>(Query::builder().all().offset(offset).limit(limit).build())
            })
            .map(|records| {
                records
                    .into_iter()
                    .map(|record| record.status_uri.unwrap_or_default().0)
                    .collect()
            })
            .map_err(CanisterError::from)
    }
}
