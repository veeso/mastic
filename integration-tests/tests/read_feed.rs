use did::common::Visibility;
use did::user::{PublishStatusResponse, ReadFeedResponse};
use integration_tests::helpers::{actor_uri, advance_time_secs, follow_and_accept, sign_up_user};
use integration_tests::{MasticCanisterSetup, UserClient, carol};
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
async fn test_own_statuses_appear_in_feed(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let alice_client = UserClient::new(&env, alice_canister);

    publish(&alice_client, alice(), "first", Visibility::Public, vec![]).await;
    publish(&alice_client, alice(), "second", Visibility::Public, vec![]).await;

    let ReadFeedResponse::Ok(feed) = alice_client.read_feed(alice(), 0, 10).await else {
        panic!("read_feed failed");
    };
    assert_eq!(feed.len(), 2);
    let contents: Vec<&str> = feed.iter().map(|i| i.status.content.as_str()).collect();
    assert!(contents.contains(&"first"));
    assert!(contents.contains(&"second"));
}

#[pocket_ic_harness::test]
async fn test_followed_users_statuses_appear_in_feed(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let bob_canister = sign_up_user(&env, bob(), "bob".to_string()).await;
    let alice_canister =
        follow_and_accept(&env, alice(), "alice", bob(), bob_canister, "bob").await;

    let bob_client = UserClient::new(&env, bob_canister);
    publish(&bob_client, bob(), "bob's post", Visibility::Public, vec![]).await;

    let alice_client = UserClient::new(&env, alice_canister);
    let ReadFeedResponse::Ok(feed) = alice_client.read_feed(alice(), 0, 10).await else {
        panic!("read_feed failed");
    };
    assert_eq!(feed.len(), 1);
    assert_eq!(feed[0].status.content, "bob's post");
    assert_eq!(feed[0].status.author, actor_uri("bob"));
}

#[pocket_ic_harness::test]
async fn test_feed_sorted_newest_first(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let bob_canister = sign_up_user(&env, bob(), "bob".to_string()).await;
    let alice_canister =
        follow_and_accept(&env, alice(), "alice", bob(), bob_canister, "bob").await;

    let alice_client = UserClient::new(&env, alice_canister);
    let bob_client = UserClient::new(&env, bob_canister);

    publish(
        &alice_client,
        alice(),
        "alice-1",
        Visibility::Public,
        vec![],
    )
    .await;
    advance_time_secs(&env, 2).await;
    publish(&bob_client, bob(), "bob-1", Visibility::Public, vec![]).await;
    advance_time_secs(&env, 2).await;
    publish(
        &alice_client,
        alice(),
        "alice-2",
        Visibility::Public,
        vec![],
    )
    .await;

    let ReadFeedResponse::Ok(feed) = alice_client.read_feed(alice(), 0, 10).await else {
        panic!("read_feed failed");
    };
    assert_eq!(feed.len(), 3);

    // newest-first
    for pair in feed.windows(2) {
        assert!(
            pair[0].status.created_at >= pair[1].status.created_at,
            "feed must be sorted newest-first"
        );
    }
    assert_eq!(feed[0].status.content, "alice-2");
    assert_eq!(feed[2].status.content, "alice-1");
}

#[pocket_ic_harness::test]
async fn test_feed_pagination(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let alice_client = UserClient::new(&env, alice_canister);

    for i in 0..5 {
        publish(
            &alice_client,
            alice(),
            &format!("post-{i}"),
            Visibility::Public,
            vec![],
        )
        .await;
        advance_time_secs(&env, 1).await;
    }

    let ReadFeedResponse::Ok(page1) = alice_client.read_feed(alice(), 0, 2).await else {
        panic!("page1 read_feed failed");
    };
    assert_eq!(page1.len(), 2);

    let ReadFeedResponse::Ok(page2) = alice_client.read_feed(alice(), 2, 2).await else {
        panic!("page2 read_feed failed");
    };
    assert_eq!(page2.len(), 2);

    let ReadFeedResponse::Ok(page3) = alice_client.read_feed(alice(), 4, 2).await else {
        panic!("page3 read_feed failed");
    };
    assert_eq!(page3.len(), 1);

    // No overlap between pages
    let mut all_ids: Vec<u64> = page1
        .iter()
        .chain(page2.iter())
        .chain(page3.iter())
        .map(|i| i.status.id)
        .collect();
    all_ids.sort();
    all_ids.dedup();
    assert_eq!(all_ids.len(), 5, "pages must contain distinct statuses");
}

#[pocket_ic_harness::test]
async fn test_empty_feed(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let alice_client = UserClient::new(&env, alice_canister);

    let ReadFeedResponse::Ok(feed) = alice_client.read_feed(alice(), 0, 10).await else {
        panic!("read_feed failed");
    };
    assert!(feed.is_empty());
}

#[pocket_ic_harness::test]
async fn test_direct_message_visible_to_recipient_only(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let bob_canister = sign_up_user(&env, bob(), "bob".to_string()).await;
    let carol_canister = sign_up_user(&env, carol(), "carol".to_string()).await;

    let alice_client = UserClient::new(&env, alice_canister);
    publish(
        &alice_client,
        alice(),
        "dm for bob",
        Visibility::Direct,
        vec![actor_uri("bob")],
    )
    .await;

    let bob_client = UserClient::new(&env, bob_canister);
    let ReadFeedResponse::Ok(bob_feed) = bob_client.read_feed(bob(), 0, 10).await else {
        panic!("bob read_feed failed");
    };
    assert_eq!(bob_feed.len(), 1);
    assert_eq!(bob_feed[0].status.content, "dm for bob");
    assert_eq!(bob_feed[0].status.visibility, Visibility::Direct);

    let carol_client = UserClient::new(&env, carol_canister);
    let ReadFeedResponse::Ok(carol_feed) = carol_client.read_feed(carol(), 0, 10).await else {
        panic!("carol read_feed failed");
    };
    assert!(carol_feed.is_empty(), "carol should not see the DM");
}
