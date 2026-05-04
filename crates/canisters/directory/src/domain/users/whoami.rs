//! Whoami flow implementation

use candid::Principal;
use db_utils::repository::Repository;
use did::directory::{WhoAmI, WhoAmIError, WhoAmIResponse};

use crate::repository::users::UserRepository;

/// Returns the user information for the caller.
///
/// - If the caller is registered, returns their handle, user canister (if any), and canister status as [`WhoAmI`].
/// - If the caller is not registered, returns a [`WhoAmIError::NotRegistered`] error.
/// - If an internal error occurs while retrieving user information, returns an [`WhoAmIError::InternalError`] error.
pub fn whoami(caller: Principal) -> WhoAmIResponse {
    ic_utils::log!("whoami: looking up user {caller}");

    let user = match UserRepository::oneshot().get_user_by_principal(caller) {
        Ok(Some(user)) => user,
        Ok(None) => {
            ic_utils::log!("whoami: user {caller} is not registered");
            return WhoAmIResponse::Err(WhoAmIError::NotRegistered);
        }
        Err(e) => {
            ic_utils::log!("whoami: internal error querying user {caller}: {e}");
            return WhoAmIResponse::Err(WhoAmIError::InternalError(format!(
                "failed to query database: {e}"
            )));
        }
    };

    ic_utils::log!(
        "whoami: found user {caller} with handle {:?}, canister status {:?}",
        user.handle.0,
        user.canister_status.0
    );

    WhoAmIResponse::Ok(WhoAmI {
        handle: user.handle.0,
        user_canister: user.canister_id.into_opt().map(|p| p.0),
        canister_status: user.canister_status.into(),
    })
}

#[cfg(test)]
mod tests {

    use did::directory::UserCanisterStatus;

    use super::*;
    use crate::test_utils::{
        bob, rey_canisteryo, setup, setup_registered_user, setup_registered_user_with_canister,
    };

    #[test]
    fn test_should_return_user_info_for_registered_user_without_canister() {
        setup();
        setup_registered_user(bob(), "alice");

        let response = whoami(bob());

        match response {
            WhoAmIResponse::Ok(info) => {
                assert_eq!(info.handle, "alice");
                assert!(info.user_canister.is_none());
                assert_eq!(info.canister_status, UserCanisterStatus::CreationPending);
            }
            WhoAmIResponse::Err(e) => panic!("expected Ok, got Err({e:?})"),
        }
    }

    #[test]
    fn test_should_return_user_info_for_registered_user_with_canister() {
        setup();

        let canister_id = rey_canisteryo();
        setup_registered_user_with_canister(bob(), "alice", canister_id);

        let response = whoami(bob());

        match response {
            WhoAmIResponse::Ok(info) => {
                assert_eq!(info.handle, "alice");
                assert_eq!(info.user_canister, Some(canister_id));
                assert_eq!(info.canister_status, UserCanisterStatus::Active);
            }
            WhoAmIResponse::Err(e) => panic!("expected Ok, got Err({e:?})"),
        }
    }

    #[test]
    fn test_should_return_not_registered_for_unknown_principal() {
        setup();

        let response = whoami(bob());

        assert_eq!(response, WhoAmIResponse::Err(WhoAmIError::NotRegistered));
    }
}
