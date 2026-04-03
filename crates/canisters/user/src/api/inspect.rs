//! Inspect module for checking permissions and other user-related information.

use candid::Principal;

/// Inspect whether the provided [`Principal`] is the owner of the canister.
pub fn is_owner(principal: Principal) -> bool {
    principal == crate::settings::get_owner_principal().expect("should read owner principal")
}
