//! Canister API

use did::user::UserInstallArgs;
use ic_dbms_canister::prelude::DBMS_CONTEXT;

/// Initializes the canister with the given arguments.
pub fn init(args: UserInstallArgs) {
    let UserInstallArgs::Init {
        owner,
        federation_canister,
    } = args
    else {
        ic_utils::trap!("Invalid initialization arguments");
    };

    // register database schema
    DBMS_CONTEXT.with(|ctx| {
        if let Err(err) = crate::schema::Schema::register_tables(ctx) {
            ic_utils::trap!("Failed to register database schema: {err}");
        }
    });

    // set owner
    if let Err(err) = crate::settings::set_owner_principal(owner) {
        ic_utils::trap!("Failed to set owner principal: {:?}", err);
    }

    // set federation canister
    if let Err(err) = crate::settings::set_federation_canister(federation_canister) {
        ic_utils::trap!("Failed to set federation canister: {:?}", err);
    }
}

/// Post-upgrade function for the canister.
pub fn post_upgrade(args: UserInstallArgs) {
    let UserInstallArgs::Upgrade { .. } = args else {
        ic_utils::trap!("Invalid post-upgrade arguments");
    };
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
        });
    }
}
