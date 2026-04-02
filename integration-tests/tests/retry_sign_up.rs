use did::directory::{RetrySignUpError, RetrySignUpResponse, SignUpResponse};
use integration_tests::{DirectoryClient, MasticCanisterSetup};
use pocket_ic_harness::{PocketIcTestEnv, alice};

#[pocket_ic_harness::test]
async fn test_should_not_retry_sign_up_if_not_failed(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    let response = client.sign_up(alice(), "alice".to_string()).await;
    assert_eq!(response, SignUpResponse::Ok);

    let response = client.retry_sign_up(alice()).await;
    assert_eq!(
        response,
        RetrySignUpResponse::Err(RetrySignUpError::CanisterNotInFailedState)
    );
}
