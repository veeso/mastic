//! Canister settings.

use candid::Principal;
use db_utils::settings::{Settings, SettingsError};
use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms_api::prelude::Value;

use crate::error::{CanisterError, CanisterResult};
use crate::schema::Schema;

/// Setting key for the federation canister principal.
const SETTING_FEDERATION_CANISTER: u16 = 0;
/// Setting key for the owner principal.
const SETTING_OWNER_PRINCIPAL: u16 = 1;
/// Setting key for the user's public key PEM.
const SETTING_USER_PUBLIC_KEY_PEM: u16 = 2;
/// Setting key for the user's private key PEM.
const SETTING_USER_PRIVATE_KEY_PEM: u16 = 3;

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

/// Gets the user's public key in PEM format.
pub fn get_user_public_key_pem() -> CanisterResult<Option<String>> {
    DBMS_CONTEXT
        .with(|ctx| {
            Settings::get_settings_value(ctx, Schema, SETTING_USER_PUBLIC_KEY_PEM, |value| {
                if let Value::Text(text) = value {
                    Ok(text.0.clone())
                } else {
                    Err(SettingsError::BadConfig)
                }
            })
        })
        .map_err(CanisterError::from)
}

/// Sets the user's public key in PEM format.
pub fn set_user_public_key_pem(public_key_pem: String) -> CanisterResult<()> {
    DBMS_CONTEXT
        .with(|ctx| {
            Settings::set_config_key(
                ctx,
                Schema,
                SETTING_USER_PUBLIC_KEY_PEM,
                Value::Text(public_key_pem.into()),
            )
        })
        .map_err(CanisterError::from)
}

/// Gets the user's private key in PEM format.
pub fn get_user_private_key_pem() -> CanisterResult<Option<String>> {
    DBMS_CONTEXT
        .with(|ctx| {
            Settings::get_settings_value(ctx, Schema, SETTING_USER_PRIVATE_KEY_PEM, |value| {
                if let Value::Text(text) = value {
                    Ok(text.0.clone())
                } else {
                    Err(SettingsError::BadConfig)
                }
            })
        })
        .map_err(CanisterError::from)
}

/// Sets the user's private key in PEM format.
pub fn set_user_private_key_pem(private_key_pem: String) -> CanisterResult<()> {
    DBMS_CONTEXT
        .with(|ctx| {
            Settings::set_config_key(
                ctx,
                Schema,
                SETTING_USER_PRIVATE_KEY_PEM,
                Value::Text(private_key_pem.into()),
            )
        })
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

    #[test]
    fn test_should_set_and_get_user_public_key_pem() {
        setup();

        let pem = "-----BEGIN PUBLIC KEY-----\ntest\n-----END PUBLIC KEY-----".to_string();
        set_user_public_key_pem(pem.clone()).expect("should set public key pem");

        assert_eq!(
            get_user_public_key_pem()
                .expect("should read public key pem")
                .expect("should be Some"),
            pem
        );
    }

    #[test]
    fn test_should_return_none_for_unset_public_key_pem() {
        setup();

        assert!(
            get_user_public_key_pem()
                .expect("should read public key pem")
                .is_none(),
            "public key pem should be None when not set"
        );
    }

    #[test]
    fn test_should_overwrite_user_public_key_pem() {
        setup();

        let pem1 = "-----BEGIN PUBLIC KEY-----\nfirst\n-----END PUBLIC KEY-----".to_string();
        let pem2 = "-----BEGIN PUBLIC KEY-----\nsecond\n-----END PUBLIC KEY-----".to_string();
        set_user_public_key_pem(pem1).expect("should set first public key pem");
        set_user_public_key_pem(pem2.clone()).expect("should set second public key pem");

        assert_eq!(
            get_user_public_key_pem()
                .expect("should read public key pem")
                .expect("should be Some"),
            pem2
        );
    }

    #[test]
    fn test_should_set_and_get_user_private_key_pem() {
        setup();

        let pem = "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----".to_string();
        set_user_private_key_pem(pem.clone()).expect("should set private key pem");
        assert_eq!(
            get_user_private_key_pem()
                .expect("should read private key pem")
                .expect("should be Some"),
            pem
        );
    }
}
