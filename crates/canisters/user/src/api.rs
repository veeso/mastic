//! Canister API

pub mod inspect;

use did::user::{GetProfileResponse, PublishStatusArgs, PublishStatusResponse, UserInstallArgs};
use ic_dbms_canister::prelude::DBMS_CONTEXT;

/// Initializes the canister with the given arguments.
pub fn init(args: UserInstallArgs) {
    ic_utils::log!("Initializing user canister");

    let UserInstallArgs::Init {
        owner,
        federation_canister,
        handle,
    } = args
    else {
        ic_utils::trap!("Invalid initialization arguments");
    };

    // register database schema
    ic_utils::log!("Registering database schema");
    DBMS_CONTEXT.with(|ctx| {
        if let Err(err) = crate::schema::Schema::register_tables(ctx) {
            ic_utils::trap!("Failed to register database schema: {err}");
        }
    });

    // set owner
    ic_utils::log!("Setting owner principal to {owner}");
    if let Err(err) = crate::settings::set_owner_principal(owner) {
        ic_utils::trap!("Failed to set owner principal: {:?}", err);
    }

    // set federation canister
    ic_utils::log!("Setting federation canister to {federation_canister}");
    if let Err(err) = crate::settings::set_federation_canister(federation_canister) {
        ic_utils::trap!("Failed to set federation canister: {:?}", err);
    }

    // init profile
    ic_utils::log!("Creating user profile with handle {handle}");
    if let Err(err) = crate::domain::profile::create_profile(owner, &handle) {
        ic_utils::trap!("Failed to create user profile: {:?}", err);
    }

    ic_utils::log!("User canister initialized successfully for owner {owner}");
}

/// Post-upgrade function for the canister.
pub fn post_upgrade(args: UserInstallArgs) {
    ic_utils::log!("Post-upgrade user canister");

    let UserInstallArgs::Upgrade { .. } = args else {
        ic_utils::trap!("Invalid post-upgrade arguments");
    };

    ic_utils::log!("User canister post-upgrade completed successfully");
}

/// Gets the user profile.
pub fn get_profile() -> GetProfileResponse {
    crate::domain::profile::get_profile()
}

/// Publishes a new status.
pub async fn publish_status(args: PublishStatusArgs) -> PublishStatusResponse {
    if !inspect::is_owner(ic_utils::caller()) {
        ic_utils::trap!("Only the owner can publish status updates");
    }

    crate::domain::status::publish_status(args).await
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::{admin, federation, setup};

    #[test]
    fn test_should_init_canister() {
        setup();

        assert_eq!(
            crate::settings::get_owner_principal().expect("should read owner principal"),
            admin()
        );
        assert_eq!(
            crate::settings::get_federation_canister().expect("should read federation canister"),
            federation()
        );
    }

    #[test]
    #[should_panic(expected = "Invalid initialization arguments")]
    fn test_should_trap_on_init_with_upgrade_args() {
        init(UserInstallArgs::Upgrade {});
    }

    #[test]
    fn test_should_post_upgrade_with_upgrade_args() {
        setup();
        post_upgrade(UserInstallArgs::Upgrade {});
    }

    #[test]
    #[should_panic(expected = "Invalid post-upgrade arguments")]
    fn test_should_trap_on_post_upgrade_with_init_args() {
        setup();
        post_upgrade(UserInstallArgs::Init {
            owner: admin(),
            federation_canister: federation(),
            handle: "rey_canisteryo".to_string(),
        });
    }

    #[test]
    fn test_should_init_canister_with_profile() {
        setup();

        let response = get_profile();

        let GetProfileResponse::Ok(profile) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(profile.handle, "rey_canisteryo");
        assert!(profile.display_name.is_none());
        assert!(profile.bio.is_none());
        assert!(profile.avatar.is_none());
        assert!(profile.header.is_none());
    }
}
