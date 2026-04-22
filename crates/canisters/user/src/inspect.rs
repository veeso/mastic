//! Inspect module for user canister calls.

pub fn inspect() {
    let method_name = ic_cdk::api::msg_method_name();

    match method_name.as_str() {
        "accept_follow"
        | "follow_user"
        | "get_follow_requests"
        | "publish_status"
        | "read_feed"
        | "reject_follow"
        | "update_profile" => {
            let caller = ic_utils::caller();
            if !crate::api::inspect::is_owner(caller) {
                ic_cdk::api::msg_reject(
                    "Unauthorized caller. Only the owner can call this method.",
                );
                return;
            }
        }
        "receive_activity" => {
            let caller = ic_utils::caller();
            if !crate::api::inspect::is_federation_canister(caller) {
                ic_cdk::api::msg_reject(
                    "Unauthorized caller. Only the federation canister can call this method.",
                );
                return;
            }
        }
        "emit_delete_profile_activity" => {
            let caller = ic_utils::caller();
            if !crate::api::inspect::is_directory_canister(caller) {
                ic_cdk::api::msg_reject(
                    "Unauthorized caller. Only the directory canister can call this method.",
                );
                return;
            }
        }
        _ => {}
    }

    ic_cdk::api::accept_message();
}
