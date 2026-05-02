use did::common::Visibility;
use did::user::{
    GetLikedResponse, GetStatusesResponse, LikeStatusResponse, PublishStatusResponse,
    ReadFeedResponse, UnlikeStatusResponse,
};
use integration_tests::helpers::{follow_and_accept, sign_up_user};
use integration_tests::{MasticCanisterSetup, PUBLIC_URL, UserClient};
use pocket_ic_harness::{PocketIcTestEnv, alice, bob};

fn status_uri(handle: &str, id: u64) -> String {
    format!("{PUBLIC_URL}/users/{handle}/statuses/{id}")
}

/// Publish a public status as `bob` and return its assigned id.
async fn publish_bob_status(
    env: &PocketIcTestEnv<MasticCanisterSetup>,
    bob_canister: candid::Principal,
) -> u64 {
    let bob_client = UserClient::new(env, bob_canister);
    let resp = bob_client
        .publish_status(bob(), "hello".to_string(), Visibility::Public, vec![])
        .await;
    let PublishStatusResponse::Ok(status) = resp else {
        panic!("publish_status failed: {resp:?}");
    };
    status.id
}

#[pocket_ic_harness::test]
async fn test_should_like_status_and_increment_count(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let bob_canister = sign_up_user(&env, bob(), "bob".to_string()).await;
    let alice_canister =
        follow_and_accept(&env, alice(), "alice", bob(), bob_canister, "bob").await;

    let alice_client = UserClient::new(&env, alice_canister);
    let bob_client = UserClient::new(&env, bob_canister);

    let bob_status_id = publish_bob_status(&env, bob_canister).await;
    let bob_status_uri = status_uri("bob", bob_status_id);

    // alice likes bob's status
    assert_eq!(
        alice_client
            .like_status(alice(), bob_status_uri.clone())
            .await,
        LikeStatusResponse::Ok
    );

    // alice's liked collection contains the URI
    let GetLikedResponse::Ok(liked) = alice_client.get_liked(alice(), 0, 10).await else {
        panic!("get_liked failed");
    };
    assert_eq!(liked, vec![bob_status_uri.clone()]);

    // bob's status now reports like_count = 1
    let GetStatusesResponse::Ok(bob_statuses) = bob_client.get_statuses(bob(), 0, 10).await else {
        panic!("get_statuses failed");
    };
    assert_eq!(bob_statuses.len(), 1);
    assert_eq!(bob_statuses[0].id, bob_status_id);
    assert_eq!(bob_statuses[0].like_count, 1);
}

#[pocket_ic_harness::test]
async fn test_should_be_idempotent_when_liking_twice(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let bob_canister = sign_up_user(&env, bob(), "bob".to_string()).await;
    let alice_canister =
        follow_and_accept(&env, alice(), "alice", bob(), bob_canister, "bob").await;

    let alice_client = UserClient::new(&env, alice_canister);
    let bob_client = UserClient::new(&env, bob_canister);

    let bob_status_id = publish_bob_status(&env, bob_canister).await;
    let bob_status_uri = status_uri("bob", bob_status_id);

    // first like
    assert_eq!(
        alice_client
            .like_status(alice(), bob_status_uri.clone())
            .await,
        LikeStatusResponse::Ok
    );
    // second like (idempotent)
    assert_eq!(
        alice_client
            .like_status(alice(), bob_status_uri.clone())
            .await,
        LikeStatusResponse::Ok
    );

    // alice's liked list contains a single entry
    let GetLikedResponse::Ok(liked) = alice_client.get_liked(alice(), 0, 10).await else {
        panic!("get_liked failed");
    };
    assert_eq!(liked, vec![bob_status_uri]);

    // bob's status like_count incremented exactly once
    let GetStatusesResponse::Ok(bob_statuses) = bob_client.get_statuses(bob(), 0, 10).await else {
        panic!("get_statuses failed");
    };
    assert_eq!(bob_statuses[0].like_count, 1);
}

#[pocket_ic_harness::test]
async fn test_should_unlike_status_and_decrement_count(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let bob_canister = sign_up_user(&env, bob(), "bob".to_string()).await;
    let alice_canister =
        follow_and_accept(&env, alice(), "alice", bob(), bob_canister, "bob").await;

    let alice_client = UserClient::new(&env, alice_canister);
    let bob_client = UserClient::new(&env, bob_canister);

    let bob_status_id = publish_bob_status(&env, bob_canister).await;
    let bob_status_uri = status_uri("bob", bob_status_id);

    // like, then unlike
    assert_eq!(
        alice_client
            .like_status(alice(), bob_status_uri.clone())
            .await,
        LikeStatusResponse::Ok
    );
    assert_eq!(
        alice_client
            .unlike_status(alice(), bob_status_uri.clone())
            .await,
        UnlikeStatusResponse::Ok
    );

    // alice's liked list is empty
    let GetLikedResponse::Ok(liked) = alice_client.get_liked(alice(), 0, 10).await else {
        panic!("get_liked failed");
    };
    assert!(liked.is_empty());

    // bob's status like_count back to 0
    let GetStatusesResponse::Ok(bob_statuses) = bob_client.get_statuses(bob(), 0, 10).await else {
        panic!("get_statuses failed");
    };
    assert_eq!(bob_statuses[0].like_count, 0);
}

#[pocket_ic_harness::test]
async fn test_should_succeed_unlike_when_not_liked(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let bob_canister = sign_up_user(&env, bob(), "bob".to_string()).await;
    let alice_canister =
        follow_and_accept(&env, alice(), "alice", bob(), bob_canister, "bob").await;

    let alice_client = UserClient::new(&env, alice_canister);
    let bob_client = UserClient::new(&env, bob_canister);

    let bob_status_id = publish_bob_status(&env, bob_canister).await;
    let bob_status_uri = status_uri("bob", bob_status_id);

    // unlike without prior like — silent no-op
    assert_eq!(
        alice_client.unlike_status(alice(), bob_status_uri).await,
        UnlikeStatusResponse::Ok
    );

    let GetLikedResponse::Ok(liked) = alice_client.get_liked(alice(), 0, 10).await else {
        panic!("get_liked failed");
    };
    assert!(liked.is_empty());

    let GetStatusesResponse::Ok(bob_statuses) = bob_client.get_statuses(bob(), 0, 10).await else {
        panic!("get_statuses failed");
    };
    assert_eq!(bob_statuses[0].like_count, 0);
}

#[pocket_ic_harness::test]
async fn test_should_surface_liked_flag_in_feed(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let bob_canister = sign_up_user(&env, bob(), "bob".to_string()).await;
    let alice_canister =
        follow_and_accept(&env, alice(), "alice", bob(), bob_canister, "bob").await;

    let alice_client = UserClient::new(&env, alice_canister);

    let bob_status_id = publish_bob_status(&env, bob_canister).await;
    let bob_status_uri = status_uri("bob", bob_status_id);

    // Before liking: alice's feed shows bob's inbox-delivered status with liked=false.
    let ReadFeedResponse::Ok(feed_before) = alice_client.read_feed(alice(), 0, 10).await else {
        panic!("read_feed failed");
    };
    let item_before = feed_before
        .iter()
        .find(|i| i.status.id == bob_status_id)
        .expect("bob's status delivered to alice's inbox");
    assert!(!item_before.liked, "not liked yet");
    assert!(!item_before.boosted);

    // Alice likes bob's status.
    assert_eq!(
        alice_client
            .like_status(alice(), bob_status_uri.clone())
            .await,
        LikeStatusResponse::Ok
    );

    // After liking: liked flag flips to true on the same feed item.
    let ReadFeedResponse::Ok(feed_after) = alice_client.read_feed(alice(), 0, 10).await else {
        panic!("read_feed failed");
    };
    let item_after = feed_after
        .iter()
        .find(|i| i.status.id == bob_status_id)
        .expect("status still in feed after like");
    assert!(item_after.liked, "liked flag set after like");
    assert!(!item_after.boosted);

    // Unliking flips it back.
    assert_eq!(
        alice_client
            .unlike_status(alice(), bob_status_uri.clone())
            .await,
        UnlikeStatusResponse::Ok
    );

    let ReadFeedResponse::Ok(feed_unlike) = alice_client.read_feed(alice(), 0, 10).await else {
        panic!("read_feed failed");
    };
    let item_unlike = feed_unlike
        .iter()
        .find(|i| i.status.id == bob_status_id)
        .expect("status still in feed after unlike");
    assert!(!item_unlike.liked, "liked flag cleared after unlike");
}

#[pocket_ic_harness::test]
async fn test_should_surface_liked_flag_on_own_outbox_status(
    env: PocketIcTestEnv<MasticCanisterSetup>,
) {
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let alice_client = UserClient::new(&env, alice_canister);

    let resp = alice_client
        .publish_status(alice(), "Mine".to_string(), Visibility::Public, vec![])
        .await;
    let PublishStatusResponse::Ok(alice_status) = resp else {
        panic!("publish_status failed");
    };
    let alice_status_uri = status_uri("alice", alice_status.id);

    // Alice self-likes.
    assert_eq!(
        alice_client
            .like_status(alice(), alice_status_uri.clone())
            .await,
        LikeStatusResponse::Ok
    );

    let ReadFeedResponse::Ok(feed) = alice_client.read_feed(alice(), 0, 10).await else {
        panic!("read_feed failed");
    };
    let item = feed
        .iter()
        .find(|i| i.status.id == alice_status.id)
        .expect("own status in feed");
    assert!(item.liked, "self-like surfaces in feed");
    assert!(!item.boosted);
}
