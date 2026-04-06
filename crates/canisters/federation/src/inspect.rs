//! Inspect module for user canister calls.

pub fn inspect() {
    let method_name = ic_cdk::api::msg_method_name();

    if method_name == "register_user" {
        let caller = ic_utils::caller();
        if !crate::api::inspect::is_directory_canister(caller) {
            ic_cdk::api::msg_reject(
                "Unauthorized caller. Only the directory canister can call this method.",
            );
            return;
        }
    }

    ic_cdk::api::accept_message();
}
