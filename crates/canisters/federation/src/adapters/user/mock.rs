//! Mock implementation of [`UserCanister`] for unit tests.
//!
//! Only compiled on non-wasm targets (see the `#[cfg(not(target_family =
//! "wasm"))]` gate on the `mock` module declaration in [`super`]). Tests
//! that need alternative mock behavior (e.g. returning an error) should add
//! additional mock types in this module instead of modifying the default
//! always-Ok client.

use std::cell::RefCell;
use std::collections::VecDeque;

use did::user::{GetLocalStatusArgs, GetLocalStatusResponse, ReceiveActivityArgs};

use super::{UserCanister, UserCanisterClientError};

thread_local! {
    /// FIFO queue of canned [`GetLocalStatusResponse`] values returned by
    /// [`MockUserCanisterClient::get_local_status`]. Tests push expected
    /// responses with [`push_get_local_status_response`] before invoking the
    /// flow under test.
    pub static GET_LOCAL_STATUS_RESPONSES: RefCell<VecDeque<GetLocalStatusResponse>> =
        const { RefCell::new(VecDeque::new()) };
    /// Captured arguments observed by
    /// [`MockUserCanisterClient::get_local_status`] in invocation order.
    /// Tests inspect this with [`captured_get_local_status_calls`] to assert
    /// on the forwarded args.
    pub static GET_LOCAL_STATUS_CALLS: RefCell<Vec<GetLocalStatusArgs>> =
        const { RefCell::new(Vec::new()) };
}

/// Enqueue a canned response to be returned by the next call to
/// [`MockUserCanisterClient::get_local_status`].
#[allow(
    dead_code,
    reason = "exposed as a test helper; called from cfg(test) sites"
)]
pub fn push_get_local_status_response(resp: GetLocalStatusResponse) {
    GET_LOCAL_STATUS_RESPONSES.with_borrow_mut(|q| q.push_back(resp));
}

/// Snapshot of the args captured by past calls to
/// [`MockUserCanisterClient::get_local_status`], in invocation order.
#[allow(
    dead_code,
    reason = "exposed as a test helper; not all federation tests inspect calls yet"
)]
pub fn captured_get_local_status_calls() -> Vec<GetLocalStatusArgs> {
    GET_LOCAL_STATUS_CALLS.with_borrow(|v| v.clone())
}

/// Clear both the canned response queue and the captured-calls log so
/// tests start from a clean slate.
#[allow(
    dead_code,
    reason = "exposed as a test helper; called from cfg(test) sites"
)]
pub fn reset_get_local_status() {
    GET_LOCAL_STATUS_RESPONSES.with_borrow_mut(|q| q.clear());
    GET_LOCAL_STATUS_CALLS.with_borrow_mut(|v| v.clear());
}

/// A test-only [`UserCanister`] whose `receive_activity` implementation
/// always returns `Ok(())`, and whose `get_local_status` implementation
/// returns the next canned response from the
/// [`GET_LOCAL_STATUS_RESPONSES`] queue.
///
/// Use this when the behavior under test does not care about the User
/// Canister response (only that the call was attempted), or when the test
/// pre-seeds an explicit response via [`push_get_local_status_response`].
#[derive(Debug)]
pub struct MockUserCanisterClient;

impl UserCanister for MockUserCanisterClient {
    async fn receive_activity(
        &self,
        _args: ReceiveActivityArgs,
    ) -> Result<(), UserCanisterClientError> {
        Ok(())
    }

    async fn get_local_status(
        &self,
        args: GetLocalStatusArgs,
    ) -> Result<GetLocalStatusResponse, UserCanisterClientError> {
        GET_LOCAL_STATUS_CALLS.with_borrow_mut(|v| v.push(args.clone()));
        GET_LOCAL_STATUS_RESPONSES
            .with_borrow_mut(|q| q.pop_front())
            .ok_or_else(|| {
                UserCanisterClientError::CallFailed(
                    "MockUserCanisterClient::get_local_status: no canned response queued"
                        .to_string(),
                )
            })
    }
}
