//! Management canister adapter trait and implementations.
//!
//! Abstracts IC management canister calls (`create_canister`, `install_code`)
//! behind a trait so that domain logic can be unit-tested without a running replica.

#[cfg(not(target_family = "wasm"))]
pub mod mock;

use std::future::Future;

use candid::Principal;
use ic_management_canister_types::CanisterSettings;

#[cfg(target_family = "wasm")]
/// Minimum cycles to attach for canister creation to ensure it succeeds.
const CREATE_CANISTER_FEE: u128 = 500_000_000_000;

/// Abstraction over the IC management canister API.
///
/// The two operations exposed — canister creation and WASM installation — are
/// the only management-canister interactions required by the sign-up state
/// machine today. Extend this trait if additional calls become necessary.
pub trait ManagementCanister: Send + Sync + Sized {
    /// Creates a new canister with the given optional settings and amount of cycles.
    ///
    /// Returns the [`Principal`] of the newly created canister on success.
    fn create_canister(
        &self,
        settings: Option<CanisterSettings>,
        cycles: u128,
    ) -> impl Future<Output = Result<Principal, ManagementCanisterError>>;

    /// Installs WASM code on an existing canister.
    fn install_code(
        &self,
        canister_id: Principal,
        wasm_module: &[u8],
        arg: Vec<u8>,
    ) -> impl Future<Output = Result<(), ManagementCanisterError>>;

    /// Stops a running canister.
    ///
    /// Idempotent: stopping an already-stopped canister must succeed.
    fn stop_canister(
        &self,
        canister_id: Principal,
    ) -> impl Future<Output = Result<(), ManagementCanisterError>>;

    /// Deletes a stopped canister, reclaiming its cycles and canister ID.
    ///
    /// Idempotent: deleting a non-existent canister must succeed.
    fn delete_canister(
        &self,
        canister_id: Principal,
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
        cycles: u128,
    ) -> Result<Principal, ManagementCanisterError> {
        ic_utils::log!(
            "IcManagementCanisterClient::create_canister: sending create_canister request"
        );
        let canister_version = self.canister_version();
        let request = ic_management_canister_types::ProvisionalCreateCanisterWithCyclesArgs {
            sender_canister_version: Some(canister_version),
            settings,
            amount: Some(cycles.into()),
            specified_id: None,
        };

        let response = ic_cdk::call::Call::bounded_wait(
            Principal::management_canister(),
            "provisional_create_canister_with_cycles",
        )
        .with_arg(request)
        .with_cycles(CREATE_CANISTER_FEE)
        .await
        .map_err(|e| {
            ic_utils::log!("IcManagementCanisterClient::create_canister: call failed: {e:?}");
            ManagementCanisterError::CallFailed(format!("{e:?}"))
        })?;

        let response = candid::decode_one::<
            ic_management_canister_types::ProvisionalCreateCanisterWithCyclesResult,
        >(&response)
        .map_err(|e| {
            ic_utils::log!("IcManagementCanisterClient::create_canister: decode failed: {e}");
            ManagementCanisterError::DecodeFailed(e.to_string())
        })?;

        ic_utils::log!(
            "IcManagementCanisterClient::create_canister: created canister {}",
            response.canister_id
        );

        Ok(response.canister_id)
    }

    async fn install_code(
        &self,
        canister_id: Principal,
        wasm_module: &[u8],
        arg: Vec<u8>,
    ) -> Result<(), ManagementCanisterError> {
        ic_utils::log!(
            "IcManagementCanisterClient::install_code: installing code on canister {canister_id} (wasm size: {} bytes)",
            wasm_module.len()
        );
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
            .map_err(|e| {
                ic_utils::log!("IcManagementCanisterClient::install_code: call failed for canister {canister_id}: {e:?}");
                ManagementCanisterError::CallFailed(format!("{e:?}"))
            })?;

        ic_utils::log!(
            "IcManagementCanisterClient::install_code: code installed on canister {canister_id}"
        );

        Ok(())
    }

    async fn stop_canister(&self, canister_id: Principal) -> Result<(), ManagementCanisterError> {
        ic_utils::log!(
            "IcManagementCanisterClient::stop_canister: stopping canister {canister_id}"
        );
        let args = ic_management_canister_types::StopCanisterArgs { canister_id };

        match ic_cdk::call::Call::bounded_wait(Principal::management_canister(), "stop_canister")
            .with_arg(args)
            .await
        {
            Ok(_) => {
                ic_utils::log!(
                    "IcManagementCanisterClient::stop_canister: stopped canister {canister_id}"
                );
                Ok(())
            }
            Err(e) => {
                let msg = format!("{e:?}");
                if msg.contains("CanisterNotFound")
                    || msg.contains("Canister has already been stopped")
                {
                    ic_utils::log!(
                        "IcManagementCanisterClient::stop_canister: treating as success for {canister_id}: {msg}"
                    );
                    Ok(())
                } else {
                    ic_utils::log!(
                        "IcManagementCanisterClient::stop_canister: call failed for {canister_id}: {msg}"
                    );
                    Err(ManagementCanisterError::CallFailed(msg))
                }
            }
        }
    }

    async fn delete_canister(&self, canister_id: Principal) -> Result<(), ManagementCanisterError> {
        ic_utils::log!(
            "IcManagementCanisterClient::delete_canister: deleting canister {canister_id}"
        );
        let args = ic_management_canister_types::DeleteCanisterArgs { canister_id };

        match ic_cdk::call::Call::bounded_wait(Principal::management_canister(), "delete_canister")
            .with_arg(args)
            .await
        {
            Ok(_) => {
                ic_utils::log!(
                    "IcManagementCanisterClient::delete_canister: deleted canister {canister_id}"
                );
                Ok(())
            }
            Err(e) => {
                let msg = format!("{e:?}");
                if msg.contains("CanisterNotFound") {
                    ic_utils::log!(
                        "IcManagementCanisterClient::delete_canister: canister {canister_id} not found; treating as success"
                    );
                    Ok(())
                } else {
                    ic_utils::log!(
                        "IcManagementCanisterClient::delete_canister: call failed for {canister_id}: {msg}"
                    );
                    Err(ManagementCanisterError::CallFailed(msg))
                }
            }
        }
    }

    fn canister_version(&self) -> u64 {
        ic_cdk::api::canister_version()
    }

    fn canister_self(&self) -> Principal {
        ic_cdk::api::canister_self()
    }
}
