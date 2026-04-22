use std::time::{Duration, Instant};

use did::directory::{
    DeleteProfileError, DeleteProfileResponse, GetUserArgs, GetUserError, GetUserResponse,
    RetryDeleteProfileError, RetryDeleteProfileResponse, SignUpError, SignUpResponse,
    UserCanisterError, UserCanisterResponse, WhoAmIError, WhoAmIResponse,
};
use integration_tests::helpers::sign_up_user;
use integration_tests::{DirectoryClient, MasticCanisterSetup};
use pocket_ic_harness::{PocketIcTestEnv, alice, bob};

async fn wait_for_deleted(
    env: &PocketIcTestEnv<MasticCanisterSetup>,
    client: &DirectoryClient<'_>,
    user: candid::Principal,
) {
    let t = Instant::now();
    loop {
        if t.elapsed() > Duration::from_secs(60) {
            panic!("timeout waiting for user deletion to complete");
        }
        if let WhoAmIResponse::Err(WhoAmIError::NotRegistered) = client.whoami(user).await {
            return;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        env.pic.advance_time(Duration::from_secs(1)).await;
        env.pic.tick().await;
    }
}

#[pocket_ic_harness::test]
async fn test_should_delete_profile(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    sign_up_user(&env, bob(), "bob".to_string()).await;

    let response = client.delete_profile(bob()).await;
    assert_eq!(response, DeleteProfileResponse::Ok);

    wait_for_deleted(&env, &client, bob()).await;

    let response = client.user_canister(bob(), None).await;
    assert_eq!(
        response,
        UserCanisterResponse::Err(UserCanisterError::NotRegistered)
    );

    let response = client
        .get_user(GetUserArgs::Handle("bob".to_string()))
        .await;
    assert_eq!(response, GetUserResponse::Err(GetUserError::NotFound));
}

#[pocket_ic_harness::test]
async fn test_should_reject_unregistered_delete(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    let response = client.delete_profile(bob()).await;
    assert_eq!(
        response,
        DeleteProfileResponse::Err(DeleteProfileError::NotRegistered)
    );
}

#[pocket_ic_harness::test]
async fn test_should_tombstone_handle_after_delete(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    sign_up_user(&env, bob(), "bob".to_string()).await;

    client.delete_profile(bob()).await;
    wait_for_deleted(&env, &client, bob()).await;

    // another user cannot reclaim the handle during the tombstone grace period
    let response = client.sign_up(alice(), "bob".to_string()).await;
    assert_eq!(response, SignUpResponse::Err(SignUpError::HandleTombstoned));
}

#[pocket_ic_harness::test]
async fn test_retry_delete_profile_rejects_when_not_in_deletion_state(
    env: PocketIcTestEnv<MasticCanisterSetup>,
) {
    let client = DirectoryClient::new(&env);

    sign_up_user(&env, bob(), "bob".to_string()).await;

    let response = client.retry_delete_profile(bob()).await;
    assert_eq!(
        response,
        RetryDeleteProfileResponse::Err(RetryDeleteProfileError::CanisterNotInDeletionState)
    );
}

#[pocket_ic_harness::test]
async fn test_retry_delete_profile_rejects_unregistered(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    let response = client.retry_delete_profile(bob()).await;
    assert_eq!(
        response,
        RetryDeleteProfileResponse::Err(RetryDeleteProfileError::NotRegistered)
    );
}
