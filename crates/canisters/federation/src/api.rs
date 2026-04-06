//! API for the canister

pub mod inspect;

use did::federation::{FederationInstallArgs, RegisterUserArgs, RegisterUserResponse};

/// Initialize the canister with the given arguments
pub fn init(args: FederationInstallArgs) {
    let FederationInstallArgs::Init {
        directory_canister,
        public_url,
    } = args
    else {
        ic_utils::trap!("Invalid install arguments");
    };

    ic_utils::log!(
        "Federation canister initialized with directory canister: {directory_canister} and public URL: {public_url}",
    );

    crate::settings::set_directory_canister(directory_canister);
    ic_utils::log!(
        "Directory canister set to: {}",
        crate::settings::get_directory_canister()
    );

    crate::settings::set_public_url(public_url);
    ic_utils::log!("Public URL set to: {}", crate::settings::get_public_url());
}

/// Post-upgrade function to handle any necessary state migrations or updates after an upgrade
pub fn post_upgrade(args: FederationInstallArgs) {
    let FederationInstallArgs::Upgrade { .. } = args else {
        ic_utils::trap!("Invalid upgrade arguments");
    };
}

/// Register a new user with the given arguments, returning a response indicating success or failure
pub fn register_user(args: RegisterUserArgs) -> RegisterUserResponse {
    let caller = ic_utils::caller();
    if !self::inspect::is_directory_canister(caller) {
        ic_utils::trap!(
            "Unauthorized caller: {caller}. Only the directory canister is allowed to register users.",
        );
    }

    ic_utils::log!(
        "Registering user with ID: {}, handle: {}, canister ID {}",
        args.user_id,
        args.user_handle,
        args.user_canister_id
    );

    crate::directory::insert_user(args.user_id, args.user_handle, args.user_canister_id);

    RegisterUserResponse::Ok
}

#[cfg(test)]
mod tests {

    use candid::Principal;
    use did::federation::{FederationInstallArgs, RegisterUserArgs, RegisterUserResponse};

    use super::*;
    use crate::test_utils::{alice, directory, public_url, setup};

    /// Set up the canister so that the dummy test caller is treated as the
    /// directory canister, allowing `register_user` to pass the authorization
    /// check.
    fn setup_with_caller_as_directory() {
        let caller = ic_utils::caller();
        init(FederationInstallArgs::Init {
            directory_canister: caller,
            public_url: public_url(),
        });
    }

    #[test]
    fn test_should_init_canister() {
        setup();

        assert_eq!(crate::settings::get_directory_canister(), directory());
        assert_eq!(crate::settings::get_public_url(), public_url());
    }

    #[test]
    #[should_panic(expected = "Invalid install arguments")]
    fn test_should_trap_on_init_with_upgrade_args() {
        init(FederationInstallArgs::Upgrade {});
    }

    #[test]
    fn test_should_post_upgrade_with_upgrade_args() {
        setup();
        post_upgrade(FederationInstallArgs::Upgrade {});
    }

    #[test]
    #[should_panic(expected = "Invalid upgrade arguments")]
    fn test_should_trap_on_post_upgrade_with_init_args() {
        setup();
        post_upgrade(FederationInstallArgs::Init {
            directory_canister: directory(),
            public_url: public_url(),
        });
    }

    // M-UNIT-TEST: register_user inserts the user and returns Ok when called
    // by the directory canister.
    #[test]
    fn test_should_register_user() {
        setup_with_caller_as_directory();

        let user_canister =
            Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").expect("valid principal");

        let response = register_user(RegisterUserArgs {
            user_id: alice(),
            user_handle: "alice".to_string(),
            user_canister_id: user_canister,
        });

        assert_eq!(response, RegisterUserResponse::Ok);

        let stored =
            crate::directory::get_user_by_id(&alice()).expect("user should exist after register");
        assert_eq!(stored.user_id, alice());
        assert_eq!(stored.user_handle, "alice");
        assert_eq!(stored.user_canister_id, user_canister);
    }

    // M-UNIT-TEST: register_user traps when the caller is not the directory
    // canister.
    #[test]
    #[should_panic(expected = "Unauthorized caller")]
    fn test_should_trap_on_register_user_with_unauthorized_caller() {
        setup();

        let user_canister =
            Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").expect("valid principal");

        register_user(RegisterUserArgs {
            user_id: alice(),
            user_handle: "alice".to_string(),
            user_canister_id: user_canister,
        });
    }
}
