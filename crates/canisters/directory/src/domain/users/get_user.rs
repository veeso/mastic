//! Flow implementation to get a user by handle or principal.

use candid::Principal;
use db_utils::handle::{HandleSanitizer, HandleValidator};
use did::directory::{GetUser, GetUserArgs, GetUserError, GetUserResponse};

use crate::domain::users::repository::UserRepository;
use crate::error::CanisterResult;
use crate::schema::User;

/// Retrieves a user's information by handle or principal.
///
/// 1. Resolves the user via the appropriate lookup (handle or principal).
/// 2. If the handle variant is used and the handle is invalid, returns [`GetUserError::InvalidHandle`].
/// 3. If the user exists, returns their information as [`GetUser`] wrapped in [`GetUserResponse::Ok`].
/// 4. If the user does not exist, returns [`GetUserError::NotFound`].
/// 5. If any internal error occurs, returns [`GetUserError::InternalError`].
pub fn get_user(args: GetUserArgs) -> GetUserResponse {
    let result = match args {
        GetUserArgs::Handle(ref handle) => lookup_by_handle(handle),
        GetUserArgs::Principal(principal) => lookup_by_principal(principal),
    };

    match result {
        Ok(user_data) => GetUserResponse::Ok(GetUser {
            handle: user_data.handle.0,
            canister_id: user_data.canister_id.into_opt().map(|c| c.0),
            canister_status: user_data.canister_status.0,
        }),
        Err(e) => GetUserResponse::Err(e),
    }
}

/// Look up a user by handle, sanitizing and validating the input first.
fn lookup_by_handle(handle: &str) -> Result<User, GetUserError> {
    ic_utils::log!("get_user: looking up user by handle {handle}");

    let handle = HandleSanitizer::sanitize_handle(handle);
    if let Err(err) = HandleValidator::check_handle(&handle) {
        ic_utils::log!("get_user: invalid handle {handle}: {err}");
        return Err(GetUserError::InvalidHandle);
    }

    resolve_user(
        UserRepository::oneshot().get_user_by_handle(&handle),
        &handle,
    )
}

/// Look up a user by their IC principal.
fn lookup_by_principal(principal: Principal) -> Result<User, GetUserError> {
    ic_utils::log!("get_user: looking up user by principal {principal}");

    resolve_user(
        UserRepository::oneshot().get_user_by_principal(principal),
        &principal.to_string(),
    )
}

/// Shared resolution logic: maps a repository result to either a [`User`] or a [`GetUserError`].
fn resolve_user(
    result: CanisterResult<Option<User>>,
    identifier: &str,
) -> Result<User, GetUserError> {
    match result {
        Ok(Some(user)) => Ok(user),
        Ok(None) => {
            ic_utils::log!("get_user: user {identifier} is not registered");
            Err(GetUserError::NotFound)
        }
        Err(e) => {
            ic_utils::log!("get_user: internal error querying user {identifier}: {e}");
            Err(GetUserError::InternalError(format!(
                "failed to query database: {e}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {

    use did::directory::UserCanisterStatus;

    use super::*;
    use crate::test_utils::{
        bob, rey_canisteryo, setup, setup_registered_user, setup_registered_user_with_canister,
    };

    #[test]
    fn test_should_return_user_without_canister_by_handle() {
        setup();
        setup_registered_user(bob(), "alice");

        let response = get_user(GetUserArgs::Handle("alice".to_string()));

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
    fn test_should_return_user_with_canister_by_handle() {
        setup();

        let canister_id = rey_canisteryo();
        setup_registered_user_with_canister(bob(), "alice", canister_id);

        let response = get_user(GetUserArgs::Handle("alice".to_string()));

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

        let response = get_user(GetUserArgs::Handle("  @Alice  ".to_string()));

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

        let response = get_user(GetUserArgs::Handle("nonexistent".to_string()));

        assert_eq!(response, GetUserResponse::Err(GetUserError::NotFound));
    }

    #[test]
    fn test_should_return_invalid_handle_for_empty_handle() {
        setup();

        let response = get_user(GetUserArgs::Handle(String::new()));

        assert_eq!(response, GetUserResponse::Err(GetUserError::InvalidHandle));
    }

    #[test]
    fn test_should_return_invalid_handle_for_special_chars() {
        setup();

        let response = get_user(GetUserArgs::Handle("alice!@#".to_string()));

        assert_eq!(response, GetUserResponse::Err(GetUserError::InvalidHandle));
    }

    #[test]
    fn test_should_return_user_by_principal() {
        setup();
        setup_registered_user(bob(), "alice");

        let response = get_user(GetUserArgs::Principal(bob()));

        match response {
            GetUserResponse::Ok(user) => {
                assert_eq!(user.handle, "alice");
            }
            GetUserResponse::Err(e) => panic!("expected Ok, got Err({e:?})"),
        }
    }

    #[test]
    fn test_should_return_not_found_for_unknown_principal() {
        setup();

        let response = get_user(GetUserArgs::Principal(bob()));

        assert_eq!(response, GetUserResponse::Err(GetUserError::NotFound));
    }
}
