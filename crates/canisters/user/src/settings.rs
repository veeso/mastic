//! Canister settings.

use candid::Principal;
use db_utils::settings::Settings;
use ic_dbms_canister::prelude::DBMS_CONTEXT;

use crate::error::{CanisterError, CanisterResult};
use crate::schema::Schema;

/// Setting key for the federation canister principal.
const SETTING_FEDERATION_CANISTER: u16 = 0;
/// Setting key for the owner principal.
const SETTING_OWNER_PRINCIPAL: u16 = 1;
/// Setting key for the public URL.
const SETTING_PUBLIC_URL: u16 = 2;
/// Setting key for the directory canister principal.
const SETTING_DIRECTORY_CANISTER: u16 = 3;

/// Gets the principal of the federation canister.
pub fn get_federation_canister() -> CanisterResult<Principal> {
    DBMS_CONTEXT
        .with(|ctx| {
            Settings::get_required_settings_value(
                ctx,
                Schema,
                SETTING_FEDERATION_CANISTER,
                Settings::get_as_principal,
            )
        })
        .map_err(CanisterError::from)
}

/// Sets the principal of the federation canister.
pub fn set_federation_canister(principal: Principal) -> CanisterResult<()> {
    DBMS_CONTEXT
        .with(|ctx| {
            Settings::set_config_key(
                ctx,
                Schema,
                SETTING_FEDERATION_CANISTER,
                principal.as_slice().to_vec().as_slice(),
            )
        })
        .map_err(CanisterError::from)
}

/// Gets the principal of the owner.
pub fn get_owner_principal() -> CanisterResult<Principal> {
    DBMS_CONTEXT
        .with(|ctx| {
            Settings::get_required_settings_value(
                ctx,
                Schema,
                SETTING_OWNER_PRINCIPAL,
                Settings::get_as_principal,
            )
        })
        .map_err(CanisterError::from)
}

/// Sets the principal of the owner.
pub fn set_owner_principal(principal: Principal) -> CanisterResult<()> {
    DBMS_CONTEXT
        .with(|ctx| {
            Settings::set_config_key(
                ctx,
                Schema,
                SETTING_OWNER_PRINCIPAL,
                principal.as_slice().to_vec().as_slice(),
            )
        })
        .map_err(CanisterError::from)
}

/// Gets the principal of the directory canister.
pub fn get_directory_canister() -> CanisterResult<Principal> {
    DBMS_CONTEXT
        .with(|ctx| {
            Settings::get_required_settings_value(
                ctx,
                Schema,
                SETTING_DIRECTORY_CANISTER,
                Settings::get_as_principal,
            )
        })
        .map_err(CanisterError::from)
}

/// Sets the principal of the directory canister.
pub fn set_directory_canister(principal: Principal) -> CanisterResult<()> {
    DBMS_CONTEXT
        .with(|ctx| {
            Settings::set_config_key(
                ctx,
                Schema,
                SETTING_DIRECTORY_CANISTER,
                principal.as_slice().to_vec().as_slice(),
            )
        })
        .map_err(CanisterError::from)
}

/// Gets the public URL of the Mastic instance.
pub fn get_public_url() -> CanisterResult<String> {
    DBMS_CONTEXT
        .with(|ctx| {
            Settings::get_required_settings_value(
                ctx,
                Schema,
                SETTING_PUBLIC_URL,
                Settings::get_as_string,
            )
        })
        .map_err(CanisterError::from)
}

/// Sets the public URL of the Mastic instance.
pub fn set_public_url(url: String) -> CanisterResult<()> {
    DBMS_CONTEXT
        .with(|ctx| Settings::set_config_key(ctx, Schema, SETTING_PUBLIC_URL, url))
        .map_err(CanisterError::from)
}

#[cfg(test)]
mod tests {

    use candid::Principal;

    use super::*;
    use crate::test_utils::setup;

    #[test]
    fn test_should_overwrite_federation_canister_setting() {
        setup();

        let new_federation =
            Principal::from_text("b77ix-eeaaa-aaaaa-qaada-cai").expect("valid principal");
        set_federation_canister(new_federation).expect("should set federation canister");

        assert_eq!(
            get_federation_canister().expect("should read federation canister after overwrite"),
            new_federation
        );
    }

    #[test]
    fn test_should_overwrite_directory_canister_setting() {
        setup();

        let new_directory =
            Principal::from_text("b77ix-eeaaa-aaaaa-qaada-cai").expect("valid principal");
        set_directory_canister(new_directory).expect("should set directory canister");

        assert_eq!(
            get_directory_canister().expect("should read directory canister after overwrite"),
            new_directory
        );
    }

    #[test]
    fn test_should_overwrite_public_url_setting() {
        setup();

        let new_url = "https://new.mastic.social";
        set_public_url(new_url.to_string()).expect("should set public url");

        assert_eq!(
            get_public_url().expect("should read public url after overwrite"),
            new_url
        );
    }

    #[test]
    fn test_should_overwrite_owner_principal_setting() {
        setup();

        let new_owner =
            Principal::from_text("b77ix-eeaaa-aaaaa-qaada-cai").expect("valid principal");
        set_owner_principal(new_owner).expect("should set owner principal");

        assert_eq!(
            get_owner_principal().expect("should read owner principal after overwrite"),
            new_owner
        );
    }
}
