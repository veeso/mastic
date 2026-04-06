//! Directory user data

use std::cell::RefCell;

use candid::Principal;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};

use crate::memory::{
    MEMORY_MANAGER, StorablePrincipal, USER_DATA_BY_HANDLE_MEMORY_ID, USER_DATA_BY_ID_MEMORY_ID,
    USER_DATA_MEMORY_ID, UserData,
};

thread_local! {

    /// Secondary index: maps a user's IC principal to the numeric key in [`USER_DATA`].
    static USER_DATA_BY_ID: RefCell<StableBTreeMap<StorablePrincipal, u64, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|mm| mm.get(USER_DATA_BY_ID_MEMORY_ID))));

    /// Secondary index: maps a user handle (e.g. `"alice"`) to the numeric key in [`USER_DATA`].
    static USER_DATA_BY_HANDLE: RefCell<StableBTreeMap<String, u64, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|mm| mm.get(USER_DATA_BY_HANDLE_MEMORY_ID))));

    /// Primary store: maps an auto-incremented numeric key to [`UserData`].
    static USER_DATA: RefCell<StableBTreeMap<u64, UserData, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|mm| mm.get(USER_DATA_MEMORY_ID))));

}

/// Insert a new user into the directory, returning the assigned user ID.
pub fn insert_user(user_id: Principal, user_handle: String, user_canister_id: Principal) {
    let next_id = USER_DATA.with_borrow(|m| m.len());

    let user_data = UserData {
        user_id,
        user_handle: user_handle.clone(),
        user_canister_id,
    };

    USER_DATA.with_borrow_mut(|data| data.insert(next_id, user_data));
    USER_DATA_BY_ID.with_borrow_mut(|index| index.insert(StorablePrincipal(user_id), next_id));
    USER_DATA_BY_HANDLE.with_borrow_mut(|index| index.insert(user_handle, next_id));
}

/// Get user data by user ID, returning `None` if no user with the given ID exists.
#[allow(
    dead_code,
    reason = "will be used by activity routing in a later milestone"
)]
pub fn get_user_by_id(user_id: &Principal) -> Option<UserData> {
    let key = USER_DATA_BY_ID.with_borrow(|data| data.get(&StorablePrincipal(*user_id)))?;

    USER_DATA.with_borrow(|data| data.get(&key))
}

/// Get user data by handle, returning `None` if no user with the given handle exists.
#[allow(
    dead_code,
    reason = "will be used by activity routing in a later milestone"
)]
pub fn get_user_by_handle(user_handle: &String) -> Option<UserData> {
    let key = USER_DATA_BY_HANDLE.with_borrow(|data| data.get(user_handle))?;

    USER_DATA.with_borrow(|data| data.get(&key))
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Clears all three thread-local maps so tests are independent of execution order.
    fn reset_state() {
        USER_DATA.with_borrow_mut(|m| {
            let keys: Vec<_> = m.iter().map(|entry| *entry.key()).collect();
            for k in keys {
                m.remove(&k);
            }
        });
        USER_DATA_BY_ID.with_borrow_mut(|m| {
            let keys: Vec<_> = m.iter().map(|entry| *entry.key()).collect();
            for k in keys {
                m.remove(&k);
            }
        });
        USER_DATA_BY_HANDLE.with_borrow_mut(|m| {
            let keys: Vec<_> = m.iter().map(|entry| entry.key().clone()).collect();
            for k in keys {
                m.remove(&k);
            }
        });
    }

    fn alice_principal() -> Principal {
        Principal::from_text("mfufu-x6j4c-gomzb-geilq").unwrap()
    }

    fn alice_canister() -> Principal {
        Principal::from_text("bkyz2-fmaaa-aaaaa-qaaaq-cai").unwrap()
    }

    fn bob_principal() -> Principal {
        Principal::from_text("bs5l3-6b3zu-dpqyj-p2x4a-jyg4k-goneb-afof2-y5d62-skt67-3756q-dqe")
            .unwrap()
    }

    fn bob_canister() -> Principal {
        Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").unwrap()
    }

    // M-UNIT-TEST: insert_user stores data retrievable by both ID and handle.
    #[test]
    fn test_should_insert_and_retrieve_user_by_id() {
        reset_state();

        insert_user(alice_principal(), "alice".to_string(), alice_canister());

        let user = get_user_by_id(&alice_principal()).expect("user should exist");
        assert_eq!(user.user_id, alice_principal());
        assert_eq!(user.user_handle, "alice");
        assert_eq!(user.user_canister_id, alice_canister());
    }

    // M-UNIT-TEST: insert_user stores data retrievable by handle.
    #[test]
    fn test_should_insert_and_retrieve_user_by_handle() {
        reset_state();

        insert_user(alice_principal(), "alice".to_string(), alice_canister());

        let user =
            get_user_by_handle(&"alice".to_string()).expect("user should be found by handle");
        assert_eq!(user.user_id, alice_principal());
        assert_eq!(user.user_handle, "alice");
        assert_eq!(user.user_canister_id, alice_canister());
    }

    // M-UNIT-TEST: get_user_by_id returns None for an unknown principal.
    #[test]
    fn test_should_return_none_for_unknown_id() {
        reset_state();

        assert!(get_user_by_id(&alice_principal()).is_none());
    }

    // M-UNIT-TEST: get_user_by_handle returns None for an unknown handle.
    #[test]
    fn test_should_return_none_for_unknown_handle() {
        reset_state();

        assert!(get_user_by_handle(&"nonexistent".to_string()).is_none());
    }

    // M-UNIT-TEST: multiple users can be inserted and retrieved independently.
    #[test]
    fn test_should_insert_multiple_users() {
        reset_state();

        insert_user(alice_principal(), "alice".to_string(), alice_canister());
        insert_user(bob_principal(), "bob".to_string(), bob_canister());

        let alice = get_user_by_id(&alice_principal()).expect("alice should exist");
        assert_eq!(alice.user_handle, "alice");
        assert_eq!(alice.user_canister_id, alice_canister());

        let bob = get_user_by_id(&bob_principal()).expect("bob should exist");
        assert_eq!(bob.user_handle, "bob");
        assert_eq!(bob.user_canister_id, bob_canister());
    }

    // M-UNIT-TEST: both indexes point to the same user for the same insert.
    #[test]
    fn test_should_return_consistent_data_across_indexes() {
        reset_state();

        insert_user(alice_principal(), "alice".to_string(), alice_canister());

        let by_id = get_user_by_id(&alice_principal()).expect("by_id lookup");
        let by_handle = get_user_by_handle(&"alice".to_string()).expect("by_handle lookup");

        assert_eq!(by_id, by_handle);
    }

    // M-UNIT-TEST: auto-incremented keys are unique across inserts.
    #[test]
    fn test_should_assign_sequential_keys() {
        reset_state();

        insert_user(alice_principal(), "alice".to_string(), alice_canister());
        insert_user(bob_principal(), "bob".to_string(), bob_canister());

        let alice_key =
            USER_DATA_BY_ID.with_borrow(|m| m.get(&StorablePrincipal(alice_principal())));
        let bob_key = USER_DATA_BY_ID.with_borrow(|m| m.get(&StorablePrincipal(bob_principal())));

        assert_eq!(alice_key, Some(0));
        assert_eq!(bob_key, Some(1));
    }

    // M-UNIT-TEST: retrieving bob does not return alice's data.
    #[test]
    fn test_should_not_cross_contaminate_users() {
        reset_state();

        insert_user(alice_principal(), "alice".to_string(), alice_canister());
        insert_user(bob_principal(), "bob".to_string(), bob_canister());

        let alice = get_user_by_handle(&"alice".to_string()).unwrap();
        let bob = get_user_by_handle(&"bob".to_string()).unwrap();

        assert_ne!(alice.user_id, bob.user_id);
        assert_ne!(alice.user_canister_id, bob.user_canister_id);
    }
}
