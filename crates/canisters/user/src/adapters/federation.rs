//! Federation canister client adapter.

#[cfg(test)]
pub mod mock;

use did::federation::{FetchStatusArgs, FetchStatusResponse, SendActivityArgs};

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

/// Fetch a status from the Federation Canister.
///
/// Dispatches to the mock client in tests and to the real
/// inter-canister client on wasm targets.
pub async fn fetch_status(_args: FetchStatusArgs) -> CanisterResult<FetchStatusResponse> {
    #[cfg(test)]
    {
        mock::FederationCanisterMockClient
            .fetch_status(_args)
            .await
            .map_err(crate::error::CanisterError::from)
    }
    #[cfg(target_family = "wasm")]
    {
        let federation_canister = crate::settings::get_federation_canister()?;
        IcFederationCanisterClient::from(federation_canister)
            .fetch_status(_args)
            .await
            .map_err(crate::error::CanisterError::from)
    }
    #[cfg(not(any(target_family = "wasm", test)))]
    {
        panic!("fetch_status is not implemented for non-wasm, non-test targets");
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

    /// call the `fetch_status` method of the federation canister with the given arguments.
    fn fetch_status(
        &self,
        args: FetchStatusArgs,
    ) -> impl Future<Output = Result<FetchStatusResponse, FederationCanisterClientError>>;
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

    async fn fetch_status(
        &self,
        args: FetchStatusArgs,
    ) -> Result<FetchStatusResponse, FederationCanisterClientError> {
        ic_utils::log!("IcFederationCanisterClient::fetch_status: sending fetch_status request");

        let raw = ic_cdk::call::Call::bounded_wait(self.canister_id, "fetch_status")
            .with_arg(args)
            .await
            .map_err(|e| {
                ic_utils::log!("IcFederationCanisterClient::fetch_status: call failed: {e:?}");
                FederationCanisterClientError::CallFailed(format!("{e:?}"))
            })?;

        let response = candid::decode_one::<FetchStatusResponse>(&raw).map_err(|e| {
            ic_utils::log!("IcFederationCanisterClient::fetch_status: decode failed: {e}");
            FederationCanisterClientError::DecodeFailed(e.to_string())
        })?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use did::common::{Status, Visibility};
    use did::federation::{FetchStatusArgs, FetchStatusResponse};

    use super::fetch_status;
    use crate::adapters::federation::mock::push_fetch_status_response;
    use crate::test_utils::setup;

    fn fixture_status() -> Status {
        Status {
            id: 1,
            content: "hi".into(),
            author: "https://x/users/a".into(),
            created_at: 0,
            visibility: Visibility::Public,
            like_count: 0,
            boost_count: 0,
            spoiler_text: None,
            sensitive: false,
        }
    }

    #[tokio::test]
    async fn test_fetch_status_returns_mocked_response() {
        setup();
        push_fetch_status_response(FetchStatusResponse::Ok(fixture_status()));

        let resp = fetch_status(FetchStatusArgs {
            uri: "https://x/users/a/statuses/1".into(),
            requester_actor_uri: None,
        })
        .await
        .expect("should call");

        assert!(matches!(resp, FetchStatusResponse::Ok(_)));
    }
}
