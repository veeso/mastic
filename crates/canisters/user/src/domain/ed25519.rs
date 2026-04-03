// TODO: remove once canister methods use this module
#![allow(dead_code, reason = "will be used by upcoming canister methods")]

//! Ed25519 key management via IC threshold Schnorr signing.
//!
//! Provides functions to retrieve the canister's Ed25519 public key and
//! to sign arbitrary messages. The public key is fetched once from the
//! management canister and cached in a thread-local for subsequent calls.

use std::cell::RefCell;

use crate::adapters::schnorr::SchnorrCanister;
use crate::error::{CanisterError, CanisterResult};

thread_local! {
    /// Cached Ed25519 public key. Populated on first call to [`public_key`].
    static PUBLIC_KEY: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
}

/// Returns the canister's Ed25519 public key.
///
/// On the first invocation the key is fetched from the management
/// canister via [`SchnorrCanister::schnorr_public_key`] and then cached in a thread-local.
/// Subsequent calls return the cached value without an inter-canister call.
pub async fn public_key<C>(client: &C) -> CanisterResult<Vec<u8>>
where
    C: SchnorrCanister,
{
    let cached = PUBLIC_KEY.with(|pk| pk.borrow().clone());

    if let Some(key) = cached {
        return Ok(key);
    }

    let key = client
        .schnorr_public_key()
        .await
        .map_err(|e| CanisterError::SchnorrCall(e.to_string()))?;

    PUBLIC_KEY.with(|pk| {
        *pk.borrow_mut() = Some(key.clone());
    });

    Ok(key)
}

/// Signs `message` with the canister's Ed25519 threshold key via [`SchnorrCanister::sign_with_schnorr`].
pub async fn sign<C>(client: &C, message: Vec<u8>) -> CanisterResult<Vec<u8>>
where
    C: SchnorrCanister,
{
    client
        .sign_with_schnorr(message)
        .await
        .map_err(|e| CanisterError::SchnorrCall(e.to_string()))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::adapters::schnorr::mock::MockSchnorrClient;

    fn mock_client() -> MockSchnorrClient {
        MockSchnorrClient {
            public_key: vec![1, 2, 3, 4],
            signature: vec![10, 20, 30],
        }
    }

    #[tokio::test]
    async fn test_should_return_public_key() {
        // Reset the cache before test.
        PUBLIC_KEY.with(|pk| *pk.borrow_mut() = None);

        let client = mock_client();
        let key = public_key(&client).await.expect("should return public key");

        assert_eq!(key, vec![1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_should_cache_public_key() {
        // Reset and populate cache.
        PUBLIC_KEY.with(|pk| *pk.borrow_mut() = None);

        let client = mock_client();
        let key1 = public_key(&client).await.expect("first call");

        // Second call should return the same cached value even with a
        // different mock (proving it does not call the client again).
        let different_client = MockSchnorrClient {
            public_key: vec![99, 99],
            signature: vec![],
        };
        let key2 = public_key(&different_client)
            .await
            .expect("second call (cached)");

        assert_eq!(key1, key2);
    }

    #[tokio::test]
    async fn test_should_sign_message() {
        let client = mock_client();
        let sig = sign(&client, b"hello".to_vec()).await.expect("should sign");

        assert_eq!(sig, vec![10, 20, 30]);
    }
}
