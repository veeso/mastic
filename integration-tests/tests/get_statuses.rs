use did::common::Visibility;
use did::user::{GetStatusesResponse, PublishStatusResponse};
use integration_tests::helpers::{follow_and_accept, sign_up_user};
use integration_tests::{MasticCanisterSetup, UserClient, carol, charlie};
use pocket_ic_harness::{PocketIcTestEnv, alice, bob};

async fn publish(
    client: &UserClient<'_>,
    caller: candid::Principal,
    content: &str,
    visibility: Visibility,
    mentions: Vec<String>,
) {
    let resp = client
        .publish_status(caller, content.to_string(), visibility, mentions)
        .await;
    assert!(matches!(resp, PublishStatusResponse::Ok(_)), "{resp:?}");
}

#[pocket_ic_harness::test]
async fn test_public_outbox_visible_to_any_caller(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let alice_client = UserClient::new(&env, alice_canister);

    publish(
        &alice_client,
        alice(),
        "public post",
        Visibility::Public,
        vec![],
    )
    .await;
    publish(
        &alice_client,
        alice(),
        "unlisted post",
        Visibility::Unlisted,
        vec![],
    )
    .await;

    let charlie_canister = sign_up_user(&env, charlie(), "charlie".to_string()).await;

    // charlie (not a follower) queries alice's statuses
    let GetStatusesResponse::Ok(statuses) = alice_client.get_statuses(charlie(), 0, 10).await
    else {
        panic!("get_statuses failed");
    };
    assert_eq!(statuses.len(), 2);
    let contents: Vec<&str> = statuses.iter().map(|s| s.content.as_str()).collect();
    assert!(contents.contains(&"public post"));
    assert!(contents.contains(&"unlisted post"));

    // ensure charlie canister variable used (avoid unused warning)
    let _ = charlie_canister;
}

#[pocket_ic_harness::test]
async fn test_followers_only_filtered_by_relationship(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    // bob is a follower of alice
    let _bob_canister =
        follow_and_accept(&env, bob(), "bob", alice(), alice_canister, "alice").await;
    // carol is signed up but does not follow alice
    let _carol_canister = sign_up_user(&env, carol(), "carol".to_string()).await;

    let alice_client = UserClient::new(&env, alice_canister);
    publish(
        &alice_client,
        alice(),
        "followers only".to_string().as_str(),
        Visibility::FollowersOnly,
        vec![],
    )
    .await;
    publish(&alice_client, alice(), "public", Visibility::Public, vec![]).await;

    // bob (follower) sees both
    let GetStatusesResponse::Ok(bob_view) = alice_client.get_statuses(bob(), 0, 10).await else {
        panic!("bob get_statuses failed");
    };
    assert_eq!(bob_view.len(), 2);
    let bob_contents: Vec<&str> = bob_view.iter().map(|s| s.content.as_str()).collect();
    assert!(bob_contents.contains(&"followers only"));
    assert!(bob_contents.contains(&"public"));

    // carol (non-follower) sees only the public one
    let GetStatusesResponse::Ok(carol_view) = alice_client.get_statuses(carol(), 0, 10).await
    else {
        panic!("carol get_statuses failed");
    };
    assert_eq!(carol_view.len(), 1);
    assert_eq!(carol_view[0].content, "public");
    assert_eq!(carol_view[0].visibility, Visibility::Public);
}

#[pocket_ic_harness::test]
async fn test_direct_excluded_from_get_statuses(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let _bob_canister =
        follow_and_accept(&env, bob(), "bob", alice(), alice_canister, "alice").await;
    let _carol_canister = sign_up_user(&env, carol(), "carol".to_string()).await;

    let alice_client = UserClient::new(&env, alice_canister);

    // alice sends a Direct mentioning bob
    publish(
        &alice_client,
        alice(),
        "secret to bob",
        Visibility::Direct,
        vec![integration_tests::helpers::actor_uri("bob")],
    )
    .await;

    // bob (follower + mentioned) does NOT see it via get_statuses — Direct excluded
    let GetStatusesResponse::Ok(bob_view) = alice_client.get_statuses(bob(), 0, 10).await else {
        panic!("bob get_statuses failed");
    };
    assert!(
        bob_view.iter().all(|s| s.visibility != Visibility::Direct),
        "bob must not see Direct via get_statuses"
    );

    // carol (non-follower) also sees nothing (no public/unlisted posts exist)
    let GetStatusesResponse::Ok(carol_view) = alice_client.get_statuses(carol(), 0, 10).await
    else {
        panic!("carol get_statuses failed");
    };
    assert!(carol_view.is_empty());
}
