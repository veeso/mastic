//! Inspect module for user canister calls.

pub fn inspect() {
    let method_name = ic_cdk::api::msg_method_name();
    if method_name.as_str() == "publish_status" {
        let caller = ic_utils::caller();
        if !crate::api::inspect::is_owner(caller) {
            ic_cdk::api::msg_reject("Unauthorized caller");
            return;
        }
    }

    ic_cdk::api::accept_message();
}
