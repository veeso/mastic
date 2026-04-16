//! Inspect module for user canister calls.

pub fn inspect() {
    let method_name = ic_cdk::api::msg_method_name();

    match method_name.as_str() {
        "register_user" => {
            let caller = ic_utils::caller();
            if !crate::api::inspect::is_directory_canister(caller) {
                ic_cdk::api::msg_reject(
                    "Unauthorized caller. Only the directory canister can call this method.",
                );
                return;
            }
        }
        "send_activity" => {
            let caller = ic_utils::caller();
            if !crate::api::inspect::is_user_canister(caller) {
                ic_cdk::api::msg_reject(
                    "Unauthorized caller. Only registered user canisters can call this method.",
                );
                return;
            }
        }
        _ => {}
    }

    ic_cdk::api::accept_message();
}
