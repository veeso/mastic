//! Management canister adapter trait and implementations.
//!
//! Abstracts IC management canister calls (`create_canister`, `install_code`)
//! behind a trait so that domain logic can be unit-tested without a running replica.

#[cfg(not(target_family = "wasm"))]
pub mod mock;

use std::future::Future;

use candid::Principal;
use ic_management_canister_types::CanisterSettings;

/// Abstraction over the IC management canister API.
///
/// The two operations exposed — canister creation and WASM installation — are
/// the only management-canister interactions required by the sign-up state
/// machine today. Extend this trait if additional calls become necessary.
pub trait ManagementCanister: Send + Sync + Sized {
    /// Creates a new canister with the given optional settings.
    ///
    /// Returns the [`Principal`] of the newly created canister on success.
    fn create_canister(
        &self,
        settings: Option<CanisterSettings>,
    ) -> impl Future<Output = Result<Principal, ManagementCanisterError>>;

    /// Installs WASM code on an existing canister.
    fn install_code(
        &self,
        canister_id: Principal,
        wasm_module: &[u8],
        arg: Vec<u8>,
    ) -> impl Future<Output = Result<(), ManagementCanisterError>>;

    /// Returns the current canister version.
    fn canister_version(&self) -> u64;

    /// Returns the principal of the current canister.
    fn canister_self(&self) -> Principal;
}

/// Errors returned by [`ManagementCanister`] operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ManagementCanisterError {
    /// The inter-canister call failed.
    #[error("inter-canister call failed: {0}")]
    #[cfg(any(target_family = "wasm", test))]
    CallFailed(String),
    /// Failed to decode the response.
    #[cfg(target_family = "wasm")]
    #[error("failed to decode response: {0}")]
    DecodeFailed(String),
}

/// Production implementation that delegates to `ic_cdk` calls.
#[cfg(target_family = "wasm")]
pub struct IcManagementCanisterClient;

#[cfg(target_family = "wasm")]
impl ManagementCanister for IcManagementCanisterClient {
    async fn create_canister(
        &self,
        settings: Option<CanisterSettings>,
    ) -> Result<Principal, ManagementCanisterError> {
        let canister_version = self.canister_version();
        let request = ic_management_canister_types::CreateCanisterArgs {
            sender_canister_version: Some(canister_version),
            settings,
        };

        let response =
            ic_cdk::call::Call::bounded_wait(Principal::management_canister(), "create_canister")
                .with_arg(request)
                .await
                .map_err(|e| ManagementCanisterError::CallFailed(format!("{e:?}")))?;

        let response =
            candid::decode_one::<ic_management_canister_types::CreateCanisterResult>(&response)
                .map_err(|e| ManagementCanisterError::DecodeFailed(e.to_string()))?;

        Ok(response.canister_id)
    }

    async fn install_code(
        &self,
        canister_id: Principal,
        wasm_module: &[u8],
        arg: Vec<u8>,
    ) -> Result<(), ManagementCanisterError> {
        let canister_version = self.canister_version();
        let install_args = ic_management_canister_types::InstallCodeArgs {
            mode: ic_management_canister_types::CanisterInstallMode::Install,
            canister_id,
            wasm_module: wasm_module.to_vec(),
            arg,
            sender_canister_version: Some(canister_version),
        };

        ic_cdk::call::Call::bounded_wait(Principal::management_canister(), "install_code")
            .with_arg(install_args)
            .await
            .map_err(|e| ManagementCanisterError::CallFailed(format!("{e:?}")))?;

        Ok(())
    }

    fn canister_version(&self) -> u64 {
        ic_cdk::api::canister_version()
    }

    fn canister_self(&self) -> Principal {
        ic_cdk::api::canister_self()
    }
}
