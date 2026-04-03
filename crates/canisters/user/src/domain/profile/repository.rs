//! User profile repository

use candid::Principal;
use db_utils::settings::SettingsError;
use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::*;

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{Profile, ProfileInsertRequest, ProfileRecord, Schema};

pub struct ProfileRepository;

impl ProfileRepository {
    /// Get the profile of the current user.
    pub fn get_profile() -> CanisterResult<Profile> {
        let row = DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            let record = db.select::<Profile>(Query::builder().all().limit(1).build())?;

            Ok::<Option<ProfileRecord>, CanisterError>(record.into_iter().next())
        })?;

        if let Some(row) = row {
            Ok(Self::record_to_profile(row))
        } else {
            Err(CanisterError::Settings(SettingsError::Uninitialized))
        }
    }

    /// Create a new profile for the given principal and handle.
    pub fn create_profile(principal: Principal, handle: &str) -> CanisterResult<()> {
        let insert_request = ProfileInsertRequest {
            principal: ic_dbms_canister::prelude::Principal(principal),
            handle: handle.into(),
            display_name: Nullable::Null,
            bio: Nullable::Null,
            avatar_data: Nullable::Null,
            header_data: Nullable::Null,
            created_at: ic_utils::now().into(),
            updated_at: ic_utils::now().into(),
        };

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            db.insert::<Profile>(insert_request)?;

            Ok(())
        })
    }

    fn record_to_profile(record: ProfileRecord) -> Profile {
        Profile {
            principal: record.principal.expect("principal cannot be missing"),
            handle: record.handle.expect("handle cannot be missing"),
            display_name: record.display_name.expect("display name cannot be missing"),
            bio: record.bio.expect("bio cannot be missing"),
            avatar_data: record.avatar_data.expect("avatar data cannot be missing"),
            header_data: record.header_data.expect("header data cannot be missing"),
            created_at: record.created_at.expect("created at cannot be missing"),
            updated_at: record.updated_at.expect("updated at cannot be missing"),
        }
    }
}
