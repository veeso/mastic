//! Canister settings.

use candid::Principal;
use db_utils::settings::Settings;
use ic_dbms_canister::prelude::DBMS_CONTEXT;

use crate::error::{CanisterError, CanisterResult};
use crate::schema::Schema;

/// Setting key for the federation canister principal.
const SETTING_FEDERATION_CANISTER: u16 = 0;

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
        .map_err(CanisterError::Settings)
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
        .map_err(CanisterError::Settings)
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
}
