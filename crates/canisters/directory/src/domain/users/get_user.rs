//! Flow implementation to get a user by their handle.

use db_utils::handle::{HandleSanitizer, HandleValidator};
use did::directory::{GetUser, GetUserError, GetUserResponse};

use crate::domain::users::repository::UserRepository;

/// Retrieves a user's information based on their handle.
///
/// 1. Checks if a user with the given handle exists in the repository.
/// 2. If the handle is invalid (e.g. empty or contains disallowed characters), returns [`GetUserError::InvalidHandle`] wrapped in [`GetUserResponse::Err`].
/// 3. If the user exists, returns their information as [`GetUser`] wrapped in [`GetUserResponse::Ok`].
/// 4. If the user does not exist, returns [`GetUserError::NotFound`] wrapped in [`GetUserResponse::Err`].
/// 5. If any internal error occurs during the retrieval process, returns a descriptive error message wrapped
///    in [`GetUserResponse::Err`] with the variant [`GetUserError::InternalError`].
pub fn get_user(handle: &str) -> GetUserResponse {
    ic_utils::log!("get_user: looking up user with {handle}");

    let handle = HandleSanitizer::sanitize_handle(handle);
    if let Err(err) = HandleValidator::check_handle(&handle) {
        ic_utils::log!("get_user: invalid handle {handle}: {err}");
        return GetUserResponse::Err(GetUserError::InvalidHandle);
    }

    let user_data = match UserRepository::get_user_by_handle(&handle) {
        Ok(Some(user)) => user,
        Ok(None) => {
            ic_utils::log!("get_user: user {handle} is not registered");
            return GetUserResponse::Err(GetUserError::NotFound);
        }
        Err(e) => {
            ic_utils::log!("get_user: internal error querying user {handle}: {e}");
            return GetUserResponse::Err(GetUserError::InternalError(format!(
                "failed to query database: {e}"
            )));
        }
    };

    GetUserResponse::Ok(GetUser {
        handle: user_data.handle.0,
        canister_id: user_data.canister_id.into_opt().map(|c| c.0),
        canister_status: user_data.canister_status.0,
    })
}

#[cfg(test)]
mod tests {

    use did::directory::UserCanisterStatus;

    use super::*;
    use crate::test_utils::{
        bob, rey_canisteryo, setup, setup_registered_user, setup_registered_user_with_canister,
    };

    #[test]
    fn test_should_return_user_without_canister() {
        setup();
        setup_registered_user(bob(), "alice");

        let response = get_user("alice");

        match response {
            GetUserResponse::Ok(user) => {
                assert_eq!(user.handle, "alice");
                assert!(user.canister_id.is_none());
                assert_eq!(user.canister_status, UserCanisterStatus::CreationPending);
            }
            GetUserResponse::Err(e) => panic!("expected Ok, got Err({e:?})"),
        }
    }

    #[test]
    fn test_should_return_user_with_canister() {
        setup();

        let canister_id = rey_canisteryo();
        setup_registered_user_with_canister(bob(), "alice", canister_id);

        let response = get_user("alice");

        match response {
            GetUserResponse::Ok(user) => {
                assert_eq!(user.handle, "alice");
                assert_eq!(user.canister_id, Some(canister_id));
                assert_eq!(user.canister_status, UserCanisterStatus::Active);
            }
            GetUserResponse::Err(e) => panic!("expected Ok, got Err({e:?})"),
        }
    }

    #[test]
    fn test_should_sanitize_handle_before_lookup() {
        setup();
        setup_registered_user(bob(), "alice");

        let response = get_user("  @Alice  ");

        match response {
            GetUserResponse::Ok(user) => {
                assert_eq!(user.handle, "alice");
            }
            GetUserResponse::Err(e) => panic!("expected Ok, got Err({e:?})"),
        }
    }

    #[test]
    fn test_should_return_not_found_for_unknown_handle() {
        setup();

        let response = get_user("nonexistent");

        assert_eq!(response, GetUserResponse::Err(GetUserError::NotFound));
    }

    #[test]
    fn test_should_return_invalid_handle_for_empty_handle() {
        setup();

        let response = get_user("");

        assert_eq!(response, GetUserResponse::Err(GetUserError::InvalidHandle));
    }

    #[test]
    fn test_should_return_invalid_handle_for_special_chars() {
        setup();

        let response = get_user("alice!@#");

        assert_eq!(response, GetUserResponse::Err(GetUserError::InvalidHandle));
    }
}
