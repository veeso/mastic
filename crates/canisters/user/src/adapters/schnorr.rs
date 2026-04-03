// TODO: remove once canister methods use this module
#![allow(dead_code, reason = "will be used by upcoming canister methods")]

//! Schnorr signing adapter trait and implementations.
//!
//! Abstracts IC threshold Schnorr calls (`schnorr_public_key`,
//! `sign_with_schnorr`) behind a trait so that domain logic can be
//! unit-tested without a running replica.

#[cfg(not(target_family = "wasm"))]
pub mod mock;

use std::future::Future;

/// Abstraction over the IC threshold Schnorr API.
///
/// Exposes only the two operations needed by the user canister today:
/// retrieving the Ed25519 public key and signing a message.
pub trait SchnorrCanister: Send + Sync + Sized {
    /// Retrieves the Ed25519 public key for this canister.
    ///
    /// The key is derived from the subnet's threshold key using an
    /// empty derivation path, which yields a canister-unique key.
    fn schnorr_public_key(&self) -> impl Future<Output = Result<Vec<u8>, SchnorrCanisterError>>;

    /// Signs `message` with the canister's Ed25519 threshold key.
    fn sign_with_schnorr(
        &self,
        message: Vec<u8>,
    ) -> impl Future<Output = Result<Vec<u8>, SchnorrCanisterError>>;
}

/// Errors returned by [`SchnorrCanister`] operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SchnorrCanisterError {
    /// The inter-canister call failed.
    #[error("inter-canister call failed: {0}")]
    CallFailed(String),
    /// Failed to decode the response.
    #[cfg(target_family = "wasm")]
    #[error("failed to decode response: {0}")]
    DecodeFailed(String),
}

/// Key name used for the Ed25519 test key on the IC.
#[cfg(target_family = "wasm")]
const KEY_NAME: &str = "dfx_test_key";

/// Production implementation that delegates to `ic_cdk` management canister calls.
#[cfg(target_family = "wasm")]
pub struct IcSchnorrClient;

#[cfg(target_family = "wasm")]
impl SchnorrCanister for IcSchnorrClient {
    async fn schnorr_public_key(&self) -> Result<Vec<u8>, SchnorrCanisterError> {
        ic_utils::log!("IcSchnorrClient::schnorr_public_key: requesting Ed25519 public key");

        let request = ic_management_canister_types::SchnorrPublicKeyArgs {
            canister_id: None,
            derivation_path: vec![],
            key_id: ic_management_canister_types::SchnorrKeyId {
                algorithm: ic_management_canister_types::SchnorrAlgorithm::Ed25519,
                name: KEY_NAME.to_string(),
            },
        };

        let response = ic_cdk::call::Call::bounded_wait(
            candid::Principal::management_canister(),
            "schnorr_public_key",
        )
        .with_arg(request)
        .await
        .map_err(|e| {
            ic_utils::log!("IcSchnorrClient::schnorr_public_key: call failed: {e:?}");
            SchnorrCanisterError::CallFailed(format!("{e:?}"))
        })?;

        let result =
            candid::decode_one::<ic_management_canister_types::SchnorrPublicKeyResult>(&response)
                .map_err(|e| {
                ic_utils::log!("IcSchnorrClient::schnorr_public_key: decode failed: {e}");
                SchnorrCanisterError::DecodeFailed(e.to_string())
            })?;

        ic_utils::log!(
            "IcSchnorrClient::schnorr_public_key: received public key ({} bytes)",
            result.public_key.len()
        );

        Ok(result.public_key)
    }

    async fn sign_with_schnorr(&self, message: Vec<u8>) -> Result<Vec<u8>, SchnorrCanisterError> {
        ic_utils::log!(
            "IcSchnorrClient::sign_with_schnorr: signing message ({} bytes)",
            message.len()
        );

        let request = ic_management_canister_types::SignWithSchnorrArgs {
            message,
            derivation_path: vec![],
            key_id: ic_management_canister_types::SchnorrKeyId {
                algorithm: ic_management_canister_types::SchnorrAlgorithm::Ed25519,
                name: KEY_NAME.to_string(),
            },
            aux: None,
        };

        let response = ic_cdk::call::Call::bounded_wait(
            candid::Principal::management_canister(),
            "sign_with_schnorr",
        )
        .with_arg(request)
        .await
        .map_err(|e| {
            ic_utils::log!("IcSchnorrClient::sign_with_schnorr: call failed: {e:?}");
            SchnorrCanisterError::CallFailed(format!("{e:?}"))
        })?;

        let result =
            candid::decode_one::<ic_management_canister_types::SignWithSchnorrResult>(&response)
                .map_err(|e| {
                    ic_utils::log!("IcSchnorrClient::sign_with_schnorr: decode failed: {e}");
                    SchnorrCanisterError::DecodeFailed(e.to_string())
                })?;

        ic_utils::log!(
            "IcSchnorrClient::sign_with_schnorr: signature produced ({} bytes)",
            result.signature.len()
        );

        Ok(result.signature)
    }
}
