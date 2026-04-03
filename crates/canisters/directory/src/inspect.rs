//! Inspect module for user canister calls.

use candid::Principal;

pub fn inspect() {
    let method = ic_cdk::api::msg_method_name();

    match method.as_str() {
        "retry_sign_up" | "sign_up" | "whoami" => {
            let caller = ic_utils::caller();
            if caller == Principal::anonymous() {
                ic_cdk::api::msg_reject("Anonymous caller cannot do this");
            }
        }
        _ => {}
    }

    ic_cdk::api::accept_message();
}
