//! Inspect module for user canister calls.

pub fn inspect() {
    ic_cdk::api::accept_message();
}
