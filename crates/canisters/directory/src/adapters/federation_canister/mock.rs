//! Mock implementation of [`FederationCanister`] for unit tests.

use candid::Principal;

use super::{FederationCanister, FederationCanisterError};

/// A test-only [`FederationCanister`] that always succeeds.
#[derive(Debug)]
pub struct MockFederationCanisterClient;

impl FederationCanister for MockFederationCanisterClient {
    async fn register_user(
        &self,
        _user_id: Principal,
        _user_handle: String,
        _user_canister_id: Principal,
    ) -> Result<(), FederationCanisterError> {
        Ok(())
    }
}
