use did::user::PublishStatusResponse;
use integration_tests::{MasticCanisterSetup, UserClient, rey_canisteryo};
use pocket_ic_harness::PocketIcTestEnv;

#[pocket_ic_harness::test]
async fn test_should_publish_status(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let user_canister_id = integration_tests::helpers::sign_up_user(
        &env,
        rey_canisteryo(),
        "rey_canisteryo".to_string(),
    )
    .await;

    // TODO: should have follower to check the feed

    let user_client = UserClient::new(&env, user_canister_id);
    if let PublishStatusResponse::Err(err) = user_client
        .publish_status(
            rey_canisteryo(),
            "Hello, World!".to_string(),
            did::common::Visibility::Public,
        )
        .await
    {
        panic!("Failed to publish status: {:?}", err);
    }
}
