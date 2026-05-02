//! User canister client adapter.
//!
//! Abstracts user canister calls (`receive_activity`) behind a trait so that
//! domain logic can be unit-tested without a running replica. Production
//! code uses [`IcUserCanisterClient`] on wasm targets; unit tests use
//! [`mock::MockUserCanisterClient`] on native targets.

#[cfg(not(target_family = "wasm"))]
pub mod mock;

use did::user::{GetLocalStatusArgs, GetLocalStatusResponse, ReceiveActivityArgs};

/// Abstraction over the user canister API used by the Federation Canister.
///
/// Callers construct an implementation (e.g. [`IcUserCanisterClient`] or
/// [`mock::MockUserCanisterClient`]) and invoke methods on it. Every method
/// returns a typed [`Result`] to let domain code distinguish transport
/// failures from canister-level rejections.
pub trait UserCanister: Send + Sync + Sized {
    /// Deliver an activity to the User Canister by calling its
    /// `receive_activity` method.
    ///
    /// Returns `Ok(())` when the target canister accepts the activity.
    /// Returns [`UserCanisterClientError::CallFailed`] or
    /// [`UserCanisterClientError::DecodeFailed`] on transport-level
    /// problems, and [`UserCanisterClientError::Rejected`] when the target
    /// canister responds with a typed error.
    fn receive_activity(
        &self,
        args: ReceiveActivityArgs,
    ) -> impl Future<Output = Result<(), UserCanisterClientError>>;

    /// Query the User Canister for a single locally-hosted status by id.
    ///
    /// Returns the raw [`GetLocalStatusResponse`] — the user-canister-level
    /// success / error variant is preserved so domain callers can decide
    /// how to map it. Transport-level problems are reported as
    /// [`UserCanisterClientError::CallFailed`] or
    /// [`UserCanisterClientError::DecodeFailed`].
    fn get_local_status(
        &self,
        args: GetLocalStatusArgs,
    ) -> impl Future<Output = Result<GetLocalStatusResponse, UserCanisterClientError>>;
}

/// Errors returned by [`UserCanister`] operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum UserCanisterClientError {
    /// The inter-canister call failed.
    #[error("inter-canister call failed: {0}")]
    #[allow(dead_code, reason = "constructed only in wasm builds")]
    CallFailed(String),
    /// The response could not be decoded.
    #[error("decode failed: {0}")]
    #[allow(dead_code, reason = "constructed only in wasm builds")]
    DecodeFailed(String),
    /// The user canister rejected the activity.
    #[error("user canister rejected the activity: {0}")]
    #[allow(dead_code, reason = "constructed only in wasm builds")]
    Rejected(String),
}

/// Production implementation of [`UserCanister`] that delegates to
/// `ic_cdk::call` on wasm targets.
///
/// Construct with [`IcUserCanisterClient::from`] passing the target User
/// Canister principal. Only available when compiling for
/// `wasm32-unknown-unknown`.
#[cfg(target_family = "wasm")]
pub struct IcUserCanisterClient {
    /// Principal of the target User Canister.
    canister_id: candid::Principal,
}

#[cfg(target_family = "wasm")]
impl From<candid::Principal> for IcUserCanisterClient {
    /// Build a client that will call the User Canister identified by
    /// `canister_id`.
    fn from(canister_id: candid::Principal) -> Self {
        Self { canister_id }
    }
}

#[cfg(target_family = "wasm")]
impl UserCanister for IcUserCanisterClient {
    async fn receive_activity(
        &self,
        args: ReceiveActivityArgs,
    ) -> Result<(), UserCanisterClientError> {
        use did::user::ReceiveActivityResponse;

        ic_utils::log!(
            "IcUserCanisterClient::receive_activity: delivering activity to {}",
            self.canister_id
        );

        let raw = ic_cdk::call::Call::bounded_wait(self.canister_id, "receive_activity")
            .with_arg(args)
            .await
            .map_err(|e| {
                ic_utils::log!("IcUserCanisterClient::receive_activity: call failed: {e:?}");
                UserCanisterClientError::CallFailed(format!("{e:?}"))
            })?;

        let response = candid::decode_one::<ReceiveActivityResponse>(&raw).map_err(|e| {
            ic_utils::log!("IcUserCanisterClient::receive_activity: decode failed: {e}");
            UserCanisterClientError::DecodeFailed(e.to_string())
        })?;

        match response {
            ReceiveActivityResponse::Ok => {
                ic_utils::log!(
                    "IcUserCanisterClient::receive_activity: activity delivered to {}",
                    self.canister_id
                );
                Ok(())
            }
            ReceiveActivityResponse::Err(e) => {
                ic_utils::log!(
                    "IcUserCanisterClient::receive_activity: user canister error: {e:?}"
                );
                Err(UserCanisterClientError::Rejected(format!("{e:?}")))
            }
        }
    }

    async fn get_local_status(
        &self,
        args: GetLocalStatusArgs,
    ) -> Result<GetLocalStatusResponse, UserCanisterClientError> {
        ic_utils::log!(
            "IcUserCanisterClient::get_local_status: querying status {} on {}",
            args.id,
            self.canister_id
        );

        let raw = ic_cdk::call::Call::bounded_wait(self.canister_id, "get_local_status")
            .with_arg(args)
            .await
            .map_err(|e| {
                ic_utils::log!("IcUserCanisterClient::get_local_status: call failed: {e:?}");
                UserCanisterClientError::CallFailed(format!("{e:?}"))
            })?;

        let response = candid::decode_one::<GetLocalStatusResponse>(&raw).map_err(|e| {
            ic_utils::log!("IcUserCanisterClient::get_local_status: decode failed: {e}");
            UserCanisterClientError::DecodeFailed(e.to_string())
        })?;

        Ok(response)
    }
}
