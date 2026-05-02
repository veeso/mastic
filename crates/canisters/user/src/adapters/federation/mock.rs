use std::cell::RefCell;
use std::collections::VecDeque;

use did::federation::{FetchStatusArgs, FetchStatusResponse, SendActivityArgs};

use crate::adapters::federation::FederationCanister;

thread_local! {
    static CAPTURED: RefCell<Vec<SendActivityArgs>> = const { RefCell::new(Vec::new()) };
    static FETCH_STATUS_RESPONSES: RefCell<VecDeque<FetchStatusResponse>> = RefCell::new(VecDeque::new());
    static FETCH_STATUS_CALLS: RefCell<Vec<FetchStatusArgs>> = const { RefCell::new(Vec::new()) };
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

    async fn fetch_status(
        &self,
        args: FetchStatusArgs,
    ) -> Result<FetchStatusResponse, crate::adapters::federation::FederationCanisterClientError>
    {
        FETCH_STATUS_CALLS.with_borrow_mut(|v| v.push(args.clone()));
        Ok(FETCH_STATUS_RESPONSES
            .with_borrow_mut(|q| q.pop_front())
            .unwrap_or_else(|| {
                panic!(
                    "FederationCanisterMockClient::fetch_status: no canned response queued for {args:?}"
                )
            }))
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

/// Queue a canned [`FetchStatusResponse`] to be returned by the next
/// [`FederationCanisterMockClient::fetch_status`] call.
pub fn push_fetch_status_response(resp: FetchStatusResponse) {
    FETCH_STATUS_RESPONSES.with_borrow_mut(|q| q.push_back(resp));
}

/// Return a clone of every [`FetchStatusArgs`] passed to
/// [`FederationCanisterMockClient::fetch_status`] since the last
/// [`reset_fetch_status`].
#[allow(dead_code, reason = "used by domain-level unit tests")]
pub fn captured_fetch_status_calls() -> Vec<FetchStatusArgs> {
    FETCH_STATUS_CALLS.with_borrow(|v| v.clone())
}

/// Reset the fetch-status response queue and call log. Call from `test_utils::setup`.
pub fn reset_fetch_status() {
    FETCH_STATUS_RESPONSES.with_borrow_mut(|q| q.clear());
    FETCH_STATUS_CALLS.with_borrow_mut(|v| v.clear());
}
