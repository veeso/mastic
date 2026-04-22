use std::cell::RefCell;

use did::federation::SendActivityArgs;

use crate::adapters::federation::FederationCanister;

thread_local! {
    static CAPTURED: RefCell<Vec<SendActivityArgs>> = const { RefCell::new(Vec::new()) };
}

#[derive(Debug)]
pub struct FederationCanisterMockClient;

impl FederationCanister for FederationCanisterMockClient {
    async fn send_activity(
        &self,
        args: SendActivityArgs,
    ) -> Result<(), crate::adapters::federation::FederationCanisterClientError> {
        CAPTURED.with(|c| c.borrow_mut().push(args));
        Ok(())
    }
}

/// Reset the captured-activities buffer. Call from `test_utils::setup`.
pub fn reset_captured() {
    CAPTURED.with(|c| c.borrow_mut().clear());
}

/// Return a clone of every [`SendActivityArgs`] passed to
/// [`FederationCanisterMockClient::send_activity`] since the last
/// [`reset_captured`].
#[allow(dead_code, reason = "used by domain-level unit tests")]
pub fn captured() -> Vec<SendActivityArgs> {
    CAPTURED.with(|c| c.borrow().clone())
}
