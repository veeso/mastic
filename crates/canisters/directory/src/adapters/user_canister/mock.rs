//! Mock implementation of [`UserCanister`] for unit tests.

use candid::Principal;

use super::{UserCanister, UserCanisterError};

/// A test-only [`UserCanister`] that always succeeds.
#[derive(Debug)]
pub struct MockUserCanisterClient;

impl UserCanister for MockUserCanisterClient {
    async fn emit_delete_profile_activity(
        &self,
        _user_canister_id: Principal,
    ) -> Result<(), UserCanisterError> {
        Ok(())
    }
}
