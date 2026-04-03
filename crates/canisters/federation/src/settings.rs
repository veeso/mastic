//! Settings for the federation canister

use std::cell::RefCell;

use candid::Principal;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableCell};

use crate::memory::{
    DIRECTORY_CANISTER_MEMORY_ID, MEMORY_MANAGER, PUBLIC_URL_MEMORY_ID, StorablePrincipal,
};

thread_local! {

    /// Stable cell for the directory canister principal
    static DIRECTORY_CANISTER: RefCell<StableCell<StorablePrincipal, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(DIRECTORY_CANISTER_MEMORY_ID)), Principal::anonymous().into()));

    /// Stable cell for the public URL
    static PUBLIC_URL: RefCell<StableCell<String, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(PUBLIC_URL_MEMORY_ID)), "".to_string()));

}

/// Set the directory canister principal
pub fn set_directory_canister(principal: Principal) {
    DIRECTORY_CANISTER.with_borrow_mut(|cell| cell.set(principal.into()));
}

/// Get the directory canister principal
pub fn get_directory_canister() -> Principal {
    DIRECTORY_CANISTER.with_borrow(|cell| *cell.get().as_principal())
}

/// Set the public URL
pub fn set_public_url(url: String) {
    PUBLIC_URL.with_borrow_mut(|cell| cell.set(url));
}

/// Get the public URL
pub fn get_public_url() -> String {
    PUBLIC_URL.with_borrow(|cell| cell.get().clone())
}

#[cfg(test)]
mod tests {

    use candid::Principal;

    use super::*;
    use crate::test_utils::setup;

    #[test]
    fn test_should_overwrite_directory_canister_setting() {
        setup();

        let new_directory =
            Principal::from_text("b77ix-eeaaa-aaaaa-qaada-cai").expect("valid principal");
        set_directory_canister(new_directory);

        assert_eq!(get_directory_canister(), new_directory);
    }

    #[test]
    fn test_should_overwrite_public_url_setting() {
        setup();

        let new_url = "https://new.mastic.social".to_string();
        set_public_url(new_url.clone());

        assert_eq!(get_public_url(), new_url);
    }
}
