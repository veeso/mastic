//! Repository for the `blocks` table.

use std::collections::HashSet;

use ic_dbms_canister::prelude::{DBMS_CONTEXT, IcAccessControlList, IcMemoryProvider};
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms::prelude::DbmsContext;
use wasm_dbms_api::prelude::{Database, Query, TransactionId};

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{Block, BlockRecord, Schema};

pub struct BlockRepository {
    tx: Option<TransactionId>,
}

impl BlockRepository {
    pub const fn oneshot() -> Self {
        Self { tx: None }
    }

    // Reserved for future cross-repo atomic flows that need to splice block
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

    /// Return the set of actor URIs blocked by the owner.
    pub fn list_blocked_uris(&self) -> CanisterResult<HashSet<String>> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx)
                .select::<Block>(Query::builder().all().build())
                .map(|records| {
                    records
                        .into_iter()
                        .filter_map(Self::record_to_uri)
                        .collect()
                })
                .map_err(CanisterError::from)
        })
    }

    fn record_to_uri(record: BlockRecord) -> Option<String> {
        record.actor_uri.map(|t| t.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::BlockInsertRequest;
    use crate::test_utils::setup;

    fn insert_block(actor_uri: &str) {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Block>(BlockInsertRequest {
                actor_uri: actor_uri.into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert block");
        });
    }

    #[test]
    fn test_list_blocked_uris_empty() {
        setup();
        let blocked = BlockRepository::oneshot()
            .list_blocked_uris()
            .expect("should query");
        assert!(blocked.is_empty());
    }

    #[test]
    fn test_list_blocked_uris_returns_inserted() {
        setup();
        insert_block("https://remote.example/users/eve");
        insert_block("https://remote.example/users/mallory");

        let blocked = BlockRepository::oneshot()
            .list_blocked_uris()
            .expect("should query");
        assert_eq!(blocked.len(), 2);
        assert!(blocked.contains("https://remote.example/users/eve"));
        assert!(blocked.contains("https://remote.example/users/mallory"));
    }
}
