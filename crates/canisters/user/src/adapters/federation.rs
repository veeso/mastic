//! Federation canister client adapter.

#[cfg(test)]
pub mod mock;

#[cfg(any(target_family = "wasm", test))]
use did::federation::SendActivityArgs;
#[cfg(target_family = "wasm")]
use did::federation::SendActivityResponse;

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

        match response {
            SendActivityResponse::Ok => {
                ic_utils::log!(
                    "IcFederationCanisterClient::send_activity: sent activity successfully"
                );
                Ok(())
            }
            SendActivityResponse::Err(e) => {
                ic_utils::log!(
                    "IcFederationCanisterClient::send_activity: federation error: {e:?}"
                );
                Err(FederationCanisterClientError::CallFailed(format!("{e:?}")))
            }
        }
    }
}
