//! Mock implementation of [`SchnorrCanister`] for unit tests.

use super::{SchnorrCanister, SchnorrCanisterError};

/// A test-only [`SchnorrCanister`] that returns pre-configured values.
#[derive(Debug)]
pub struct MockSchnorrClient {
    /// The public key returned by `schnorr_public_key`.
    pub public_key: Vec<u8>,
    /// The signature returned by `sign_with_schnorr`.
    pub signature: Vec<u8>,
}

impl SchnorrCanister for MockSchnorrClient {
    async fn schnorr_public_key(&self) -> Result<Vec<u8>, SchnorrCanisterError> {
        Ok(self.public_key.clone())
    }

    async fn sign_with_schnorr(&self, _message: Vec<u8>) -> Result<Vec<u8>, SchnorrCanisterError> {
        Ok(self.signature.clone())
    }
}
