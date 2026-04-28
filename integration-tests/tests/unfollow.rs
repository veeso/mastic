use did::user::{
    AcceptFollowResponse, FollowUserResponse, GetFollowRequestsResponse, GetFollowersResponse,
    GetFollowingResponse, UnfollowUserResponse,
};
use integration_tests::{MasticCanisterSetup, PUBLIC_URL, UserClient};
use pocket_ic_harness::{PocketIcTestEnv, alice, bob};

fn actor_uri(handle: &str) -> String {
    format!("{PUBLIC_URL}/users/{handle}")
}

#[pocket_ic_harness::test]
async fn test_should_unfollow_after_accept(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister_id =
        integration_tests::helpers::sign_up_user(&env, alice(), "alice".to_string()).await;
    let bob_canister_id =
        integration_tests::helpers::sign_up_user(&env, bob(), "bob".to_string()).await;

    let alice_client = UserClient::new(&env, alice_canister_id);
    let bob_client = UserClient::new(&env, bob_canister_id);

    let alice_uri = actor_uri("alice");
    let bob_uri = actor_uri("bob");

    // alice follows bob
    assert_eq!(
        alice_client.follow_user(alice(), "bob".to_string()).await,
        FollowUserResponse::Ok
    );

    // bob accepts alice's follow request
    assert_eq!(
        bob_client.accept_follow(bob(), alice_uri.clone()).await,
        AcceptFollowResponse::Ok
    );

    // sanity: alice follows bob, bob has alice as follower
    let GetFollowingResponse::Ok(following) = alice_client.get_following(alice(), 0, 10).await
    else {
        panic!("Failed to get following list for alice");
    };
    assert_eq!(following, vec![bob_uri.clone()]);

    let GetFollowersResponse::Ok(followers) = bob_client.get_followers(bob(), 0, 10).await else {
        panic!("Failed to get followers list for bob");
    };
    assert_eq!(followers, vec![alice_uri.clone()]);

    // alice unfollows bob
    assert_eq!(
        alice_client.unfollow_user(alice(), bob_uri.clone()).await,
        UnfollowUserResponse::Ok
    );

    // alice's following list is empty
    let GetFollowingResponse::Ok(following) = alice_client.get_following(alice(), 0, 10).await
    else {
        panic!("Failed to get following list for alice");
    };
    assert!(following.is_empty());

    // bob's followers list is empty (Undo(Follow) delivered)
    let GetFollowersResponse::Ok(followers) = bob_client.get_followers(bob(), 0, 10).await else {
        panic!("Failed to get followers list for bob");
    };
    assert!(followers.is_empty());
}

#[pocket_ic_harness::test]
async fn test_should_cancel_pending_follow_on_unfollow(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister_id =
        integration_tests::helpers::sign_up_user(&env, alice(), "alice".to_string()).await;
    let bob_canister_id =
        integration_tests::helpers::sign_up_user(&env, bob(), "bob".to_string()).await;

    let alice_client = UserClient::new(&env, alice_canister_id);
    let bob_client = UserClient::new(&env, bob_canister_id);

    let alice_uri = actor_uri("alice");
    let bob_uri = actor_uri("bob");

    // alice follows bob, but bob has not accepted yet
    assert_eq!(
        alice_client.follow_user(alice(), "bob".to_string()).await,
        FollowUserResponse::Ok
    );

    // bob has alice in follow_requests
    let GetFollowRequestsResponse::Ok(bob_follow_requests) =
        bob_client.get_follow_requests(bob(), 0, 10).await
    else {
        panic!("Failed to get follow requests for bob");
    };
    assert_eq!(bob_follow_requests, vec![alice_uri.clone()]);

    // alice unfollows bob before acceptance — cancels outbound follow
    assert_eq!(
        alice_client.unfollow_user(alice(), bob_uri.clone()).await,
        UnfollowUserResponse::Ok
    );

    // alice's pending following row is gone
    let GetFollowingResponse::Ok(following) = alice_client.get_following(alice(), 0, 10).await
    else {
        panic!("Failed to get following list for alice");
    };
    assert!(following.is_empty());

    // bob's pending follow_request from alice is gone (Undo(Follow) delivered)
    let GetFollowRequestsResponse::Ok(bob_follow_requests) =
        bob_client.get_follow_requests(bob(), 0, 10).await
    else {
        panic!("Failed to get follow requests for bob");
    };
    assert!(bob_follow_requests.is_empty());

    // bob never had alice as a follower
    let GetFollowersResponse::Ok(followers) = bob_client.get_followers(bob(), 0, 10).await else {
        panic!("Failed to get followers list for bob");
    };
    assert!(followers.is_empty());
}

#[pocket_ic_harness::test]
async fn test_should_succeed_unfollow_when_not_following(
    env: PocketIcTestEnv<MasticCanisterSetup>,
) {
    let alice_canister_id =
        integration_tests::helpers::sign_up_user(&env, alice(), "alice".to_string()).await;
    let _bob_canister_id =
        integration_tests::helpers::sign_up_user(&env, bob(), "bob".to_string()).await;

    let alice_client = UserClient::new(&env, alice_canister_id);

    let bob_uri = actor_uri("bob");

    // alice never followed bob — unfollow is a silent no-op
    assert_eq!(
        alice_client.unfollow_user(alice(), bob_uri).await,
        UnfollowUserResponse::Ok
    );

    let GetFollowingResponse::Ok(following) = alice_client.get_following(alice(), 0, 10).await
    else {
        panic!("Failed to get following list for alice");
    };
    assert!(following.is_empty());
}
