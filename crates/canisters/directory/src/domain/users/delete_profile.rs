//! Domain logic for deleting a user's profile and associated User Canister.

mod state;

use candid::Principal;
use did::directory::{
    DeleteProfileError, DeleteProfileResponse, RetryDeleteProfileError, RetryDeleteProfileResponse,
    UserCanisterStatus,
};

use crate::domain::tombstone::TombstoneRepository;
use crate::domain::users::repository::UserRepository;

/// Handles the `delete_profile` method call to delete the caller's profile and User Canister.
///
/// Flow:
/// 0. Reject anonymous principal.
/// 1. Look up the user in the database.
/// 2. Validate canister status: must be `Active` (not mid-creation or mid-deletion).
/// 3. Insert a tombstone for the handle to prevent immediate reuse.
/// 4. Mark the user `DeletionPending` in the directory.
/// 5. Spawn the in-memory state machine that drives activity dispatch, canister
///    stop/delete, and final row removal.
pub fn delete_profile(caller: Principal) -> DeleteProfileResponse {
    ic_utils::log!("delete_profile called by {caller}");

    if caller == Principal::anonymous() {
        ic_utils::log!("delete_profile: rejected anonymous principal");
        return DeleteProfileResponse::Err(DeleteProfileError::AnonymousPrincipal);
    }

    let user = match UserRepository::get_user_by_principal(caller) {
        Ok(Some(user)) => user,
        Ok(None) => {
            ic_utils::log!("delete_profile: user {caller} not registered");
            return DeleteProfileResponse::Err(DeleteProfileError::NotRegistered);
        }
        Err(err) => {
            ic_utils::log!("delete_profile: failed to look up user {caller}: {err}");
            return DeleteProfileResponse::Err(DeleteProfileError::Internal(err.to_string()));
        }
    };

    let canister_id = match (user.canister_status.0, user.canister_id.into_opt()) {
        (UserCanisterStatus::Active, Some(cid)) => cid.0,
        (UserCanisterStatus::DeletionPending, _) => {
            ic_utils::log!("delete_profile: deletion already in progress for {caller}");
            return DeleteProfileResponse::Err(DeleteProfileError::DeletionAlreadyInProgress);
        }
        (status, _) => {
            ic_utils::log!(
                "delete_profile: user {caller} canister not active (status: {status:?})"
            );
            return DeleteProfileResponse::Err(DeleteProfileError::CanisterNotActive);
        }
    };

    let handle = user.handle.0.clone();

    if let Err(err) = TombstoneRepository::insert_or_update(caller, handle.clone()) {
        ic_utils::log!("delete_profile: failed to insert tombstone for {caller}: {err}");
        return DeleteProfileResponse::Err(DeleteProfileError::Internal(err.to_string()));
    }

    if let Err(err) = UserRepository::mark_user_for_deletion(caller) {
        ic_utils::log!("delete_profile: failed to mark user {caller} for deletion: {err}");
        return DeleteProfileResponse::Err(DeleteProfileError::Internal(err.to_string()));
    }

    start_delete_profile_state_machine(caller, canister_id);

    DeleteProfileResponse::Ok
}

/// Retry an interrupted `delete_profile` flow for the caller.
///
/// Allowed only when the caller is currently in `DeletionPending` state. Tombstone
/// and repository state stay as-is; the state machine restarts from `EmitActivities`.
/// All management-canister calls are idempotent, so re-running previously completed
/// steps is safe.
pub fn retry_delete_profile(caller: Principal) -> RetryDeleteProfileResponse {
    ic_utils::log!("retry_delete_profile called by {caller}");

    if caller == Principal::anonymous() {
        ic_utils::log!("retry_delete_profile: rejected anonymous principal");
        return RetryDeleteProfileResponse::Err(RetryDeleteProfileError::NotRegistered);
    }

    let user = match UserRepository::get_user_by_principal(caller) {
        Ok(Some(user)) => user,
        Ok(None) => {
            ic_utils::log!("retry_delete_profile: user {caller} not registered");
            return RetryDeleteProfileResponse::Err(RetryDeleteProfileError::NotRegistered);
        }
        Err(err) => {
            ic_utils::log!("retry_delete_profile: failed to look up user {caller}: {err}");
            return RetryDeleteProfileResponse::Err(RetryDeleteProfileError::Internal(
                err.to_string(),
            ));
        }
    };

    if user.canister_status.0 != UserCanisterStatus::DeletionPending {
        ic_utils::log!(
            "retry_delete_profile: user {caller} not in deletion-pending state (status: {:?})",
            user.canister_status.0
        );
        return RetryDeleteProfileResponse::Err(
            RetryDeleteProfileError::CanisterNotInDeletionState,
        );
    }

    let Some(canister_id) = user.canister_id.into_opt() else {
        ic_utils::log!("retry_delete_profile: user {caller} has no canister_id set");
        return RetryDeleteProfileResponse::Err(RetryDeleteProfileError::Internal(
            "user has no canister_id".to_string(),
        ));
    };

    start_delete_profile_state_machine(caller, canister_id.0);

    RetryDeleteProfileResponse::Ok
}

fn start_delete_profile_state_machine(_user_id: Principal, _canister_id: Principal) {
    #[cfg(target_family = "wasm")]
    {
        state::DeleteProfileStateMachine::start(
            _user_id,
            _canister_id,
            crate::adapters::management_canister::IcManagementCanisterClient,
            crate::adapters::user_canister::IcUserCanisterClient,
        );
    }
    #[cfg(not(target_family = "wasm"))]
    {
        state::DeleteProfileStateMachine::start(
            _user_id,
            _canister_id,
            crate::adapters::management_canister::mock::MockManagementCanisterClient {
                canister_self: Principal::management_canister(),
                created_canister_id: _canister_id,
            },
            crate::adapters::user_canister::mock::MockUserCanisterClient,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{bob, rey_canisteryo, setup, setup_registered_user_with_canister};

    #[test]
    fn test_should_reject_anonymous() {
        setup();

        let response = delete_profile(Principal::anonymous());

        assert_eq!(
            response,
            DeleteProfileResponse::Err(DeleteProfileError::AnonymousPrincipal)
        );
    }

    #[test]
    fn test_should_reject_unregistered() {
        setup();

        let response = delete_profile(bob());

        assert_eq!(
            response,
            DeleteProfileResponse::Err(DeleteProfileError::NotRegistered)
        );
    }

    #[test]
    fn test_should_reject_when_canister_not_active() {
        setup();
        UserRepository::sign_up(bob(), "bob".to_string()).expect("should sign up");

        let response = delete_profile(bob());

        assert_eq!(
            response,
            DeleteProfileResponse::Err(DeleteProfileError::CanisterNotActive)
        );
    }

    #[test]
    fn test_should_mark_user_for_deletion_and_insert_tombstone() {
        setup();
        setup_registered_user_with_canister(bob(), "bob", rey_canisteryo());

        let response = delete_profile(bob());
        assert_eq!(response, DeleteProfileResponse::Ok);

        let user = UserRepository::get_user_by_principal(bob())
            .expect("should query user")
            .expect("user should still exist pre-commit");
        assert_eq!(user.canister_status.0, UserCanisterStatus::DeletionPending);

        assert!(TombstoneRepository::is_tombstoned("bob").expect("should query tombstone"));
    }

    #[test]
    fn test_should_reject_double_delete() {
        setup();
        setup_registered_user_with_canister(bob(), "bob", rey_canisteryo());

        delete_profile(bob());
        let response = delete_profile(bob());

        assert_eq!(
            response,
            DeleteProfileResponse::Err(DeleteProfileError::DeletionAlreadyInProgress)
        );
    }

    #[test]
    fn test_retry_should_reject_anonymous() {
        setup();

        let response = retry_delete_profile(Principal::anonymous());

        assert_eq!(
            response,
            RetryDeleteProfileResponse::Err(RetryDeleteProfileError::NotRegistered)
        );
    }

    #[test]
    fn test_retry_should_reject_unregistered() {
        setup();

        let response = retry_delete_profile(bob());

        assert_eq!(
            response,
            RetryDeleteProfileResponse::Err(RetryDeleteProfileError::NotRegistered)
        );
    }

    #[test]
    fn test_retry_should_reject_when_not_in_deletion_state() {
        setup();
        setup_registered_user_with_canister(bob(), "bob", rey_canisteryo());

        let response = retry_delete_profile(bob());

        assert_eq!(
            response,
            RetryDeleteProfileResponse::Err(RetryDeleteProfileError::CanisterNotInDeletionState)
        );
    }

    #[test]
    fn test_retry_should_succeed_after_delete_marks_user() {
        setup();
        setup_registered_user_with_canister(bob(), "bob", rey_canisteryo());

        delete_profile(bob());

        let response = retry_delete_profile(bob());
        assert_eq!(response, RetryDeleteProfileResponse::Ok);
    }
}
