//! Inspect module for user canister calls.

pub fn inspect() {
    let method_name = ic_cdk::api::msg_method_name();

    match method_name.as_str() {
        "accept_follow"
        | "follow_user"
        | "get_follow_requests"
        | "publish_status"
        | "reject_follow" => {
            let caller = ic_utils::caller();
            if !crate::api::inspect::is_owner(caller) {
                ic_cdk::api::msg_reject("Unauthorized caller");
                return;
            }
        }
        _ => {}
    }

    ic_cdk::api::accept_message();
}
