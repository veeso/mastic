//! Canister implementation

use did::directory::{DirectoryInstallArgs, RetrySignUpResponse, SignUpRequest, SignUpResponse};
use ic_dbms_canister::prelude::DBMS_CONTEXT;

/// Initializes the canister.
pub fn init(args: DirectoryInstallArgs) {
    let DirectoryInstallArgs::Init {
        initial_moderator,
        federation_canister,
    } = args
    else {
        ic_utils::trap!("Invalid initialization arguments");
    };

    DBMS_CONTEXT.with(|ctx| {
        if let Err(err) = crate::schema::Schema::register_tables(ctx) {
            ic_utils::trap!("Failed to register database schema: {err}");
        }
    });

    if let Err(err) = crate::settings::set_federation_canister(federation_canister) {
        ic_utils::trap!("Failed to set federation canister: {err}");
    }

    if let Err(err) = crate::domain::moderators::add_moderator(initial_moderator) {
        ic_utils::trap!("Failed to add initial moderator: {err}");
    }
}

/// Post-upgrade function for the canister.
pub fn post_upgrade(args: DirectoryInstallArgs) {
    let DirectoryInstallArgs::Upgrade { .. } = args else {
        ic_utils::trap!("Invalid post-upgrade arguments");
    };
}

/// Handles the `sign_up` method call to register a new user in the directory, creating a User Canister
pub fn sign_up(request: SignUpRequest) -> SignUpResponse {
    let caller = ic_cdk::api::msg_caller();

    crate::domain::users::sign_up(caller, request)
}

/// Retry canister creation for the user that called this method.
/// This is used in case the canister creation failed during the sign up process
pub fn retry_sign_up() -> RetrySignUpResponse {
    let caller = ic_cdk::api::msg_caller();

    crate::domain::users::retry_sign_up(caller)
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
        });
    }
}
