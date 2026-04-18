use did::user::GetProfileResponse;
use integration_tests::{MasticCanisterSetup, UserClient, rey_canisteryo};
use pocket_ic_harness::{PocketIcTestEnv, bob};

#[pocket_ic_harness::test]
async fn test_should_get_profile(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let user_canister_id = integration_tests::helpers::sign_up_user(
        &env,
        rey_canisteryo(),
        "rey_canisteryo".to_string(),
    )
    .await;

    let user_client = UserClient::new(&env, user_canister_id);
    // call get profile
    let GetProfileResponse::Ok(profile) = user_client.get_profile(rey_canisteryo()).await else {
        panic!("Failed to get profile");
    };

    assert_eq!(profile.avatar, None);
    assert_eq!(profile.display_name, None);
    assert_eq!(profile.handle, "rey_canisteryo".to_string());

    // call get profile with another user
    let GetProfileResponse::Ok(profile) = user_client.get_profile(bob()).await else {
        panic!("Failed to get profile");
    };

    assert_eq!(profile.avatar, None);
    assert_eq!(profile.display_name, None);
    assert_eq!(profile.handle, "rey_canisteryo".to_string());
}
