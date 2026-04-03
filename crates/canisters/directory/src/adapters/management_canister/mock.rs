//! Mock implementation of [`ManagementCanister`] for unit tests.

use candid::Principal;
use ic_management_canister_types::CanisterSettings;

use super::{ManagementCanister, ManagementCanisterError};

/// A test-only [`ManagementCanister`] that always succeeds.
#[derive(Debug)]
pub struct MockManagementCanisterClient {
    /// The canister ID returned by `create_canister`.
    pub created_canister_id: Principal,
    /// Value returned by `canister_self`.
    pub canister_self: Principal,
}

impl ManagementCanister for MockManagementCanisterClient {
    async fn create_canister(
        &self,
        _settings: Option<CanisterSettings>,
        _cycles: u128,
    ) -> Result<Principal, ManagementCanisterError> {
        Ok(self.created_canister_id)
    }

    async fn install_code(
        &self,
        _canister_id: Principal,
        _wasm_module: &[u8],
        _arg: Vec<u8>,
    ) -> Result<(), ManagementCanisterError> {
        let _version = self.canister_version();
        Ok(())
    }

    fn canister_version(&self) -> u64 {
        0
    }

    fn canister_self(&self) -> Principal {
        self.canister_self
    }
}
