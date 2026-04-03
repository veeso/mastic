//! Flow for getting the `user_canister` by principal.

use candid::Principal;
use did::directory::{UserCanisterError, UserCanisterResponse};

use crate::domain::users::repository::UserRepository;

/// Given a [`Principal`] of a user, returns the [`Principal`] of their User Canister if they are registered in the directory.
///
/// 1. If the caller is not registered in the directory, returns [`UserCanisterError::NotRegistered`].
/// 2. If the caller's canister is not active (e.g. pending creation or failed), returns [`UserCanisterError::CanisterNotActive`].
/// 3. If an internal error occurs while retrieving the User Canister, returns [`UserCanisterError::InternalError`] with a message describing the error.
/// 4. If the caller is registered and their canister is active, returns the [`Principal`] of their User Canister wrapped in [`UserCanisterResponse::Ok`].
pub fn user_canister(user: Principal) -> UserCanisterResponse {
    ic_utils::log!("user_canister: looking up user {user}");

    let user_data = match UserRepository::get_user_by_principal(user) {
        Ok(Some(user)) => user,
        Ok(None) => {
            ic_utils::log!("whoami: user {user} is not registered");
            return UserCanisterResponse::Err(UserCanisterError::NotRegistered);
        }
        Err(e) => {
            ic_utils::log!("whoami: internal error querying user {user}: {e}");
            return UserCanisterResponse::Err(UserCanisterError::InternalError(format!(
                "failed to query database: {e}"
            )));
        }
    };

    match user_data.canister_id.into_opt() {
        Some(canister_id) => {
            ic_utils::log!("user_canister: found canister {canister_id} for user {user}");
            UserCanisterResponse::Ok(canister_id.0)
        }
        None => {
            ic_utils::log!("user_canister: user {user} has no active canister");
            UserCanisterResponse::Err(UserCanisterError::CanisterNotActive)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::{
        bob, rey_canisteryo, setup, setup_registered_user, setup_registered_user_with_canister,
    };

    #[test]
    fn test_should_return_canister_for_active_user() {
        setup();

        let canister_id = rey_canisteryo();
        setup_registered_user_with_canister(bob(), "alice", canister_id);

        let response = user_canister(bob());

        assert_eq!(response, UserCanisterResponse::Ok(canister_id));
    }

    #[test]
    fn test_should_return_not_active_when_canister_pending() {
        setup();
        setup_registered_user(bob(), "alice");

        let response = user_canister(bob());

        assert_eq!(
            response,
            UserCanisterResponse::Err(UserCanisterError::CanisterNotActive)
        );
    }

    #[test]
    fn test_should_return_not_registered_for_unknown_principal() {
        setup();

        let response = user_canister(bob());

        assert_eq!(
            response,
            UserCanisterResponse::Err(UserCanisterError::NotRegistered)
        );
    }
}
