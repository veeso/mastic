//! User Sign up flow.

use candid::Principal;
use db_utils::handle::{HandleSanitizer, HandleValidator};
use did::directory::{
    RetrySignUpError, RetrySignUpResponse, SignUpError, SignUpRequest, SignUpResponse,
};

use crate::domain::users::repository::UserRepository;

mod state;

/// Starts the sign up process for a user by initializing their
/// state in the `USER_SIGN_UP_STATES` thread-local storage.
///
/// The flow for signin up is as follows:
///
/// 0. Check if the caller's principal is anonymous, if it is, return [`SignUpError::AnonymousPrincipal`].
/// 1. Check whether there is already a user with the given `user_id` in the database, if there is, return [`SignUpError::AlreadyRegistered`].
/// 2. Sanitize and validate the handle. See `handles.md` document for specs.
/// 3. Check if the handle is already taken by another user in the database, if it is, return [`SignUpError::HandleTaken`].
/// 4. Insert the new user in the database with the canister status set to [`did::directory::UserCanisterStatus::CreationPending`].
/// 5. Spawn the state machine that will drive the user canister creation process, and return [`SignUpResponse::Ok`].
///
/// Any internal error is returned as [`SignUpError::InternalError`] with a message describing the error.
pub fn sign_up(user_id: Principal, request: SignUpRequest) -> SignUpResponse {
    ic_utils::log!(
        "sign_up: starting for user {user_id} with handle {:?}",
        request.handle
    );

    // 0. Check if the caller's principal is anonymous, if it is, return [`SignUpError::AnonymousPrincipal`].
    if user_id == Principal::anonymous() {
        ic_utils::log!("sign_up: rejected anonymous principal");
        return SignUpResponse::Err(SignUpError::AnonymousPrincipal);
    }

    // 1. Check whether there is already a user with the given `user_id` in the database, if there is, return [`SignUpError::AlreadyRegistered`].
    match UserRepository::get_user_by_principal(user_id) {
        Err(err) => {
            ic_utils::log!("sign_up: internal error checking principal {user_id}: {err}");
            return SignUpResponse::Err(SignUpError::InternalError(format!(
                "Failed to check existing user by principal: {err}"
            )));
        }
        Ok(Some(_)) => {
            ic_utils::log!("sign_up: user {user_id} is already registered");
            return SignUpResponse::Err(SignUpError::AlreadyRegistered);
        }
        Ok(None) => (),
    };

    // 2. Sanitize and validate the handle. See `handles.md` document for specs.
    let handle = HandleSanitizer::sanitize_handle(&request.handle);
    ic_utils::log!(
        "sign_up: sanitized handle {:?} -> {handle:?}",
        request.handle
    );
    if HandleValidator::check_handle(&handle).is_err() {
        ic_utils::log!("sign_up: handle {handle:?} is invalid");
        return SignUpResponse::Err(SignUpError::InvalidHandle);
    }

    // 3. Check if the handle is already taken by another user in the database, if it is, return [`SignUpError::HandleTaken`].
    match UserRepository::get_user_by_handle(&handle) {
        Err(err) => {
            ic_utils::log!("sign_up: internal error checking handle {handle:?}: {err}");
            return SignUpResponse::Err(SignUpError::InternalError(format!(
                "Failed to check existing user by handle: {err}"
            )));
        }
        Ok(Some(_)) => {
            ic_utils::log!("sign_up: handle {handle:?} is already taken");
            return SignUpResponse::Err(SignUpError::HandleTaken);
        }
        Ok(None) => (),
    };

    // 4. Insert the new user in the database with the canister status set to [`did::directory::UserCanisterStatus::CreationPending`].
    if let Err(err) = UserRepository::sign_up(user_id, handle.clone()) {
        ic_utils::log!("sign_up: failed to insert user {user_id} in the database: {err}");
        return SignUpResponse::Err(SignUpError::InternalError(format!(
            "Failed to insert new user in the database: {err}"
        )));
    }

    // 5. Spawn the state machine that will drive the user canister creation process, and return [`SignUpResponse::Ok`].
    ic_utils::log!(
        "sign_up: user {user_id} registered with handle {handle:?}, starting canister creation"
    );
    start_sign_up_state_machine(user_id, handle);

    SignUpResponse::Ok
}

/// Retry canister creation for the user that called this method.
///
/// This is used in case the canister creation failed during the sign up process,
/// allowing the user to retry the canister creation without having to go through the whole sign up process again.
///
/// The flow for retrying sign up is as follows:
///
/// 1. Check whether there is a user with the given `user_id` in the database, if there isn't, return [`RetrySignUpError::NotRegistered`].
/// 2. Check if the user's canister is in a failed state, if it isn't, return [`RetrySignUpError::CanisterNotInFailedState`].
/// 3. Update the user's canister status in the database to [`did::directory::UserCanisterStatus::CreationPending`]
/// 4. Spawn the state machine that will drive the user canister creation process, then return [`RetrySignUpResponse::Ok`].
pub fn retry_sign_up(user_id: Principal) -> RetrySignUpResponse {
    ic_utils::log!("retry_sign_up: starting for user {user_id}");

    if user_id == Principal::anonymous() {
        ic_utils::log!("retry_sign_up: rejected anonymous principal");
        return RetrySignUpResponse::Err(RetrySignUpError::NotRegistered);
    }

    // 1. Check whether there is a user with the given `user_id` in the database, if there isn't, return [`RetrySignUpError::NotRegistered`].
    // 2. Check if the user's canister is in a failed state, if it isn't, return [`RetrySignUpError::CanisterNotInFailedState`].
    let handle = match UserRepository::get_user_by_principal(user_id) {
        Err(err) => {
            ic_utils::log!("retry_sign_up: internal error checking principal {user_id}: {err}");
            return RetrySignUpResponse::Err(RetrySignUpError::InternalError(format!(
                "Failed to check existing user by principal: {err}"
            )));
        }
        Ok(None) => {
            ic_utils::log!("retry_sign_up: user {user_id} is not registered");
            return RetrySignUpResponse::Err(RetrySignUpError::NotRegistered);
        }
        Ok(Some(user))
            if user.canister_status.0 != did::directory::UserCanisterStatus::CreationFailed =>
        {
            ic_utils::log!(
                "retry_sign_up: user {user_id} canister is not in failed state (status: {:?})",
                user.canister_status.0
            );
            return RetrySignUpResponse::Err(RetrySignUpError::CanisterNotInFailedState);
        }
        Ok(Some(user)) => user.handle.0,
    };

    // 3. Update the user's canister status in the database to [`did::directory::UserCanisterStatus::CreationPending`]
    if let Err(err) = UserRepository::retry_user_canister_creation(user_id) {
        ic_utils::log!("retry_sign_up: failed to update canister status for user {user_id}: {err}");
        return RetrySignUpResponse::Err(RetrySignUpError::InternalError(format!(
            "Failed to update user canister status in the database: {err}"
        )));
    }

    // 4. Spawn the state machine that will drive the user canister creation process, then return [`RetrySignUpResponse::Ok`].
    ic_utils::log!("retry_sign_up: restarting canister creation for user {user_id}");
    start_sign_up_state_machine(user_id, handle);

    RetrySignUpResponse::Ok
}

/// Returns a second test principal distinct from `bob()` and `rey_canisteryo()`.
#[cfg(test)]
fn alice() -> Principal {
    Principal::from_text("b77ix-eeaaa-aaaaa-qaada-cai").unwrap()
}

fn start_sign_up_state_machine(_user_id: Principal, handle: String) {
    #[cfg(target_family = "wasm")]
    {
        state::SignUpStateMachine::start(
            _user_id,
            handle,
            crate::adapters::management_canister::IcManagementCanisterClient,
        );
    }
    #[cfg(not(target_family = "wasm"))]
    {
        state::SignUpStateMachine::start(
            _user_id,
            handle,
            crate::adapters::management_canister::mock::MockManagementCanisterClient {
                canister_self: Principal::from_text("br5f7-7uaaa-aaaaa-qaaca-cai").unwrap(),
                created_canister_id: Principal::from_text("bkyz2-fmaaa-aaaaa-qaaaq-cai").unwrap(),
            },
        );
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::{bob, setup};

    #[test]
    fn test_should_sign_up_with_valid_handle() {
        setup();

        let response = sign_up(
            bob(),
            SignUpRequest {
                handle: "alice".to_string(),
            },
        );

        assert_eq!(response, SignUpResponse::Ok);
    }

    #[test]
    fn test_should_sanitize_handle_before_validation() {
        setup();

        let response = sign_up(
            bob(),
            SignUpRequest {
                handle: "  @Alice  ".to_string(),
            },
        );

        assert_eq!(response, SignUpResponse::Ok);

        // verify it was stored sanitized
        let user = UserRepository::get_user_by_handle("alice")
            .expect("should query user")
            .expect("user should exist");
        assert_eq!(user.handle.0, "alice");
    }

    #[test]
    fn test_should_reject_anonymous_principal() {
        setup();

        let response = sign_up(
            Principal::anonymous(),
            SignUpRequest {
                handle: "alice".to_string(),
            },
        );

        assert_eq!(
            response,
            SignUpResponse::Err(SignUpError::AnonymousPrincipal)
        );
    }

    #[test]
    fn test_should_reject_duplicate_principal() {
        setup();

        sign_up(
            bob(),
            SignUpRequest {
                handle: "alice".to_string(),
            },
        );

        let response = sign_up(
            bob(),
            SignUpRequest {
                handle: "bob".to_string(),
            },
        );

        assert_eq!(
            response,
            SignUpResponse::Err(SignUpError::AlreadyRegistered)
        );
    }

    #[test]
    fn test_should_reject_duplicate_handle() {
        setup();

        sign_up(
            bob(),
            SignUpRequest {
                handle: "alice".to_string(),
            },
        );

        let response = sign_up(
            alice(),
            SignUpRequest {
                handle: "alice".to_string(),
            },
        );

        assert_eq!(response, SignUpResponse::Err(SignUpError::HandleTaken));
    }

    #[test]
    fn test_should_reject_invalid_handle_with_special_chars() {
        setup();

        let response = sign_up(
            bob(),
            SignUpRequest {
                handle: "alice!@#".to_string(),
            },
        );

        assert_eq!(response, SignUpResponse::Err(SignUpError::InvalidHandle));
    }

    #[test]
    fn test_should_reject_empty_handle() {
        setup();

        let response = sign_up(
            bob(),
            SignUpRequest {
                handle: String::new(),
            },
        );

        assert_eq!(response, SignUpResponse::Err(SignUpError::InvalidHandle));
    }

    #[test]
    fn test_should_reject_handle_exceeding_max_length() {
        setup();

        let response = sign_up(
            bob(),
            SignUpRequest {
                handle: "a".repeat(31),
            },
        );

        assert_eq!(response, SignUpResponse::Err(SignUpError::InvalidHandle));
    }

    #[test]
    fn test_should_reject_reserved_handle() {
        setup();

        let response = sign_up(
            bob(),
            SignUpRequest {
                handle: "admin".to_string(),
            },
        );

        assert_eq!(response, SignUpResponse::Err(SignUpError::InvalidHandle));
    }

    #[test]
    fn test_should_retry_sign_up_after_failure() {
        setup();

        sign_up(
            bob(),
            SignUpRequest {
                handle: "alice".to_string(),
            },
        );

        // simulate canister creation failure
        UserRepository::set_failed_user_canister_create(bob())
            .expect("should set canister creation failed");

        let response = retry_sign_up(bob());

        assert_eq!(response, RetrySignUpResponse::Ok);
    }

    #[test]
    fn test_should_reject_retry_for_anonymous_principal() {
        setup();

        let response = retry_sign_up(Principal::anonymous());

        assert_eq!(
            response,
            RetrySignUpResponse::Err(RetrySignUpError::NotRegistered)
        );
    }

    #[test]
    fn test_should_reject_retry_for_unregistered_user() {
        setup();

        let response = retry_sign_up(bob());

        assert_eq!(
            response,
            RetrySignUpResponse::Err(RetrySignUpError::NotRegistered)
        );
    }

    #[test]
    fn test_should_reject_retry_when_canister_not_in_failed_state() {
        setup();

        sign_up(
            bob(),
            SignUpRequest {
                handle: "alice".to_string(),
            },
        );

        // canister is in CreationPending, not CreationFailed
        let response = retry_sign_up(bob());

        assert_eq!(
            response,
            RetrySignUpResponse::Err(RetrySignUpError::CanisterNotInFailedState)
        );
    }
}
