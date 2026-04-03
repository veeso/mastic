use std::time::Instant;

use did::directory::{SignUpError, SignUpResponse, WhoAmIResponse};
use did::user::GetProfileResponse;
use integration_tests::{DirectoryClient, MasticCanisterSetup, UserClient, rey_canisteryo};
use pocket_ic_harness::{PocketIcTestEnv, alice, bob};

#[pocket_ic_harness::test]
async fn test_should_sign_up(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    let response = client
        .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())
        .await;
    assert_eq!(response, SignUpResponse::Ok);

    // wait for canister to be created
    let t = Instant::now();
    let user_canister = loop {
        if t.elapsed() > std::time::Duration::from_secs(30) {
            panic!("timout waiting for canister to be created");
        }
        match client.whoami(rey_canisteryo()).await {
            WhoAmIResponse::Ok(info) if info.user_canister.is_some() => {
                assert_eq!(info.handle, "rey_canisteryo");
                assert_eq!(
                    info.canister_status,
                    did::directory::UserCanisterStatus::Active
                );
                break info.user_canister.unwrap();
            }
            WhoAmIResponse::Ok(info) => {
                assert!(info.user_canister.is_none());
                if info.canister_status == did::directory::UserCanisterStatus::CreationFailed {
                    panic!("canister creation failed");
                }
                assert!(
                    info.canister_status == did::directory::UserCanisterStatus::CreationPending
                );
                println!("canister status: {:?}, waiting...", info.canister_status);
            }
            WhoAmIResponse::Err(e) => panic!("expected Ok, got Err({e:?})"),
        }
        // Sleep for a bit before retrying
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        env.pic
            .advance_time(std::time::Duration::from_secs(1))
            .await;
        env.pic.tick().await;
    };

    // get user profile and verify the handle
    let user_client = UserClient::new(&env, user_canister);
    let GetProfileResponse::Ok(profile) = user_client.get_profile(rey_canisteryo()).await else {
        panic!("expected Ok, got Err");
    };
    assert_eq!(profile.handle, "rey_canisteryo");
}

#[pocket_ic_harness::test]
async fn test_should_not_accept_duplicate_handle(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    let response = client
        .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())
        .await;
    assert_eq!(response, SignUpResponse::Ok);

    let response = client.sign_up(bob(), "rey_canisteryo".to_string()).await;
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
