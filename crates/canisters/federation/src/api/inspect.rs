use candid::Principal;

/// Inspect whether the provided [`Principal`] is the directory canister.
pub fn is_directory_canister(principal: Principal) -> bool {
    principal == crate::settings::get_directory_canister()
}

/// Inspect whether the provided [`Principal`] is a registered User Canister.
pub fn is_user_canister(principal: Principal) -> bool {
    crate::directory::get_user_by_canister_id(&principal).is_some()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_should_check_if_directory_canister() {
        crate::test_utils::setup();

        assert!(is_directory_canister(crate::test_utils::directory()));
        assert!(!is_directory_canister(crate::test_utils::alice()));
    }

    #[test]
    fn test_should_check_if_user_canister() {
        crate::test_utils::setup();

        // add alice
        crate::directory::insert_user(
            crate::test_utils::alice(),
            "alice".to_string(),
            crate::test_utils::alice(),
        );

        assert!(is_user_canister(crate::test_utils::alice()));
        assert!(!is_user_canister(crate::test_utils::directory()));
    }
}
