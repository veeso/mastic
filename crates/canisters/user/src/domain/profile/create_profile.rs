//! Create profile flow.

use candid::Principal;
use db_utils::repository::Repository;

use crate::error::CanisterResult;
use crate::repository::profile::ProfileRepository;

/// Create a brand new profile.
///
/// This flow is called on canister init and just initialize the user with its handle.
pub fn create_profile(principal: Principal, handle: &str) -> CanisterResult<()> {
    ic_utils::log!("Creating profile for principal {principal} with handle {handle}");
    ProfileRepository::oneshot().create_profile(principal, handle)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::{admin, alice, setup};

    #[test]
    fn test_should_create_profile() {
        setup();

        let result = create_profile(alice(), "alice");

        assert!(result.is_ok());
    }

    #[test]
    fn test_should_create_profile_with_sanitized_handle() {
        setup();

        let result = create_profile(alice(), "  @alice  ");

        assert!(result.is_ok());
    }

    #[test]
    fn test_should_fail_to_create_profile_with_invalid_handle() {
        setup();

        let result = create_profile(alice(), "INVALID!");

        assert!(result.is_err());
    }

    #[test]
    fn test_should_fail_to_create_profile_with_reserved_handle() {
        setup();

        let result = create_profile(alice(), "admin");

        assert!(result.is_err());
    }

    #[test]
    fn test_should_fail_to_create_profile_with_duplicate_handle() {
        setup();

        // setup() already creates a profile with handle "rey_canisteryo" for admin
        let result = create_profile(alice(), "rey_canisteryo");

        assert!(result.is_err());
    }

    #[test]
    fn test_should_fail_to_create_profile_with_duplicate_principal() {
        setup();

        // setup() already creates a profile for admin()
        let result = create_profile(admin(), "another_handle");

        assert!(result.is_err());
    }
}
