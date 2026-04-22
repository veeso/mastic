use did::common::FieldUpdate;
use did::user::{GetProfileResponse, UpdateProfileResponse};
use integration_tests::helpers::{follow_and_accept, sign_up_user};
use integration_tests::{MasticCanisterSetup, UserClient};
use pocket_ic_harness::{PocketIcTestEnv, alice, bob};

#[pocket_ic_harness::test]
async fn test_should_update_display_name_and_bio(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let alice_client = UserClient::new(&env, alice_canister);

    let resp = alice_client
        .update_profile(
            alice(),
            FieldUpdate::Set("Alice A.".to_string()),
            FieldUpdate::Set("hello fediverse".to_string()),
        )
        .await;
    assert_eq!(resp, UpdateProfileResponse::Ok);

    let GetProfileResponse::Ok(profile) = alice_client.get_profile(alice()).await else {
        panic!("get_profile failed");
    };
    assert_eq!(profile.display_name.as_deref(), Some("Alice A."));
    assert_eq!(profile.bio.as_deref(), Some("hello fediverse"));
}

#[pocket_ic_harness::test]
async fn test_should_clear_profile_fields(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let alice_client = UserClient::new(&env, alice_canister);

    let resp = alice_client
        .update_profile(
            alice(),
            FieldUpdate::Set("Alice".to_string()),
            FieldUpdate::Set("hi".to_string()),
        )
        .await;
    assert_eq!(resp, UpdateProfileResponse::Ok);

    let resp = alice_client
        .update_profile(alice(), FieldUpdate::Clear, FieldUpdate::Clear)
        .await;
    assert_eq!(resp, UpdateProfileResponse::Ok);

    let GetProfileResponse::Ok(profile) = alice_client.get_profile(alice()).await else {
        panic!("get_profile failed");
    };
    assert!(profile.display_name.is_none());
    assert!(profile.bio.is_none());
}

#[pocket_ic_harness::test]
async fn test_should_leave_fields_unchanged(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let alice_client = UserClient::new(&env, alice_canister);

    alice_client
        .update_profile(
            alice(),
            FieldUpdate::Set("Alice".to_string()),
            FieldUpdate::Set("bio".to_string()),
        )
        .await;

    let resp = alice_client
        .update_profile(
            alice(),
            FieldUpdate::Leave,
            FieldUpdate::Set("updated bio".to_string()),
        )
        .await;
    assert_eq!(resp, UpdateProfileResponse::Ok);

    let GetProfileResponse::Ok(profile) = alice_client.get_profile(alice()).await else {
        panic!("get_profile failed");
    };
    assert_eq!(profile.display_name.as_deref(), Some("Alice"));
    assert_eq!(profile.bio.as_deref(), Some("updated bio"));
}

#[pocket_ic_harness::test]
async fn test_should_dispatch_update_to_followers(env: PocketIcTestEnv<MasticCanisterSetup>) {
    // Alice is followed by bob. Alice updates her display name. The call
    // returns Ok, meaning dispatch to Federation succeeded. Reception-side
    // application of Update(Person) is deferred to M3.
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let _bob_canister =
        follow_and_accept(&env, bob(), "bob", alice(), alice_canister, "alice").await;

    let alice_client = UserClient::new(&env, alice_canister);
    let resp = alice_client
        .update_profile(
            alice(),
            FieldUpdate::Set("Alice Updated".to_string()),
            FieldUpdate::Leave,
        )
        .await;
    assert_eq!(resp, UpdateProfileResponse::Ok);

    let GetProfileResponse::Ok(profile) = alice_client.get_profile(alice()).await else {
        panic!("get_profile failed");
    };
    assert_eq!(profile.display_name.as_deref(), Some("Alice Updated"));
}
