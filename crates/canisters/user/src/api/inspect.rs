//! Inspect module for checking permissions and other user-related information.

use candid::Principal;

/// Inspect whether the provided [`Principal`] is the owner of the canister.
pub fn is_owner(principal: Principal) -> bool {
    principal == crate::settings::get_owner_principal().expect("should read owner principal")
}

/// Inspect whether the provided [`Principal`] is the federation canister.
pub fn is_federation_canister(principal: Principal) -> bool {
    principal
        == crate::settings::get_federation_canister()
            .expect("should read federation canister principal")
}

/// Inspect whether the provided [`Principal`] is the directory canister.
pub fn is_directory_canister(principal: Principal) -> bool {
    principal
        == crate::settings::get_directory_canister()
            .expect("should read directory canister principal")
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_should_check_if_owner() {
        crate::test_utils::setup();

        assert!(is_owner(crate::test_utils::admin()));
        assert!(!is_owner(crate::test_utils::alice()));
    }

    #[test]
    fn test_should_check_if_federation_canister() {
        crate::test_utils::setup();

        assert!(is_federation_canister(crate::test_utils::federation()));
        assert!(!is_federation_canister(crate::test_utils::alice()));
    }
}
