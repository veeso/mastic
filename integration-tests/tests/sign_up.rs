use candid::Principal;
use did::directory::{SignUpError, SignUpResponse};
use integration_tests::{DirectoryClient, MasticCanisterSetup};
use pocket_ic_harness::{PocketIcTestEnv, alice, bob};

#[pocket_ic_harness::test]
async fn test_should_sign_up(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    let response = client.sign_up(alice(), "alice".to_string()).await;
    assert_eq!(response, SignUpResponse::Ok);
}

#[pocket_ic_harness::test]
async fn test_should_not_accept_duplicate_handle(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    let response = client.sign_up(alice(), "alice".to_string()).await;
    assert_eq!(response, SignUpResponse::Ok);

    let response = client.sign_up(bob(), "alice".to_string()).await;
    assert_eq!(response, SignUpResponse::Err(SignUpError::HandleTaken));
}

#[pocket_ic_harness::test]
async fn test_should_not_accept_duplicate_principal(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    let response = client.sign_up(alice(), "alice".to_string()).await;
    assert_eq!(response, SignUpResponse::Ok);

    let response = client.sign_up(alice(), "miss_alice".to_string()).await;
    assert_eq!(
        response,
        SignUpResponse::Err(SignUpError::AlreadyRegistered)
    );
}

#[pocket_ic_harness::test]
async fn test_should_not_accept_anonymous_principal(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    let response = client
        .sign_up(Principal::anonymous(), "alice".to_string())
        .await;
    assert_eq!(
        response,
        SignUpResponse::Err(SignUpError::AnonymousPrincipal)
    );
}
