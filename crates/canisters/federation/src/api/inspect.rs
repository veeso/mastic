use candid::Principal;

/// Inspect whether the provided [`Principal`] is the directory canister.
pub fn is_directory_canister(principal: Principal) -> bool {
    principal == crate::settings::get_directory_canister()
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
}
