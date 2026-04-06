//! Federation canister adapter trait and implementations.
//!
//! Abstracts federation canister calls (`register_user`) behind a trait so that
//! domain logic can be unit-tested without a running replica.

#[cfg(not(target_family = "wasm"))]
pub mod mock;

use std::future::Future;

use candid::Principal;

/// Abstraction over the federation canister API.
///
/// Currently exposes only user registration, which the directory canister calls
/// after successfully installing a new user canister. Extend this trait if
/// additional federation calls become necessary.
pub trait FederationCanister: Send + Sync + Sized {
    /// Registers a user with the federation canister so it can route
    /// ActivityPub traffic to the correct user canister.
    fn register_user(
        &self,
        user_id: Principal,
        user_handle: String,
        user_canister_id: Principal,
    ) -> impl Future<Output = Result<(), FederationCanisterError>>;
}

/// Errors returned by [`FederationCanister`] operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum FederationCanisterError {
    /// The inter-canister call failed.
    #[error("inter-canister call failed: {0}")]
    #[allow(dead_code, reason = "constructed only in wasm builds")]
    #[cfg(any(target_family = "wasm", test))]
    CallFailed(String),
    /// The response could not be decoded.
    #[error("decode failed: {0}")]
    #[allow(dead_code, reason = "constructed only in wasm builds")]
    #[cfg(target_family = "wasm")]
    DecodeFailed(String),
}

/// Production implementation that delegates to `ic_cdk` calls.
#[cfg(target_family = "wasm")]
pub struct IcFederationCanisterClient {
    canister_id: Principal,
}

#[cfg(target_family = "wasm")]
impl From<Principal> for IcFederationCanisterClient {
    fn from(canister_id: Principal) -> Self {
        Self { canister_id }
    }
}

#[cfg(target_family = "wasm")]
impl FederationCanister for IcFederationCanisterClient {
    async fn register_user(
        &self,
        user_id: Principal,
        user_handle: String,
        user_canister_id: Principal,
    ) -> Result<(), FederationCanisterError> {
        use did::federation::{RegisterUserArgs, RegisterUserResponse};

        ic_utils::log!(
            "IcFederationCanisterClient::register_user: registering user {user_id} \
             with handle {user_handle:?} and canister {user_canister_id}"
        );

        let args = RegisterUserArgs {
            user_id,
            user_handle,
            user_canister_id,
        };

        let raw = ic_cdk::call::Call::bounded_wait(self.canister_id, "register_user")
            .with_arg(args)
            .await
            .map_err(|e| {
                ic_utils::log!("IcFederationCanisterClient::register_user: call failed: {e:?}");
                FederationCanisterError::CallFailed(format!("{e:?}"))
            })?;

        let response = candid::decode_one::<RegisterUserResponse>(&raw).map_err(|e| {
            ic_utils::log!("IcFederationCanisterClient::register_user: decode failed: {e}");
            FederationCanisterError::DecodeFailed(e.to_string())
        })?;

        match response {
            RegisterUserResponse::Ok => {
                ic_utils::log!(
                    "IcFederationCanisterClient::register_user: user {user_id} registered"
                );
                Ok(())
            }
            RegisterUserResponse::Err(e) => {
                ic_utils::log!(
                    "IcFederationCanisterClient::register_user: federation error: {e:?}"
                );
                Err(FederationCanisterError::CallFailed(format!("{e:?}")))
            }
        }
    }
}
