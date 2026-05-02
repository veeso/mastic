//! Integration tests for WI-1.7 / UC11 — boost status.

use did::common::Visibility;
use did::user::{
    BoostStatusResponse, GetStatusesResponse, PublishStatusResponse, ReadFeedResponse,
    UndoBoostResponse,
};
use integration_tests::helpers::{follow_and_accept, sign_up_user};
use integration_tests::{MasticCanisterSetup, PUBLIC_URL, UserClient, charlie};
use pocket_ic_harness::{PocketIcTestEnv, alice, bob};

fn status_uri(handle: &str, id: u64) -> String {
    format!("{PUBLIC_URL}/users/{handle}/statuses/{id}")
}

async fn publish_bob_public_status(
    env: &PocketIcTestEnv<MasticCanisterSetup>,
    bob_canister: candid::Principal,
) -> u64 {
    let bob_client = UserClient::new(env, bob_canister);
    let resp = bob_client
        .publish_status(bob(), "Bob says hi".to_string(), Visibility::Public, vec![])
        .await;
    let PublishStatusResponse::Ok(status) = resp else {
        panic!("publish_status failed: {resp:?}");
    };
    status.id
}

#[pocket_ic_harness::test]
async fn test_alice_boosts_bob_charlie_sees_it(env: PocketIcTestEnv<MasticCanisterSetup>) {
    // 1. Set up Alice (booster) and Bob (author).
    let bob_canister = sign_up_user(&env, bob(), "bob".to_string()).await;
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;

    // 2. Charlie follows Alice (so Charlie should see Alice's boost in his feed).
    let charlie_canister =
        follow_and_accept(&env, charlie(), "charlie", alice(), alice_canister, "alice").await;

    let alice_client = UserClient::new(&env, alice_canister);
    let bob_client = UserClient::new(&env, bob_canister);
    let charlie_client = UserClient::new(&env, charlie_canister);

    // 3. Bob publishes a public status.
    let bob_status_id = publish_bob_public_status(&env, bob_canister).await;
    let bob_status_uri = status_uri("bob", bob_status_id);

    // 4. Alice boosts Bob's status.
    assert_eq!(
        alice_client
            .boost_status(alice(), bob_status_uri.clone())
            .await,
        BoostStatusResponse::Ok
    );

    // 5. Alice's feed contains the boost: boosted_by = alice, author = bob, id = bob's status id.
    let ReadFeedResponse::Ok(alice_feed) = alice_client.read_feed(alice(), 0, 10).await else {
        panic!("alice read_feed failed");
    };
    let alice_boost_item = alice_feed
        .iter()
        .find(|i| i.boosted_by.is_some())
        .expect("alice feed contains the boost");
    assert_eq!(
        alice_boost_item.boosted_by.as_deref(),
        Some(format!("{PUBLIC_URL}/users/alice").as_str())
    );
    assert_eq!(
        alice_boost_item.status.author,
        format!("{PUBLIC_URL}/users/bob")
    );
    assert_eq!(alice_boost_item.status.id, bob_status_id);
    assert_eq!(alice_boost_item.status.content, "Bob says hi");

    // 6. Charlie's feed contains the boost too.
    let ReadFeedResponse::Ok(charlie_feed) = charlie_client.read_feed(charlie(), 0, 10).await
    else {
        panic!("charlie read_feed failed");
    };
    let charlie_boost_item = charlie_feed
        .iter()
        .find(|i| i.boosted_by.is_some())
        .expect("charlie feed contains the boost");
    assert_eq!(
        charlie_boost_item.boosted_by.as_deref(),
        Some(format!("{PUBLIC_URL}/users/alice").as_str())
    );
    assert_eq!(
        charlie_boost_item.status.author,
        format!("{PUBLIC_URL}/users/bob")
    );

    // 7. Bob's status reports boost_count = 1.
    let GetStatusesResponse::Ok(bob_statuses) = bob_client.get_statuses(bob(), 0, 10).await else {
        panic!("get_statuses failed");
    };
    assert_eq!(bob_statuses.len(), 1);
    assert_eq!(bob_statuses[0].id, bob_status_id);
    assert_eq!(bob_statuses[0].boost_count, 1);

    // 8. Idempotent boost — calling again leaves the count at 1.
    assert_eq!(
        alice_client
            .boost_status(alice(), bob_status_uri.clone())
            .await,
        BoostStatusResponse::Ok
    );
    let GetStatusesResponse::Ok(bob_statuses_after) = bob_client.get_statuses(bob(), 0, 10).await
    else {
        panic!("get_statuses failed");
    };
    assert_eq!(bob_statuses_after[0].boost_count, 1);

    // 9. Undo boost — the boost item disappears from both feeds and Bob's count returns to 0.
    assert_eq!(
        alice_client
            .undo_boost(alice(), bob_status_uri.clone())
            .await,
        UndoBoostResponse::Ok
    );

    let ReadFeedResponse::Ok(alice_feed_after) = alice_client.read_feed(alice(), 0, 10).await
    else {
        panic!("alice read_feed failed");
    };
    assert!(
        alice_feed_after.iter().all(|i| i.boosted_by.is_none()),
        "alice's feed has no boost item after undo"
    );

    let ReadFeedResponse::Ok(charlie_feed_after) = charlie_client.read_feed(charlie(), 0, 10).await
    else {
        panic!("charlie read_feed failed");
    };
    assert!(
        charlie_feed_after.iter().all(|i| i.boosted_by.is_none()),
        "charlie's feed has no boost item after undo"
    );

    let GetStatusesResponse::Ok(bob_statuses_final) = bob_client.get_statuses(bob(), 0, 10).await
    else {
        panic!("get_statuses failed");
    };
    assert_eq!(bob_statuses_final[0].boost_count, 0);
}

#[pocket_ic_harness::test]
async fn test_self_boost(env: PocketIcTestEnv<MasticCanisterSetup>) {
    // Alice boosts her own status — should appear in her own feed.
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let alice_client = UserClient::new(&env, alice_canister);

    let resp = alice_client
        .publish_status(alice(), "Mine".to_string(), Visibility::Public, vec![])
        .await;
    let PublishStatusResponse::Ok(alice_status) = resp else {
        panic!("publish_status failed");
    };
    let alice_status_uri = status_uri("alice", alice_status.id);

    assert_eq!(
        alice_client
            .boost_status(alice(), alice_status_uri.clone())
            .await,
        BoostStatusResponse::Ok
    );

    let ReadFeedResponse::Ok(feed) = alice_client.read_feed(alice(), 0, 10).await else {
        panic!("read_feed failed");
    };
    let boost_item = feed
        .iter()
        .find(|i| i.boosted_by.is_some())
        .expect("self-boost shows in own feed");
    assert_eq!(
        boost_item.boosted_by.as_deref(),
        Some(format!("{PUBLIC_URL}/users/alice").as_str())
    );
    assert_eq!(
        boost_item.status.author,
        format!("{PUBLIC_URL}/users/alice")
    );
    assert_eq!(boost_item.status.id, alice_status.id);
}
