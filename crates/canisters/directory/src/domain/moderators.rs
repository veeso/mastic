//! Moderation functions for the Directory canister.

mod repository;

use candid::Principal;

use crate::domain::moderators::repository::ModeratorsRepository;
use crate::error::CanisterResult;

/// Adds a moderator to the directory canister.
pub fn add_moderator(principal: Principal) -> CanisterResult<()> {
    ic_utils::log!("add_moderator: adding {principal}");
    ModeratorsRepository::oneshot().add_moderator(principal)
}

/// Returns true if the given principal is a moderator, false otherwise.
#[cfg_attr(not(test), expect(dead_code))]
pub fn is_moderator(principal: Principal) -> CanisterResult<bool> {
    ic_utils::log!("is_moderator: checking {principal}");
    ModeratorsRepository::oneshot().is_moderator(principal)
}

/// Removes a moderator from the directory canister.
#[cfg_attr(not(test), expect(dead_code))]
pub fn remove_moderator(principal: Principal) -> CanisterResult<()> {
    ic_utils::log!("remove_moderator: removing {principal}");
    ModeratorsRepository::oneshot().remove_moderator(principal)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::{rey_canisteryo, setup};

    #[test]
    fn test_should_add_and_check_moderator() {
        setup();

        add_moderator(rey_canisteryo()).expect("should add moderator");

        assert!(is_moderator(rey_canisteryo()).expect("should check moderator"));
    }

    #[test]
    fn test_should_remove_moderator() {
        setup();

        add_moderator(rey_canisteryo()).expect("should add moderator");
        remove_moderator(rey_canisteryo()).expect("should remove moderator");

        assert!(!is_moderator(rey_canisteryo()).expect("should check moderator"));
    }

    #[test]
    fn test_should_report_non_moderator_as_false() {
        setup();

        assert!(!is_moderator(rey_canisteryo()).expect("should check moderator"));
    }
}
