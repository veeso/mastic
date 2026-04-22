//! User canister adapter trait and implementations.
//!
//! Abstracts user canister calls (currently `emit_delete_profile_activity`) behind
//! a trait so that directory domain logic can be unit-tested without a running replica.

#[cfg(not(target_family = "wasm"))]
pub mod mock;

use std::future::Future;

use candid::Principal;

/// Abstraction over the user canister API.
pub trait UserCanister: Send + Sync + Sized {
    /// Ask the target user canister to aggregate and dispatch `Delete(Person)` activities
    /// to all its followers.
    fn emit_delete_profile_activity(
        &self,
        user_canister_id: Principal,
    ) -> impl Future<Output = Result<(), UserCanisterError>>;
}

/// Errors returned by [`UserCanister`] operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum UserCanisterError {
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
pub struct IcUserCanisterClient;

#[cfg(target_family = "wasm")]
impl UserCanister for IcUserCanisterClient {
    async fn emit_delete_profile_activity(
        &self,
        user_canister_id: Principal,
    ) -> Result<(), UserCanisterError> {
        use did::user::EmitDeleteProfileActivityResponse;

        ic_utils::log!(
            "IcUserCanisterClient::emit_delete_profile_activity: calling canister {user_canister_id}"
        );

        let raw =
            ic_cdk::call::Call::bounded_wait(user_canister_id, "emit_delete_profile_activity")
                .await
                .map_err(|e| {
                    ic_utils::log!(
                        "IcUserCanisterClient::emit_delete_profile_activity: call failed: {e:?}"
                    );
                    UserCanisterError::CallFailed(format!("{e:?}"))
                })?;

        let response =
            candid::decode_one::<EmitDeleteProfileActivityResponse>(&raw).map_err(|e| {
                ic_utils::log!(
                    "IcUserCanisterClient::emit_delete_profile_activity: decode failed: {e}"
                );
                UserCanisterError::DecodeFailed(e.to_string())
            })?;

        match response {
            EmitDeleteProfileActivityResponse::Ok => Ok(()),
            EmitDeleteProfileActivityResponse::Err(e) => {
                ic_utils::log!(
                    "IcUserCanisterClient::emit_delete_profile_activity: user canister error: {e:?}"
                );
                Err(UserCanisterError::CallFailed(format!("{e:?}")))
            }
        }
    }
}
