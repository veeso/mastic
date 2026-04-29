//! Inspect module for user canister calls.

use candid::Principal;

pub fn inspect() {
    let method = ic_cdk::api::msg_method_name();

    match method.as_str() {
        "retry_sign_up" | "sign_up" | "whoami" => {
            let caller = ic_utils::caller();
            if caller == Principal::anonymous() {
                ic_cdk::api::msg_reject("Anonymous caller cannot do this");
                return;
            }
        }
        "search_profiles" => {
            // check limit and offset
            let args = ic_cdk::api::msg_arg_data();
            let args = candid::decode_one::<did::directory::SearchProfilesArgs>(&args)
                .expect("failed to decode arguments");
            if let Err(err) = crate::api::inspect::inspect_search_profiles(&args) {
                ic_cdk::api::msg_reject(err);
                return;
            }
        }
        _ => {}
    }

    ic_cdk::api::accept_message();
}
