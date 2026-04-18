use did::common::Visibility;
use did::user::{PublishStatusError, PublishStatusResponse, ReadFeedResponse};
use integration_tests::helpers::{actor_uri, follow_and_accept, sign_up_user};
use integration_tests::{MasticCanisterSetup, UserClient, carol, charlie, dave, rey_canisteryo};
use pocket_ic_harness::{PocketIcTestEnv, alice, bob};

#[pocket_ic_harness::test]
async fn test_should_publish_status(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let user_canister_id = sign_up_user(&env, rey_canisteryo(), "rey_canisteryo".to_string()).await;

    let user_client = UserClient::new(&env, user_canister_id);
    if let PublishStatusResponse::Err(err) = user_client
        .publish_status(
            rey_canisteryo(),
            "Hello, World!".to_string(),
            Visibility::Public,
            vec![],
        )
        .await
    {
        panic!("Failed to publish status: {err:?}");
    }
}

#[pocket_ic_harness::test]
async fn test_should_deliver_public_status_to_follower(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let bob_canister = sign_up_user(&env, bob(), "bob".to_string()).await;
    let alice_canister =
        follow_and_accept(&env, alice(), "alice", bob(), bob_canister, "bob").await;

    let bob_client = UserClient::new(&env, bob_canister);
    let alice_client = UserClient::new(&env, alice_canister);

    let resp = bob_client
        .publish_status(bob(), "hello world".to_string(), Visibility::Public, vec![])
        .await;
    assert!(matches!(resp, PublishStatusResponse::Ok(_)), "{resp:?}");

    let ReadFeedResponse::Ok(feed) = alice_client.read_feed(alice(), 0, 10).await else {
        panic!("alice read_feed failed");
    };
    assert_eq!(feed.len(), 1);
    assert_eq!(feed[0].status.content, "hello world");
    assert_eq!(feed[0].status.author, actor_uri("bob"));
    assert_eq!(feed[0].status.visibility, Visibility::Public);
}

#[pocket_ic_harness::test]
async fn test_followers_only_not_delivered_to_strangers(env: PocketIcTestEnv<MasticCanisterSetup>) {
    // TODO: also assert follower alice sees the status once publish addressing
    // for FollowersOnly includes the owner's `/followers` collection URL.
    // Currently publish emits `to=None, cc=None` for FollowersOnly without
    // mentions, which the receiver's `infer_visibility` misclassifies as
    // Direct and then filters out because the receiver isn't addressed.
    let bob_canister = sign_up_user(&env, bob(), "bob".to_string()).await;
    let _alice_canister =
        follow_and_accept(&env, alice(), "alice", bob(), bob_canister, "bob").await;
    let charlie_canister = sign_up_user(&env, charlie(), "charlie".to_string()).await;

    let bob_client = UserClient::new(&env, bob_canister);
    let charlie_client = UserClient::new(&env, charlie_canister);

    let resp = bob_client
        .publish_status(
            bob(),
            "for my followers".to_string(),
            Visibility::FollowersOnly,
            vec![],
        )
        .await;
    assert!(matches!(resp, PublishStatusResponse::Ok(_)), "{resp:?}");

    let ReadFeedResponse::Ok(charlie_feed) = charlie_client.read_feed(charlie(), 0, 10).await
    else {
        panic!("charlie read_feed failed");
    };
    assert!(
        charlie_feed.is_empty(),
        "non-follower must not receive FollowersOnly status"
    );
}

#[pocket_ic_harness::test]
async fn test_direct_delivers_only_to_mentioned(env: PocketIcTestEnv<MasticCanisterSetup>) {
    // alice has two followers (bob, carol). alice sends Direct mentioning dave only.
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let bob_canister =
        follow_and_accept(&env, bob(), "bob", alice(), alice_canister, "alice").await;
    let carol_canister =
        follow_and_accept(&env, carol(), "carol", alice(), alice_canister, "alice").await;
    let dave_canister = sign_up_user(&env, dave(), "dave".to_string()).await;

    let alice_client = UserClient::new(&env, alice_canister);
    let bob_client = UserClient::new(&env, bob_canister);
    let carol_client = UserClient::new(&env, carol_canister);
    let dave_client = UserClient::new(&env, dave_canister);

    let resp = alice_client
        .publish_status(
            alice(),
            "psst".to_string(),
            Visibility::Direct,
            vec![actor_uri("dave")],
        )
        .await;
    assert!(matches!(resp, PublishStatusResponse::Ok(_)), "{resp:?}");

    let ReadFeedResponse::Ok(dave_feed) = dave_client.read_feed(dave(), 0, 10).await else {
        panic!("dave read_feed failed");
    };
    assert_eq!(dave_feed.len(), 1);
    assert_eq!(dave_feed[0].status.content, "psst");
    assert_eq!(dave_feed[0].status.visibility, Visibility::Direct);

    let ReadFeedResponse::Ok(bob_feed) = bob_client.read_feed(bob(), 0, 10).await else {
        panic!("bob read_feed failed");
    };
    assert!(bob_feed.is_empty(), "bob should not receive direct");

    let ReadFeedResponse::Ok(carol_feed) = carol_client.read_feed(carol(), 0, 10).await else {
        panic!("carol read_feed failed");
    };
    assert!(carol_feed.is_empty(), "carol should not receive direct");
}

#[pocket_ic_harness::test]
async fn test_direct_without_mentions_rejected(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let alice_client = UserClient::new(&env, alice_canister);

    let resp = alice_client
        .publish_status(alice(), "secret".to_string(), Visibility::Direct, vec![])
        .await;
    assert_eq!(
        resp,
        PublishStatusResponse::Err(PublishStatusError::NoRecipients)
    );
}

#[pocket_ic_harness::test]
async fn test_direct_with_multiple_mentions_no_duplicate(
    env: PocketIcTestEnv<MasticCanisterSetup>,
) {
    // carol is a follower of alice; alice sends Direct to [bob, carol].
    // Direct only targets mentions, so each gets exactly one activity —
    // no duplicate for carol even though she's also a follower.
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let carol_canister =
        follow_and_accept(&env, carol(), "carol", alice(), alice_canister, "alice").await;
    let bob_canister = sign_up_user(&env, bob(), "bob".to_string()).await;

    let alice_client = UserClient::new(&env, alice_canister);
    let bob_client = UserClient::new(&env, bob_canister);
    let carol_client = UserClient::new(&env, carol_canister);

    let resp = alice_client
        .publish_status(
            alice(),
            "hi both".to_string(),
            Visibility::Direct,
            vec![actor_uri("bob"), actor_uri("carol")],
        )
        .await;
    assert!(matches!(resp, PublishStatusResponse::Ok(_)), "{resp:?}");

    let ReadFeedResponse::Ok(bob_feed) = bob_client.read_feed(bob(), 0, 10).await else {
        panic!("bob read_feed failed");
    };
    assert_eq!(bob_feed.len(), 1);
    assert_eq!(bob_feed[0].status.content, "hi both");

    let ReadFeedResponse::Ok(carol_feed) = carol_client.read_feed(carol(), 0, 10).await else {
        panic!("carol read_feed failed");
    };
    assert_eq!(carol_feed.len(), 1, "carol should get exactly one activity");
    assert_eq!(carol_feed[0].status.content, "hi both");
}

#[pocket_ic_harness::test]
async fn test_public_fanout_to_all_followers(env: PocketIcTestEnv<MasticCanisterSetup>) {
    // alice has 3 followers (bob, carol, charlie). One Public status → all feeds include it.
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let bob_canister =
        follow_and_accept(&env, bob(), "bob", alice(), alice_canister, "alice").await;
    let carol_canister =
        follow_and_accept(&env, carol(), "carol", alice(), alice_canister, "alice").await;
    let charlie_canister =
        follow_and_accept(&env, charlie(), "charlie", alice(), alice_canister, "alice").await;

    let alice_client = UserClient::new(&env, alice_canister);
    let resp = alice_client
        .publish_status(alice(), "hi all".to_string(), Visibility::Public, vec![])
        .await;
    assert!(matches!(resp, PublishStatusResponse::Ok(_)), "{resp:?}");

    for (caller, canister, label) in [
        (bob(), bob_canister, "bob"),
        (carol(), carol_canister, "carol"),
        (charlie(), charlie_canister, "charlie"),
    ] {
        let client = UserClient::new(&env, canister);
        let ReadFeedResponse::Ok(feed) = client.read_feed(caller, 0, 10).await else {
            panic!("{label} read_feed failed");
        };
        assert_eq!(feed.len(), 1, "{label} feed should include alice's post");
        assert_eq!(feed[0].status.content, "hi all");
        assert_eq!(feed[0].status.author, actor_uri("alice"));
    }
}
