//! Federation canister client adapter.

#[cfg(test)]
pub mod mock;

use did::federation::SendActivityArgs;

use crate::error::CanisterResult;

/// Send an activity to the Federation Canister.
///
/// Dispatches to the mock client in tests and to the real
/// inter-canister client on wasm targets.
pub async fn send_activity(_args: SendActivityArgs) -> CanisterResult<()> {
    #[cfg(test)]
    {
        mock::FederationCanisterMockClient
            .send_activity(_args)
            .await
            .map_err(crate::error::CanisterError::from)
    }
    #[cfg(target_family = "wasm")]
    {
        let federation_canister = crate::settings::get_federation_canister()?;
        IcFederationCanisterClient::from(federation_canister)
            .send_activity(_args)
            .await
            .map_err(crate::error::CanisterError::from)
    }
    #[cfg(not(any(target_family = "wasm", test)))]
    {
        panic!("send_activity is not implemented for non-wasm, non-test targets");
    }
}

/// Abstraction over the federation canister API.
#[cfg(any(target_family = "wasm", test))]
pub trait FederationCanister: Send + Sync + Sized {
    /// call the `send_activity` method of the federation canister with the given arguments.
    fn send_activity(
        &self,
        args: SendActivityArgs,
    ) -> impl Future<Output = Result<(), FederationCanisterClientError>>;
}

/// Errors returned by [`FederationCanister`] operations.
#[derive(Debug, Clone, thiserror::Error)]
#[cfg(any(target_family = "wasm", test))]
pub enum FederationCanisterClientError {
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
pub struct IcFederationCanisterClient {
    canister_id: candid::Principal,
}

#[cfg(target_family = "wasm")]
impl From<candid::Principal> for IcFederationCanisterClient {
    fn from(canister_id: candid::Principal) -> Self {
        Self { canister_id }
    }
}

#[cfg(target_family = "wasm")]
impl FederationCanister for IcFederationCanisterClient {
    async fn send_activity(
        &self,
        args: SendActivityArgs,
    ) -> Result<(), FederationCanisterClientError> {
        use did::federation::{SendActivityResponse, SendActivityResult};

        ic_utils::log!("IcFederationCanisterClient::send_activity: sending send_activity request");

        let raw = ic_cdk::call::Call::bounded_wait(self.canister_id, "send_activity")
            .with_arg(args)
            .await
            .map_err(|e| {
                ic_utils::log!("FederationCanisterClientError::send_activity: call failed: {e:?}");
                FederationCanisterClientError::CallFailed(format!("{e:?}"))
            })?;

        let response = candid::decode_one::<SendActivityResponse>(&raw).map_err(|e| {
            ic_utils::log!("FederationCanisterClientError::send_activity: decode failed: {e}");
            FederationCanisterClientError::DecodeFailed(e.to_string())
        })?;

        let results: Vec<SendActivityResult> = match response {
            SendActivityResponse::One(result) => vec![result],
            SendActivityResponse::Batch(results) => results,
        };

        for result in results {
            if let SendActivityResult::Err(e) = result {
                ic_utils::log!(
                    "IcFederationCanisterClient::send_activity: federation error: {e:?}"
                );
                return Err(FederationCanisterClientError::CallFailed(format!("{e:?}")));
            }
        }

        ic_utils::log!("IcFederationCanisterClient::send_activity: sent activity successfully");
        Ok(())
    }
}
