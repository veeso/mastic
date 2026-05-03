//! User profile repository

use candid::Principal;
use db_utils::field_update::field_update_to_nullable;
use db_utils::repository::Repository;
use db_utils::settings::SettingsError;
use did::common::FieldUpdate;
use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms_api::prelude::*;

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{Profile, ProfileInsertRequest, ProfileRecord, ProfileUpdateRequest, Schema};

pub struct ProfileRepository {
    tx: Option<TransactionId>,
}

impl ProfileRepository {
    /// Get the profile of the current user.
    pub fn get_profile(&self) -> CanisterResult<Profile> {
        let row = DBMS_CONTEXT.with(|ctx| {
            let record = self
                .db(ctx)
                .select::<Profile>(Query::builder().all().limit(1).build())?;

            Ok::<Option<ProfileRecord>, CanisterError>(record.into_iter().next())
        })?;

        if let Some(row) = row {
            Ok(Self::record_to_profile(row))
        } else {
            Err(CanisterError::Settings(SettingsError::Uninitialized))
        }
    }

    /// Create a new profile for the given principal and handle.
    pub fn create_profile(&self, principal: Principal, handle: &str) -> CanisterResult<()> {
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
            self.db(ctx).insert::<Profile>(insert_request)?;

            Ok(())
        })
    }

    /// Update the user's profile with the given display name and bio.
    ///
    /// Returns `Ok(true)` when a row was written, `Ok(false)` when every
    /// field is [`FieldUpdate::Leave`] (no-op). The caller can use the
    /// boolean to decide whether to fan out an activity.
    pub fn update_profile(
        &self,
        bio: FieldUpdate<String>,
        display_name: FieldUpdate<String>,
    ) -> CanisterResult<bool> {
        if [&display_name, &bio]
            .iter()
            .all(|field| matches!(field, FieldUpdate::Leave))
        {
            ic_utils::log!("No profile fields to update, skipping database update");
            return Ok(false);
        }
        let patch = ProfileUpdateRequest {
            display_name: field_update_to_nullable(display_name.map(|v| v.into())),
            bio: field_update_to_nullable(bio.map(|v| v.into())),
            updated_at: Some(ic_utils::now().into()),
            ..Default::default()
        };

        DBMS_CONTEXT.with(|ctx| self.db(ctx).update::<Profile>(patch))?;

        Ok(true)
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

impl Repository for ProfileRepository {
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
