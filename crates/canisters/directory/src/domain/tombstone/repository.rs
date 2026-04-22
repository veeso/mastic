//! Tombstone repository

use candid::Principal;
use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::{Database as _, Filter, Query};

use crate::error::CanisterResult;
use crate::schema::{Schema, Tombstone, TombstoneInsertRequest, TombstoneUpdateRequest};

/// Time-to-live for tombstone records in seconds (e.g. 30 days)
const TOMBSTONE_TTL_SECONDS: u64 = 60 * 60 * 24 * 30;

/// Repository for managing tombstone records of deleted user profiles.
pub struct TombstoneRepository;

impl TombstoneRepository {
    /// Inserts a [`Tombstone`] record for the given user principal and handle.
    pub fn insert_or_update(user_principal: Principal, handle: String) -> CanisterResult<()> {
        ic_utils::log!(
            "TombstoneRepository::insert: inserting tombstone for user {user_principal} with handle {handle}"
        );

        if Self::is_tombstoned(&handle)? {
            ic_utils::log!(
                "TombstoneRepository::insert: existing tombstone found for handle {handle}, updating deleted_at timestamp"
            );

            let update_request = TombstoneUpdateRequest {
                deleted_at: Some(ic_utils::now().into()),
                where_clause: Some(Filter::eq("handle", handle.into())),
                ..Default::default()
            };

            DBMS_CONTEXT.with(|ctx| {
                let dbms = WasmDbmsDatabase::oneshot(ctx, Schema);
                dbms.update::<Tombstone>(update_request)
            })?;
        } else {
            ic_utils::log!(
                "TombstoneRepository::insert: no existing tombstone found for handle {handle}, inserting new record"
            );

            let insert = TombstoneInsertRequest {
                handle: handle.into(),
                principal: ic_dbms_canister::prelude::Principal(user_principal),
                deleted_at: ic_utils::now().into(),
            };

            DBMS_CONTEXT.with(|ctx| {
                let dbms = WasmDbmsDatabase::oneshot(ctx, Schema);
                dbms.insert::<Tombstone>(insert)
            })?;
        }

        Ok(())
    }

    /// Checks if a given handle is currently tombstoned (i.e. has a tombstone record with a `deleted_at` timestamp within the TTL).
    pub fn is_tombstoned(handle: &str) -> CanisterResult<bool> {
        ic_utils::log!(
            "TombstoneRepository::is_tombstoned: checking if handle {handle} is tombstoned"
        );
        let rows = DBMS_CONTEXT.with(|ctx| {
            let dbms = WasmDbmsDatabase::oneshot(ctx, Schema);
            dbms.select::<Tombstone>(
                Query::builder()
                    .field("deleted_at")
                    .limit(1)
                    .and_where(Filter::eq("handle", handle.into()))
                    .build(),
            )
        })?;

        // get `deleted_at` value from the first row if it exists, otherwise return false
        let Some(row) = rows.first() else {
            ic_utils::log!(
                "TombstoneRepository::is_tombstoned: no tombstone record found for handle {handle}"
            );
            return Ok(false);
        };

        let deleted_at = row.deleted_at.expect("must be set").0;

        Ok(ic_utils::now() - deleted_at < TOMBSTONE_TTL_SECONDS)
    }
}
