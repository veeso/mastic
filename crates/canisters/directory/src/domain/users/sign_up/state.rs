use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

use candid::Principal;
use did::user::UserInstallArgs;
use ic_management_canister_types::CanisterSettings;

use crate::adapters::management_canister::ManagementCanister;
use crate::domain::users::repository::UserRepository;
use crate::error::CanisterResult;

thread_local! {
    /// If a canister is being created for a user, we store the state of the sign up process here.
    static USER_SIGN_UP_STATES: RefCell<HashMap<Principal, SignUpStateStep>> = RefCell::new(HashMap::new());
}

/// Initial amount of cycles to attach to the user canister during creation.
const INITIAL_USER_CANISTER_CYCLES: u128 = 1_000_000_000_000;
/// Minimum interval between two sign up operations to prevent abuse.
const OPERATION_INTERVAL: Duration = Duration::from_secs(1);
/// Maximum number of retries for each step in the sign up process before giving up.
const MAX_RETRIES: u8 = 5;
/// User canister wasm bytes.
const USER_CANISTER_WASM: &[u8] = include_bytes!("../../../../../../../.artifact/user.wasm.gz");

/// Represents a step in the user sign up process, along with the number of retries for that step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SignUpStateStep {
    state: SignUpState,
    retries: u8,
}

impl Default for SignUpStateStep {
    fn default() -> Self {
        Self {
            state: SignUpState::CreateCanister,
            retries: 0,
        }
    }
}

/// Represents the result of a sign up operation for a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignUpResult {
    /// The sign up process completed successfully, and the user's canister ID is provided.
    Success { canister_id: Principal },
    /// The sign up process failed, and the user should be marked as failed in the directory canister.
    Failure,
}

/// Represents the state of a user's sign up process.
///
/// This state is used for the [`SignUpStateMachine`] to track the progress of a user's sign up operation
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum SignUpState {
    /// The user has initiated the sign up process.
    ///
    /// The state machine should send a request to the management canister to create a new canister for the user.
    #[default]
    CreateCanister,
    /// The canister has been created.
    ///
    /// The state machine is ready to install the user canister wasm and initialize the canister with the user's information.
    InstallWasm { canister_id: Principal },
    /// Commit changes to the directory canister to mark the user as active and associate them with their canister ID.
    CommitSignUp(SignUpResult),
}

/// Result of a single state machine step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepResult {
    /// Continue processing with updated state.
    Continue(SignUpStateStep),
    /// The sign up process is complete; clean up.
    Finished,
}

/// State machine for the user sign up process.
///
/// Generic over `C` so that the management canister dependency can be replaced
/// with a mock during unit tests.
pub struct SignUpStateMachine<C>
where
    C: ManagementCanister + 'static,
{
    /// The user for which the sign up process is being executed.
    user_id: Principal,
    /// Adapter for management canister calls.
    client: C,
}

impl<C> SignUpStateMachine<C>
where
    C: ManagementCanister + 'static,
{
    /// Start a new sign up process for a user by initializing their state in the `USER_SIGN_UP_STATES` thread-local storage.
    pub fn start(user_id: Principal, client: C) {
        ic_utils::log!("Starting sign up process for user {user_id}",);
        // if there is already an entry for the user, return early to prevent starting multiple sign up processes for the same user
        let already_exists =
            USER_SIGN_UP_STATES.with_borrow(|states| states.contains_key(&user_id));
        if already_exists {
            return;
        }

        // insert the user in the sign up states with the initial state
        USER_SIGN_UP_STATES
            .with_borrow_mut(|states| states.insert(user_id, SignUpStateStep::default()));

        Self { user_id, client }.tick();
    }

    /// Tick the state machine to progress the sign up process for the user.
    fn tick(self) {
        ic_utils::log!("Ticking sign up process for user {}", self.user_id);
        ic_utils::set_timer(OPERATION_INTERVAL, async move {
            ic_utils::log!("Timer fired for user {}", self.user_id);
            self.run().await;
        });
    }

    /// Run a step of the sign up process for the user based on their current state in the `USER_SIGN_UP_STATES` thread-local storage.
    async fn run(self) {
        // get the current state
        let Some(current) =
            USER_SIGN_UP_STATES.with_borrow(|states| states.get(&self.user_id).copied())
        else {
            // invalid, return early
            return;
        };

        match self.step(current).await {
            StepResult::Continue(next) => {
                USER_SIGN_UP_STATES.with_borrow_mut(|states| states.insert(self.user_id, next));
                self.tick();
            }
            StepResult::Finished => self.finish(),
        }
    }

    /// Execute a single state-transition step.
    ///
    /// This is the pure logic of the state machine, separated from timer
    /// scheduling and thread-local storage so it can be unit-tested.
    async fn step(&self, current: SignUpStateStep) -> StepResult {
        let SignUpStateStep { state, retries } = current;
        ic_utils::log!(
            "Running sign up step for user {}: state={:?}, retries={}",
            self.user_id,
            state,
            retries
        );
        let current_discriminant = std::mem::discriminant(&state);

        // check retries
        if retries >= MAX_RETRIES {
            return if state == SignUpState::CommitSignUp(SignUpResult::Failure) {
                // commit-failure itself exhausted retries; give up entirely
                StepResult::Finished
            } else {
                // transition to failure commit
                StepResult::Continue(SignUpStateStep {
                    state: SignUpState::CommitSignUp(SignUpResult::Failure),
                    retries: 0,
                })
            };
        }

        // execute the current state
        let new_state = match state {
            SignUpState::CreateCanister => self.create_canister().await,
            SignUpState::InstallWasm { canister_id } => self.install_wasm(canister_id).await,
            SignUpState::CommitSignUp(SignUpResult::Success { canister_id }) => {
                match self.commit_sign_up(canister_id) {
                    Ok(()) => return StepResult::Finished,
                    Err(_) => SignUpState::CommitSignUp(SignUpResult::Success { canister_id }),
                }
            }
            SignUpState::CommitSignUp(SignUpResult::Failure) => {
                match self.commit_sign_up_failure() {
                    Ok(()) => return StepResult::Finished,
                    Err(_) => SignUpState::CommitSignUp(SignUpResult::Failure),
                }
            }
        };
        let new_discriminant = std::mem::discriminant(&new_state);

        // determine new state step based on whether the state changed
        let next = if current_discriminant == new_discriminant {
            SignUpStateStep {
                state: new_state,
                retries: retries + 1,
            }
        } else {
            SignUpStateStep {
                state: new_state,
                retries: 0,
            }
        };

        StepResult::Continue(next)
    }

    /// Create a canister for the user by sending a request to the management canister.
    async fn create_canister(&self) -> SignUpState {
        let controller = self.client.canister_self();

        let settings = Some(CanisterSettings {
            controllers: Some(vec![controller]),
            ..Default::default()
        });

        #[allow(irrefutable_let_patterns)]
        let Ok(canister_id) = self
            .client
            .create_canister(settings, INITIAL_USER_CANISTER_CYCLES)
            .await
        else {
            // failed, return same state to retry
            return SignUpState::CreateCanister;
        };

        // success, move to next state
        SignUpState::InstallWasm { canister_id }
    }

    /// Install the user canister wasm and initialize the canister with the user's information.
    async fn install_wasm(&self, canister_id: Principal) -> SignUpState {
        let Ok(federation_canister) = crate::settings::get_federation_canister() else {
            // failed to get federation canister, return same state to retry
            return SignUpState::InstallWasm { canister_id };
        };
        // make init arguments and encode
        let init_args = UserInstallArgs::Init {
            owner: self.user_id,
            federation_canister,
        };
        let Ok(init_args) = candid::encode_one(init_args) else {
            // failed to encode, return same state to retry
            return SignUpState::InstallWasm { canister_id };
        };

        if self
            .client
            .install_code(canister_id, USER_CANISTER_WASM, init_args)
            .await
            .is_err()
        {
            // failed, return same state to retry
            return SignUpState::InstallWasm { canister_id };
        };

        // success, move to next state
        SignUpState::CommitSignUp(SignUpResult::Success { canister_id })
    }

    fn commit_sign_up(&self, canister_id: Principal) -> CanisterResult<()> {
        UserRepository::set_user_canister(self.user_id, canister_id)
    }

    fn commit_sign_up_failure(&self) -> CanisterResult<()> {
        UserRepository::set_failed_user_canister_create(self.user_id)
    }

    /// Finish the sign up process for the user by removing their state from the `USER_SIGN_UP_STATES` thread-local storage.
    fn finish(&self) {
        // remove entry
        USER_SIGN_UP_STATES.with_borrow_mut(|states| states.remove(&self.user_id));
    }
}

#[cfg(test)]
mod tests {

    use candid::Principal;

    use super::*;
    use crate::adapters::management_canister::ManagementCanisterError;
    use crate::test_utils::{rey_canisteryo, setup};

    /// Configurable mock for [`ManagementCanister`].
    struct TestClient {
        canister_self: Principal,
        create_result: Result<Principal, ManagementCanisterError>,
        install_result: Result<(), ManagementCanisterError>,
    }

    impl TestClient {
        /// Return a client where all operations succeed.
        fn ok(created_canister_id: Principal) -> Self {
            Self {
                canister_self: Principal::management_canister(),
                create_result: Ok(created_canister_id),
                install_result: Ok(()),
            }
        }

        fn with_create_err(mut self) -> Self {
            self.create_result = Err(ManagementCanisterError::CallFailed("test".to_string()));
            self
        }

        fn with_install_err(mut self) -> Self {
            self.install_result = Err(ManagementCanisterError::CallFailed("test".to_string()));
            self
        }
    }

    impl ManagementCanister for TestClient {
        async fn create_canister(
            &self,
            _settings: Option<CanisterSettings>,
            _cycles: u128,
        ) -> Result<Principal, ManagementCanisterError> {
            self.create_result.clone()
        }

        async fn install_code(
            &self,
            _canister_id: Principal,
            _wasm_module: &[u8],
            _arg: Vec<u8>,
        ) -> Result<(), ManagementCanisterError> {
            self.install_result.clone()
        }

        fn canister_version(&self) -> u64 {
            0
        }

        fn canister_self(&self) -> Principal {
            self.canister_self
        }
    }

    fn user_canister() -> Principal {
        Principal::from_text("b77ix-eeaaa-aaaaa-qaada-cai").unwrap()
    }

    fn machine(client: TestClient) -> SignUpStateMachine<TestClient> {
        SignUpStateMachine {
            user_id: rey_canisteryo(),
            client,
        }
    }

    // -- create_canister step -------------------------------------------------

    #[tokio::test]
    async fn test_create_canister_should_advance_to_install_wasm() {
        let sm = machine(TestClient::ok(user_canister()));

        let result = sm.create_canister().await;

        assert_eq!(
            result,
            SignUpState::InstallWasm {
                canister_id: user_canister()
            }
        );
    }

    #[tokio::test]
    async fn test_create_canister_should_retry_on_failure() {
        let sm = machine(TestClient::ok(user_canister()).with_create_err());

        let result = sm.create_canister().await;

        assert_eq!(result, SignUpState::CreateCanister);
    }

    // -- install_wasm step ----------------------------------------------------

    #[tokio::test]
    async fn test_install_wasm_should_advance_to_commit() {
        setup();
        let sm = machine(TestClient::ok(user_canister()));

        let result = sm.install_wasm(user_canister()).await;

        assert_eq!(
            result,
            SignUpState::CommitSignUp(SignUpResult::Success {
                canister_id: user_canister()
            })
        );
    }

    #[tokio::test]
    async fn test_install_wasm_should_retry_on_install_failure() {
        setup();
        let sm = machine(TestClient::ok(user_canister()).with_install_err());

        let result = sm.install_wasm(user_canister()).await;

        assert_eq!(
            result,
            SignUpState::InstallWasm {
                canister_id: user_canister()
            }
        );
    }

    // -- step: full state transitions -----------------------------------------

    #[tokio::test]
    async fn test_step_should_advance_from_create_to_install() {
        let sm = machine(TestClient::ok(user_canister()));
        let current = SignUpStateStep::default();

        let result = sm.step(current).await;

        assert_eq!(
            result,
            StepResult::Continue(SignUpStateStep {
                state: SignUpState::InstallWasm {
                    canister_id: user_canister()
                },
                retries: 0,
            })
        );
    }

    #[tokio::test]
    async fn test_step_should_increment_retries_on_same_state() {
        let sm = machine(TestClient::ok(user_canister()).with_create_err());
        let current = SignUpStateStep {
            state: SignUpState::CreateCanister,
            retries: 2,
        };

        let result = sm.step(current).await;

        assert_eq!(
            result,
            StepResult::Continue(SignUpStateStep {
                state: SignUpState::CreateCanister,
                retries: 3,
            })
        );
    }

    #[tokio::test]
    async fn test_step_should_transition_to_failure_on_max_retries() {
        let sm = machine(TestClient::ok(user_canister()).with_create_err());
        let current = SignUpStateStep {
            state: SignUpState::CreateCanister,
            retries: MAX_RETRIES,
        };

        let result = sm.step(current).await;

        assert_eq!(
            result,
            StepResult::Continue(SignUpStateStep {
                state: SignUpState::CommitSignUp(SignUpResult::Failure),
                retries: 0,
            })
        );
    }

    #[tokio::test]
    async fn test_step_should_finish_when_commit_failure_exhausts_retries() {
        let sm = machine(TestClient::ok(user_canister()));
        let current = SignUpStateStep {
            state: SignUpState::CommitSignUp(SignUpResult::Failure),
            retries: MAX_RETRIES,
        };

        let result = sm.step(current).await;

        assert_eq!(result, StepResult::Finished);
    }

    #[tokio::test]
    async fn test_step_should_finish_on_successful_commit() {
        setup();
        let sm = machine(TestClient::ok(user_canister()));
        // the user must exist in the DB for commit to succeed
        UserRepository::sign_up(rey_canisteryo(), "alice".to_string())
            .expect("sign_up should succeed");

        let current = SignUpStateStep {
            state: SignUpState::CommitSignUp(SignUpResult::Success {
                canister_id: user_canister(),
            }),
            retries: 0,
        };

        let result = sm.step(current).await;

        assert_eq!(result, StepResult::Finished);
    }

    #[tokio::test]
    async fn test_step_should_finish_on_successful_commit_failure() {
        setup();
        let sm = machine(TestClient::ok(user_canister()));
        UserRepository::sign_up(rey_canisteryo(), "alice".to_string())
            .expect("sign_up should succeed");

        let current = SignUpStateStep {
            state: SignUpState::CommitSignUp(SignUpResult::Failure),
            retries: 0,
        };

        let result = sm.step(current).await;

        assert_eq!(result, StepResult::Finished);
    }

    #[tokio::test]
    async fn test_step_should_retry_commit_on_missing_user() {
        setup();
        // do NOT insert the user — commit will fail
        let sm = machine(TestClient::ok(user_canister()));

        let current = SignUpStateStep {
            state: SignUpState::CommitSignUp(SignUpResult::Success {
                canister_id: user_canister(),
            }),
            retries: 1,
        };

        let result = sm.step(current).await;

        assert_eq!(
            result,
            StepResult::Continue(SignUpStateStep {
                state: SignUpState::CommitSignUp(SignUpResult::Success {
                    canister_id: user_canister()
                }),
                retries: 2,
            })
        );
    }
}
