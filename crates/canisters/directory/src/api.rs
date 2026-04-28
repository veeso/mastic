//! Canister implementation

use candid::Principal;
use did::directory::{
    DeleteProfileResponse, DirectoryInstallArgs, GetUserArgs, GetUserResponse,
    RetryDeleteProfileResponse, RetrySignUpResponse, SignUpRequest, SignUpResponse,
    UserCanisterResponse, WhoAmIResponse,
};
use ic_dbms_canister::prelude::DBMS_CONTEXT;

use crate::schema::Schema;

/// Initializes the canister.
pub fn init(args: DirectoryInstallArgs) {
    ic_utils::log!("Initializing directory canister");

    let DirectoryInstallArgs::Init {
        initial_moderator,
        federation_canister,
        public_url,
    } = args
    else {
        ic_utils::trap!("Invalid initialization arguments");
    };

    ic_utils::log!("Registering database schema");
    DBMS_CONTEXT.with(|ctx| {
        if let Err(err) = crate::schema::Schema::register_tables(ctx) {
            ic_utils::trap!("Failed to register database schema: {err}");
        }
    });

    ic_utils::log!("Setting federation canister to {federation_canister}");
    if let Err(err) = crate::settings::set_federation_canister(federation_canister) {
        ic_utils::trap!("Failed to set federation canister: {err}");
    }

    ic_utils::log!("Setting public URL to {public_url}");
    if let Err(err) = crate::settings::set_public_url(public_url) {
        ic_utils::trap!("Failed to set public URL: {err}");
    }

    ic_utils::log!("Adding initial moderator {initial_moderator}");
    if let Err(err) = crate::domain::moderators::add_moderator(initial_moderator) {
        ic_utils::trap!("Failed to add initial moderator: {err}");
    }

    ic_utils::log!("Directory canister initialized successfully");
}

/// Post-upgrade function for the canister.
pub fn post_upgrade(args: DirectoryInstallArgs) {
    ic_utils::log!("Post-upgrade directory canister");

    let DirectoryInstallArgs::Upgrade { .. } = args else {
        ic_utils::trap!("Invalid post-upgrade arguments");
    };

    db_utils::migration::run_post_upgrade_migration(&DBMS_CONTEXT, Schema);

    ic_utils::log!("Directory canister post-upgrade completed successfully");
}

/// Handles the `delete_profile` method call to delete the caller's profile and User Canister.
pub fn delete_profile() -> DeleteProfileResponse {
    let caller = ic_utils::caller();

    crate::domain::users::delete_profile(caller)
}

/// Handles the `get_user` method call to retrieve user information by handle or principal.
pub fn get_user(args: GetUserArgs) -> GetUserResponse {
    crate::domain::users::get_user(args)
}

/// Retry an interrupted profile deletion for the caller. Callable only when the
/// caller's canister status is `DeletionPending`.
pub fn retry_delete_profile() -> RetryDeleteProfileResponse {
    let caller = ic_utils::caller();
    ic_utils::log!("retry_delete_profile called by {caller}");

    crate::domain::users::retry_delete_profile(caller)
}

/// Retry canister creation for the user that called this method.
/// This is used in case the canister creation failed during the sign up process
pub fn retry_sign_up() -> RetrySignUpResponse {
    let caller = ic_utils::caller();
    ic_utils::log!("retry_sign_up called by {caller}");

    crate::domain::users::retry_sign_up(caller)
}

/// Handles the `sign_up` method call to register a new user in the directory, creating a User Canister
pub fn sign_up(request: SignUpRequest) -> SignUpResponse {
    let caller = ic_utils::caller();
    ic_utils::log!(
        "sign_up called by {caller} with handle {:?}",
        request.handle
    );

    crate::domain::users::sign_up(caller, request)
}

/// Handles the `user_canister` method call to retrieve the User Canister ID for the caller.
pub fn user_canister(principal: Option<Principal>) -> UserCanisterResponse {
    let principal = principal.unwrap_or_else(|| {
        ic_utils::log!("user_canister: user not provided");
        ic_utils::caller()
    });
    ic_utils::log!("user_canister called with argument {principal:?}; resolved to {principal}");

    crate::domain::users::user_canister(principal)
}

/// Handles the `whoami` method call to retrieve the user information for the caller.
pub fn whoami() -> WhoAmIResponse {
    let caller = ic_utils::caller();
    ic_utils::log!("whoami called by {caller}");

    crate::domain::users::whoami(caller)
}

#[cfg(test)]
mod tests {

    use did::directory::DirectoryInstallArgs;

    use super::*;
    use crate::test_utils::{admin, federation, setup};

    #[test]
    fn test_should_init_canister() {
        setup();

        assert!(crate::domain::moderators::is_moderator(admin()).expect("should read moderator"));
        assert!(
            !crate::domain::moderators::is_moderator(federation()).expect("should read moderator")
        );
        assert_eq!(
            crate::settings::get_federation_canister().expect("should read federation canister"),
            federation()
        );
    }

    #[test]
    #[should_panic(expected = "Invalid initialization arguments")]
    fn test_should_trap_on_init_with_upgrade_args() {
        init(DirectoryInstallArgs::Upgrade {});
    }

    #[test]
    fn test_should_post_upgrade_with_upgrade_args() {
        setup();
        post_upgrade(DirectoryInstallArgs::Upgrade {});
    }

    #[test]
    #[should_panic(expected = "Invalid post-upgrade arguments")]
    fn test_should_trap_on_post_upgrade_with_init_args() {
        setup();
        post_upgrade(DirectoryInstallArgs::Init {
            initial_moderator: admin(),
            federation_canister: federation(),
            public_url: "https://mastic.social".to_string(),
        });
    }

    #[test]
    fn test_should_sign_up_user() {
        setup();
        let request = SignUpRequest {
            handle: "rey_canisteryo".to_string(),
        };
        let response = sign_up(request);
        assert_eq!(response, SignUpResponse::Ok);
    }

    #[test]
    fn test_should_retry_sign_up() {
        setup();
        let response = retry_sign_up();
        assert_eq!(
            response,
            RetrySignUpResponse::Err(did::directory::RetrySignUpError::NotRegistered)
        );
    }

    #[test]
    fn test_should_whoami_for_registered_user() {
        setup();
        sign_up(SignUpRequest {
            handle: "rey_canisteryo".to_string(),
        });
        let response = whoami();
        match response {
            WhoAmIResponse::Ok(info) => {
                assert_eq!(info.handle, "rey_canisteryo");
                assert_eq!(
                    info.canister_status,
                    did::directory::UserCanisterStatus::CreationPending,
                );
            }
            WhoAmIResponse::Err(e) => panic!("expected Ok, got Err({e:?})"),
        }
    }

    #[test]
    fn test_should_whoami_return_not_registered() {
        setup();
        let response = whoami();
        assert_eq!(
            response,
            WhoAmIResponse::Err(did::directory::WhoAmIError::NotRegistered)
        );
    }

    #[test]
    fn test_should_user_canister_return_canister_for_active_user() {
        setup();

        let canister_id = crate::test_utils::rey_canisteryo();
        crate::test_utils::setup_registered_user_with_canister(
            ic_utils::caller(),
            "alice",
            canister_id,
        );

        let response = user_canister(None);

        assert_eq!(response, UserCanisterResponse::Ok(canister_id));
    }

    #[test]
    fn test_should_user_canister_return_not_active_when_pending() {
        setup();
        crate::test_utils::setup_registered_user(ic_utils::caller(), "alice");

        let response = user_canister(None);

        assert_eq!(
            response,
            UserCanisterResponse::Err(did::directory::UserCanisterError::CanisterNotActive)
        );
    }

    #[test]
    fn test_should_user_canister_return_not_registered() {
        setup();
        let response = user_canister(None);
        assert_eq!(
            response,
            UserCanisterResponse::Err(did::directory::UserCanisterError::NotRegistered)
        );
    }

    #[test]
    fn test_should_user_canister_with_explicit_principal() {
        setup();

        let principal = crate::test_utils::bob();
        let canister_id = crate::test_utils::rey_canisteryo();
        crate::test_utils::setup_registered_user_with_canister(principal, "bob", canister_id);

        let response = user_canister(Some(principal));

        assert_eq!(response, UserCanisterResponse::Ok(canister_id));
    }

    #[test]
    fn test_should_get_user_by_handle() {
        setup();
        crate::test_utils::setup_registered_user(crate::test_utils::bob(), "alice");

        let response = get_user(GetUserArgs::Handle("alice".to_string()));

        match response {
            GetUserResponse::Ok(user) => {
                assert_eq!(user.handle, "alice");
            }
            GetUserResponse::Err(e) => panic!("expected Ok, got Err({e:?})"),
        }
    }

    #[test]
    fn test_should_get_user_return_not_found() {
        setup();

        let response = get_user(GetUserArgs::Handle("nonexistent".to_string()));

        assert_eq!(
            response,
            GetUserResponse::Err(did::directory::GetUserError::NotFound)
        );
    }
}
