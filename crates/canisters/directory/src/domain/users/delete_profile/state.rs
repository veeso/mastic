//! State machine for the `delete_profile` domain logic.
//!
//! Mirrors the sign_up state-machine pattern. Only `DeletionPending` is persisted
//! in the directory database; fine-grained progress lives in this in-memory state.
//! Each management-canister step is idempotent so that `retry_delete_profile`
//! can safely restart from the beginning.

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

use candid::Principal;

use crate::adapters::management_canister::ManagementCanister;
use crate::adapters::user_canister::UserCanister;
use crate::domain::users::repository::UserRepository;

thread_local! {
    /// Active delete_profile states, keyed by user principal.
    static USER_DELETE_STATES: RefCell<HashMap<Principal, DeleteStateStep>> = RefCell::new(HashMap::new());
}

/// Minimum interval between two delete-profile operations to spread load.
const OPERATION_INTERVAL: Duration = Duration::from_secs(1);
/// Maximum number of retries for each step before giving up.
const MAX_RETRIES: u8 = 5;

/// A step in the delete_profile process, along with the number of retries attempted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeleteStateStep {
    state: DeleteState,
    retries: u8,
}

/// In-memory state of the delete_profile process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeleteState {
    /// Ask the user canister to emit `Delete(Person)` activities to all followers.
    EmitActivities { canister_id: Principal },
    /// Stop the user canister via the management canister.
    StopCanister { canister_id: Principal },
    /// Delete the user canister via the management canister.
    DeleteCanister { canister_id: Principal },
    /// Remove the directory's user record.
    Commit,
}

/// Result of a single state-machine step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepResult {
    /// Continue processing with updated state.
    Continue(DeleteStateStep),
    /// The delete_profile process is complete; clean up.
    Finished,
}

/// State machine for the user delete_profile process.
///
/// Generic over `M` (management canister) and `U` (user canister) so that
/// both dependencies can be replaced with mocks during unit tests.
pub struct DeleteProfileStateMachine<M, U>
where
    M: ManagementCanister + 'static,
    U: UserCanister + 'static,
{
    /// The user whose profile is being deleted.
    user_id: Principal,
    /// Adapter for management canister calls.
    management: M,
    /// Adapter for user canister calls.
    user_canister: U,
}

impl<M, U> DeleteProfileStateMachine<M, U>
where
    M: ManagementCanister + 'static,
    U: UserCanister + 'static,
{
    /// Start a new delete_profile process for a user.
    pub fn start(user_id: Principal, canister_id: Principal, management: M, user_canister: U) {
        ic_utils::log!("Starting delete_profile for user {user_id} (canister {canister_id})");
        let already_exists = USER_DELETE_STATES.with_borrow(|states| states.contains_key(&user_id));
        if already_exists {
            return;
        }

        USER_DELETE_STATES.with_borrow_mut(|states| {
            states.insert(
                user_id,
                DeleteStateStep {
                    state: DeleteState::EmitActivities { canister_id },
                    retries: 0,
                },
            )
        });

        Self {
            user_id,
            management,
            user_canister,
        }
        .tick();
    }

    /// Tick the state machine.
    fn tick(self) {
        ic_utils::log!("Ticking delete_profile for user {}", self.user_id);
        ic_utils::set_timer(OPERATION_INTERVAL, async move {
            ic_utils::log!("Delete timer fired for user {}", self.user_id);
            self.run().await;
        });
    }

    /// Run a step of the delete_profile process.
    async fn run(self) {
        let Some(current) =
            USER_DELETE_STATES.with_borrow(|states| states.get(&self.user_id).copied())
        else {
            return;
        };

        match self.step(current).await {
            StepResult::Continue(next) => {
                USER_DELETE_STATES.with_borrow_mut(|states| states.insert(self.user_id, next));
                self.tick();
            }
            StepResult::Finished => self.finish(),
        }
    }

    /// Execute a single state-transition step.
    async fn step(&self, current: DeleteStateStep) -> StepResult {
        let DeleteStateStep { state, retries } = current;
        ic_utils::log!(
            "delete_profile step for user {}: state={state:?}, retries={retries}",
            self.user_id,
        );
        let current_discriminant = std::mem::discriminant(&state);

        if retries >= MAX_RETRIES {
            ic_utils::log!(
                "delete_profile for user {} exhausted retries at state {state:?}; giving up",
                self.user_id
            );
            return StepResult::Finished;
        }

        let new_state = match state {
            DeleteState::EmitActivities { canister_id } => self.emit_activities(canister_id).await,
            DeleteState::StopCanister { canister_id } => self.stop_canister(canister_id).await,
            DeleteState::DeleteCanister { canister_id } => self.delete_canister(canister_id).await,
            DeleteState::Commit => match self.commit() {
                Ok(()) => return StepResult::Finished,
                Err(_) => DeleteState::Commit,
            },
        };
        let new_discriminant = std::mem::discriminant(&new_state);

        let next = if current_discriminant == new_discriminant {
            DeleteStateStep {
                state: new_state,
                retries: retries + 1,
            }
        } else {
            DeleteStateStep {
                state: new_state,
                retries: 0,
            }
        };

        StepResult::Continue(next)
    }

    async fn emit_activities(&self, canister_id: Principal) -> DeleteState {
        if self
            .user_canister
            .emit_delete_profile_activity(canister_id)
            .await
            .is_err()
        {
            return DeleteState::EmitActivities { canister_id };
        }

        DeleteState::StopCanister { canister_id }
    }

    async fn stop_canister(&self, canister_id: Principal) -> DeleteState {
        if self.management.stop_canister(canister_id).await.is_err() {
            return DeleteState::StopCanister { canister_id };
        }

        DeleteState::DeleteCanister { canister_id }
    }

    async fn delete_canister(&self, canister_id: Principal) -> DeleteState {
        if self.management.delete_canister(canister_id).await.is_err() {
            return DeleteState::DeleteCanister { canister_id };
        }

        DeleteState::Commit
    }

    fn commit(&self) -> crate::error::CanisterResult<()> {
        UserRepository::remove_user(self.user_id)
    }

    fn finish(&self) {
        ic_utils::log!("delete_profile finished for user {}", self.user_id);
        USER_DELETE_STATES.with_borrow_mut(|states| states.remove(&self.user_id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::management_canister::ManagementCanisterError;
    use crate::adapters::user_canister::UserCanisterError;
    use crate::test_utils::{rey_canisteryo, setup};

    struct TestManagementClient {
        stop_result: Result<(), ManagementCanisterError>,
        delete_result: Result<(), ManagementCanisterError>,
    }

    impl TestManagementClient {
        fn ok() -> Self {
            Self {
                stop_result: Ok(()),
                delete_result: Ok(()),
            }
        }

        fn with_stop_err(mut self) -> Self {
            self.stop_result = Err(ManagementCanisterError::CallFailed("test".to_string()));
            self
        }

        fn with_delete_err(mut self) -> Self {
            self.delete_result = Err(ManagementCanisterError::CallFailed("test".to_string()));
            self
        }
    }

    impl ManagementCanister for TestManagementClient {
        async fn create_canister(
            &self,
            _settings: Option<ic_management_canister_types::CanisterSettings>,
            _cycles: u128,
        ) -> Result<Principal, ManagementCanisterError> {
            unimplemented!()
        }

        async fn install_code(
            &self,
            _canister_id: Principal,
            _wasm_module: &[u8],
            _arg: Vec<u8>,
        ) -> Result<(), ManagementCanisterError> {
            unimplemented!()
        }

        async fn stop_canister(
            &self,
            _canister_id: Principal,
        ) -> Result<(), ManagementCanisterError> {
            self.stop_result.clone()
        }

        async fn delete_canister(
            &self,
            _canister_id: Principal,
        ) -> Result<(), ManagementCanisterError> {
            self.delete_result.clone()
        }

        fn canister_version(&self) -> u64 {
            0
        }

        fn canister_self(&self) -> Principal {
            Principal::management_canister()
        }
    }

    struct TestUserCanisterClient {
        emit_result: Result<(), UserCanisterError>,
    }

    impl TestUserCanisterClient {
        fn ok() -> Self {
            Self {
                emit_result: Ok(()),
            }
        }

        fn with_emit_err(mut self) -> Self {
            self.emit_result = Err(UserCanisterError::CallFailed("test".to_string()));
            self
        }
    }

    impl UserCanister for TestUserCanisterClient {
        async fn emit_delete_profile_activity(
            &self,
            _user_canister_id: Principal,
        ) -> Result<(), UserCanisterError> {
            self.emit_result.clone()
        }
    }

    fn user_canister_id() -> Principal {
        Principal::from_text("b77ix-eeaaa-aaaaa-qaada-cai").unwrap()
    }

    fn machine(
        mgmt: TestManagementClient,
        user: TestUserCanisterClient,
    ) -> DeleteProfileStateMachine<TestManagementClient, TestUserCanisterClient> {
        DeleteProfileStateMachine {
            user_id: rey_canisteryo(),
            management: mgmt,
            user_canister: user,
        }
    }

    #[tokio::test]
    async fn test_emit_activities_advances_to_stop_canister() {
        let sm = machine(TestManagementClient::ok(), TestUserCanisterClient::ok());

        let result = sm.emit_activities(user_canister_id()).await;

        assert_eq!(
            result,
            DeleteState::StopCanister {
                canister_id: user_canister_id()
            }
        );
    }

    #[tokio::test]
    async fn test_emit_activities_stays_on_failure() {
        let sm = machine(
            TestManagementClient::ok(),
            TestUserCanisterClient::ok().with_emit_err(),
        );

        let result = sm.emit_activities(user_canister_id()).await;

        assert_eq!(
            result,
            DeleteState::EmitActivities {
                canister_id: user_canister_id()
            }
        );
    }

    #[tokio::test]
    async fn test_stop_canister_advances_to_delete_canister() {
        let sm = machine(TestManagementClient::ok(), TestUserCanisterClient::ok());

        let result = sm.stop_canister(user_canister_id()).await;

        assert_eq!(
            result,
            DeleteState::DeleteCanister {
                canister_id: user_canister_id()
            }
        );
    }

    #[tokio::test]
    async fn test_stop_canister_stays_on_failure() {
        let sm = machine(
            TestManagementClient::ok().with_stop_err(),
            TestUserCanisterClient::ok(),
        );

        let result = sm.stop_canister(user_canister_id()).await;

        assert_eq!(
            result,
            DeleteState::StopCanister {
                canister_id: user_canister_id()
            }
        );
    }

    #[tokio::test]
    async fn test_delete_canister_advances_to_commit() {
        let sm = machine(TestManagementClient::ok(), TestUserCanisterClient::ok());

        let result = sm.delete_canister(user_canister_id()).await;

        assert_eq!(result, DeleteState::Commit);
    }

    #[tokio::test]
    async fn test_delete_canister_stays_on_failure() {
        let sm = machine(
            TestManagementClient::ok().with_delete_err(),
            TestUserCanisterClient::ok(),
        );

        let result = sm.delete_canister(user_canister_id()).await;

        assert_eq!(
            result,
            DeleteState::DeleteCanister {
                canister_id: user_canister_id()
            }
        );
    }

    #[tokio::test]
    async fn test_commit_removes_user_row() {
        setup();
        UserRepository::sign_up(rey_canisteryo(), "alice".to_string())
            .expect("should sign up user");

        let sm = machine(TestManagementClient::ok(), TestUserCanisterClient::ok());

        sm.commit().expect("commit should succeed");

        let user =
            UserRepository::get_user_by_principal(rey_canisteryo()).expect("should query user");
        assert!(user.is_none());
    }

    #[tokio::test]
    async fn test_step_increments_retries_on_same_state() {
        let sm = machine(
            TestManagementClient::ok(),
            TestUserCanisterClient::ok().with_emit_err(),
        );
        let current = DeleteStateStep {
            state: DeleteState::EmitActivities {
                canister_id: user_canister_id(),
            },
            retries: 1,
        };

        let result = sm.step(current).await;

        assert_eq!(
            result,
            StepResult::Continue(DeleteStateStep {
                state: DeleteState::EmitActivities {
                    canister_id: user_canister_id()
                },
                retries: 2,
            })
        );
    }

    #[tokio::test]
    async fn test_step_finishes_on_max_retries() {
        let sm = machine(
            TestManagementClient::ok(),
            TestUserCanisterClient::ok().with_emit_err(),
        );
        let current = DeleteStateStep {
            state: DeleteState::EmitActivities {
                canister_id: user_canister_id(),
            },
            retries: MAX_RETRIES,
        };

        let result = sm.step(current).await;

        assert_eq!(result, StepResult::Finished);
    }

    #[tokio::test]
    async fn test_step_finishes_on_successful_commit() {
        setup();
        UserRepository::sign_up(rey_canisteryo(), "alice".to_string())
            .expect("should sign up user");

        let sm = machine(TestManagementClient::ok(), TestUserCanisterClient::ok());
        let current = DeleteStateStep {
            state: DeleteState::Commit,
            retries: 0,
        };

        let result = sm.step(current).await;

        assert_eq!(result, StepResult::Finished);
    }
}
