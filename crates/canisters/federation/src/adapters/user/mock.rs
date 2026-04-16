//! Mock implementation of [`UserCanister`] for unit tests.
//!
//! Only compiled on non-wasm targets (see the `#[cfg(not(target_family =
//! "wasm"))]` gate on the `mock` module declaration in [`super`]). Tests
//! that need alternative mock behavior (e.g. returning an error) should add
//! additional mock types in this module instead of modifying the default
//! always-Ok client.

use did::user::ReceiveActivityArgs;

use super::{UserCanister, UserCanisterClientError};

/// A test-only [`UserCanister`] whose `receive_activity` implementation
/// always returns `Ok(())`.
///
/// Use this when the behavior under test does not care about the User
/// Canister response (only that the call was attempted).
#[derive(Debug)]
pub struct MockUserCanisterClient;

impl UserCanister for MockUserCanisterClient {
    async fn receive_activity(
        &self,
        _args: ReceiveActivityArgs,
    ) -> Result<(), UserCanisterClientError> {
        Ok(())
    }
}
