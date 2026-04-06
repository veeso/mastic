//! Directory canister client adapter.

#[cfg(test)]
pub mod mock;

use candid::Principal;

use crate::error::CanisterResult;

/// Resolve a principal to its handle via the Directory Canister.
///
/// Returns `Ok(Some(handle))` if the principal is registered,
/// `Ok(None)` if the principal is not registered,
/// or an error if the call itself failed.
///
/// Dispatches to the mock client in tests and to the real
/// inter-canister client on wasm targets.
pub async fn resolve_handle(_principal: Principal) -> CanisterResult<Option<String>> {
    #[cfg(test)]
    {
        mock::DirectoryCanisterMockClient
            .resolve_handle(_principal)
            .await
            .map_err(crate::error::CanisterError::from)
    }
    #[cfg(target_family = "wasm")]
    {
        let directory_canister = crate::settings::get_directory_canister()?;
        IcDirectoryCanisterClient::from(directory_canister)
            .resolve_handle(_principal)
            .await
            .map_err(crate::error::CanisterError::from)
    }
    #[cfg(not(any(target_family = "wasm", test)))]
    {
        panic!("resolve_handle is not implemented for non-wasm, non-test targets");
    }
}

/// Abstraction over the directory canister API.
#[cfg(any(target_family = "wasm", test))]
pub trait DirectoryCanister: Send + Sync + Sized {
    /// Resolve a principal to its handle. Returns `Ok(None)` if not registered.
    fn resolve_handle(
        &self,
        principal: Principal,
    ) -> impl Future<Output = Result<Option<String>, DirectoryCanisterClientError>>;
}

/// Errors returned by [`DirectoryCanister`] operations.
#[derive(Debug, Clone, thiserror::Error)]
#[cfg(any(target_family = "wasm", test))]
pub enum DirectoryCanisterClientError {
    /// The inter-canister call failed.
    #[error("inter-canister call failed: {0}")]
    #[allow(dead_code, reason = "constructed only in wasm builds")]
    CallFailed(String),
    /// The response could not be decoded.
    #[error("decode failed: {0}")]
    #[allow(dead_code, reason = "constructed only in wasm builds")]
    DecodeFailed(String),
}

/// Production implementation that delegates to `ic_cdk` calls.
#[cfg(target_family = "wasm")]
pub struct IcDirectoryCanisterClient {
    canister_id: candid::Principal,
}

#[cfg(target_family = "wasm")]
impl From<candid::Principal> for IcDirectoryCanisterClient {
    fn from(canister_id: candid::Principal) -> Self {
        Self { canister_id }
    }
}

#[cfg(target_family = "wasm")]
impl DirectoryCanister for IcDirectoryCanisterClient {
    async fn resolve_handle(
        &self,
        principal: Principal,
    ) -> Result<Option<String>, DirectoryCanisterClientError> {
        use did::directory::{GetUserArgs, GetUserResponse};

        ic_utils::log!(
            "IcDirectoryCanisterClient::resolve_handle: resolving principal {principal}"
        );

        let args = GetUserArgs::Principal(principal);

        let raw = ic_cdk::call::Call::bounded_wait(self.canister_id, "get_user")
            .with_arg(args)
            .await
            .map_err(|e| {
                ic_utils::log!("DirectoryCanisterClientError::resolve_handle: call failed: {e:?}");
                DirectoryCanisterClientError::CallFailed(format!("{e:?}"))
            })?;

        let response = candid::decode_one::<GetUserResponse>(&raw).map_err(|e| {
            ic_utils::log!("DirectoryCanisterClientError::resolve_handle: decode failed: {e}");
            DirectoryCanisterClientError::DecodeFailed(e.to_string())
        })?;

        match response {
            GetUserResponse::Ok(user) => {
                ic_utils::log!(
                    "IcDirectoryCanisterClient::resolve_handle: resolved to {}",
                    user.handle
                );
                Ok(Some(user.handle))
            }
            GetUserResponse::Err(did::directory::GetUserError::NotFound) => {
                ic_utils::log!(
                    "IcDirectoryCanisterClient::resolve_handle: principal not registered"
                );
                Ok(None)
            }
            GetUserResponse::Err(e) => {
                ic_utils::log!("IcDirectoryCanisterClient::resolve_handle: directory error: {e:?}");
                Err(DirectoryCanisterClientError::CallFailed(format!("{e:?}")))
            }
        }
    }
}
