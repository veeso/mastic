//! API for the canister

use did::federation::FederationInstallArgs;

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

    ic_utils::log!("Setting initial configuration...");
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

#[cfg(test)]
mod tests {

    use did::federation::FederationInstallArgs;

    use super::*;
    use crate::test_utils::{directory, public_url, setup};

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
}
