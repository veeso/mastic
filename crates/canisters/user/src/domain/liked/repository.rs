//! Repository for the `liked` table.

use ic_dbms_canister::prelude::{DBMS_CONTEXT, IcAccessControlList, IcMemoryProvider};
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms::prelude::DbmsContext;
use wasm_dbms_api::prelude::{Database, DeleteBehavior, Filter, Query, TableSchema, TransactionId};

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{Liked, LikedInsertRequest, Schema};

pub struct LikedRepository {
    tx: Option<TransactionId>,
}

impl LikedRepository {
    pub const fn oneshot() -> Self {
        Self { tx: None }
    }

    // Reserved for future cross-repo atomic flows that need to splice liked
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

    /// Inserts a liked status into the database.
    pub fn like_status(&self, status_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT
            .with(|ctx| {
                self.db(ctx).insert::<Liked>(LikedInsertRequest {
                    status_uri: status_uri.into(),
                    created_at: ic_utils::now().into(),
                })
            })
            .map_err(CanisterError::from)
    }

    /// Deletes a liked status from the database.
    pub fn unlike_status(&self, status_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT
            .with(|ctx| {
                self.db(ctx).delete::<Liked>(
                    DeleteBehavior::Cascade,
                    Some(Filter::eq(Liked::primary_key(), status_uri.into())),
                )
            })
            .map(|_| ())
            .map_err(CanisterError::from)
    }

    /// Checks if a status is liked by the user.
    pub fn is_liked(&self, status_uri: &str) -> CanisterResult<bool> {
        DBMS_CONTEXT
            .with(|ctx| {
                self.db(ctx).select::<Liked>(
                    Query::builder()
                        .and_where(Filter::eq(Liked::primary_key(), status_uri.into()))
                        .limit(1)
                        .build(),
                )
            })
            .map(|count| !count.is_empty())
            .map_err(CanisterError::from)
    }

    pub fn get_liked(&self, offset: usize, limit: usize) -> CanisterResult<Vec<String>> {
        DBMS_CONTEXT
            .with(|ctx| {
                self.db(ctx)
                    .select::<Liked>(Query::builder().all().offset(offset).limit(limit).build())
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
