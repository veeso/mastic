use did::user::{
    AcceptFollowResponse, FollowUserResponse, GetFollowRequestsResponse, GetFollowersResponse,
    GetFollowingResponse, RejectFollowResponse,
};
use integration_tests::{MasticCanisterSetup, PUBLIC_URL, UserClient};
use pocket_ic_harness::{PocketIcTestEnv, alice, bob};

fn actor_uri(handle: &str) -> String {
    format!("{PUBLIC_URL}/users/{handle}")
}

#[pocket_ic_harness::test]
async fn test_should_follow_and_accept_follower(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister_id =
        integration_tests::helpers::sign_up_user(&env, alice(), "alice".to_string()).await;
    let bob_canister_id =
        integration_tests::helpers::sign_up_user(&env, bob(), "bob".to_string()).await;

    let alice_client = UserClient::new(&env, alice_canister_id);
    let bob_client = UserClient::new(&env, bob_canister_id);

    let alice_uri = actor_uri("alice");
    let bob_uri = actor_uri("bob");

    // alice calls follow user "bob"
    assert_eq!(
        alice_client.follow_user(alice(), "bob".to_string()).await,
        FollowUserResponse::Ok
    );

    // bob checks his follow requests, and must have one from alice
    let GetFollowRequestsResponse::Ok(bob_follow_requests) =
        bob_client.get_follow_requests(bob(), 0, 10).await
    else {
        panic!("Failed to get follow requests for bob");
    };
    assert_eq!(bob_follow_requests.len(), 1);
    let alice_actor_uri = bob_follow_requests[0].clone();
    assert_eq!(alice_actor_uri, alice_uri);

    // bob accepts alice's follow request
    assert_eq!(
        bob_client
            .accept_follow(bob(), alice_actor_uri.clone())
            .await,
        AcceptFollowResponse::Ok
    );

    // get alice's following list, and it should include bob
    let GetFollowingResponse::Ok(following) = alice_client.get_following(alice(), 0, 10).await
    else {
        panic!("Failed to get following list for alice");
    };
    assert_eq!(following.len(), 1);
    assert_eq!(following[0], bob_uri);

    // bob's followers list should include alice
    let GetFollowersResponse::Ok(followers) = bob_client.get_followers(alice(), 0, 10).await else {
        panic!("Failed to get followers list for bob");
    };
    assert_eq!(followers.len(), 1);
    assert_eq!(followers[0], alice_uri);

    // verify the follow request is no longer in bob's follow requests
    let GetFollowRequestsResponse::Ok(bob_follow_requests) =
        bob_client.get_follow_requests(bob(), 0, 10).await
    else {
        panic!("Failed to get follow requests for bob");
    };
    assert_eq!(bob_follow_requests.len(), 0);
}

#[pocket_ic_harness::test]
async fn test_should_follow_and_reject_follower(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister_id =
        integration_tests::helpers::sign_up_user(&env, alice(), "alice".to_string()).await;
    let bob_canister_id =
        integration_tests::helpers::sign_up_user(&env, bob(), "bob".to_string()).await;

    let alice_client = UserClient::new(&env, alice_canister_id);
    let bob_client = UserClient::new(&env, bob_canister_id);

    let alice_uri = actor_uri("alice");

    // alice calls follow user "bob"
    assert_eq!(
        alice_client.follow_user(alice(), "bob".to_string()).await,
        FollowUserResponse::Ok
    );

    // bob checks his follow requests, and must have one from alice
    let GetFollowRequestsResponse::Ok(bob_follow_requests) =
        bob_client.get_follow_requests(bob(), 0, 10).await
    else {
        panic!("Failed to get follow requests for bob");
    };
    assert_eq!(bob_follow_requests.len(), 1);
    let alice_actor_uri = bob_follow_requests[0].clone();
    assert_eq!(alice_actor_uri, alice_uri);

    // bob rejects alice's follow request
    assert_eq!(
        bob_client
            .reject_follow(bob(), alice_actor_uri.clone())
            .await,
        RejectFollowResponse::Ok
    );

    // get alice's following list, and it be empty
    let GetFollowingResponse::Ok(following) = alice_client.get_following(alice(), 0, 10).await
    else {
        panic!("Failed to get following list for alice");
    };
    assert_eq!(following.len(), 0);

    // bob's followers list should be empty
    let GetFollowersResponse::Ok(followers) = bob_client.get_followers(bob(), 0, 10).await else {
        panic!("Failed to get followers list for bob");
    };
    assert_eq!(followers.len(), 0);

    // verify the follow request is no longer in bob's follow requests
    let GetFollowRequestsResponse::Ok(bob_follow_requests) =
        bob_client.get_follow_requests(bob(), 0, 10).await
    else {
        panic!("Failed to get follow requests for bob");
    };
    assert_eq!(bob_follow_requests.len(), 0);
}

#[pocket_ic_harness::test]
async fn test_should_be_able_to_refollow_after_rejection(
    env: PocketIcTestEnv<MasticCanisterSetup>,
) {
    let alice_canister_id =
        integration_tests::helpers::sign_up_user(&env, alice(), "alice".to_string()).await;
    let bob_canister_id =
        integration_tests::helpers::sign_up_user(&env, bob(), "bob".to_string()).await;

    let alice_client = UserClient::new(&env, alice_canister_id);
    let bob_client = UserClient::new(&env, bob_canister_id);

    let alice_uri = actor_uri("alice");
    let bob_uri = actor_uri("bob");

    // alice calls follow user "bob"
    assert_eq!(
        alice_client.follow_user(alice(), "bob".to_string()).await,
        FollowUserResponse::Ok
    );

    // bob checks his follow requests, and must have one from alice
    let GetFollowRequestsResponse::Ok(bob_follow_requests) =
        bob_client.get_follow_requests(bob(), 0, 10).await
    else {
        panic!("Failed to get follow requests for bob");
    };
    assert_eq!(bob_follow_requests.len(), 1);
    let alice_actor_uri = bob_follow_requests[0].clone();
    assert_eq!(alice_actor_uri, alice_uri);

    // bob rejects alice's follow request
    assert_eq!(
        bob_client
            .reject_follow(bob(), alice_actor_uri.clone())
            .await,
        RejectFollowResponse::Ok
    );

    // get alice's following list, and it be empty
    let GetFollowingResponse::Ok(following) = alice_client.get_following(alice(), 0, 10).await
    else {
        panic!("Failed to get following list for alice");
    };
    assert_eq!(following.len(), 0);

    // bob's followers list should be empty
    let GetFollowersResponse::Ok(followers) = bob_client.get_followers(bob(), 0, 10).await else {
        panic!("Failed to get followers list for bob");
    };
    assert_eq!(followers.len(), 0);

    // verify the follow request is no longer in bob's follow requests
    let GetFollowRequestsResponse::Ok(bob_follow_requests) =
        bob_client.get_follow_requests(bob(), 0, 10).await
    else {
        panic!("Failed to get follow requests for bob");
    };
    assert_eq!(bob_follow_requests.len(), 0);

    // refollow again
    assert_eq!(
        alice_client.follow_user(alice(), "bob".to_string()).await,
        FollowUserResponse::Ok
    );

    // bob should see the follow request again
    let GetFollowRequestsResponse::Ok(bob_follow_requests) =
        bob_client.get_follow_requests(bob(), 0, 10).await
    else {
        panic!("Failed to get follow requests for bob");
    };
    assert_eq!(bob_follow_requests.len(), 1);
    let alice_actor_uri = bob_follow_requests[0].clone();
    assert_eq!(alice_actor_uri, alice_uri);

    // this time bob accepts the follow request
    assert_eq!(
        bob_client
            .accept_follow(bob(), alice_actor_uri.clone())
            .await,
        AcceptFollowResponse::Ok
    );

    // get alice's following list, and it should include bob
    let GetFollowingResponse::Ok(following) = alice_client.get_following(alice(), 0, 10).await
    else {
        panic!("Failed to get following list for alice");
    };
    assert_eq!(following.len(), 1);
    assert_eq!(following[0], bob_uri);

    // bob's followers list should include alice
    let GetFollowersResponse::Ok(followers) = bob_client.get_followers(bob(), 0, 10).await else {
        panic!("Failed to get followers list for bob");
    };
    assert_eq!(followers.len(), 1);
    assert_eq!(followers[0], alice_uri);

    // verify the follow request is no longer in bob's follow requests
    let GetFollowRequestsResponse::Ok(bob_follow_requests) =
        bob_client.get_follow_requests(bob(), 0, 10).await
    else {
        panic!("Failed to get follow requests for bob");
    };
    assert_eq!(bob_follow_requests.len(), 0);
}

#[pocket_ic_harness::test]
async fn test_should_reject_self_follow(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister_id =
        integration_tests::helpers::sign_up_user(&env, alice(), "alice".to_string()).await;

    let alice_client = UserClient::new(&env, alice_canister_id);

    // alice calls follow user "alice" (self-follow)
    assert_eq!(
        alice_client.follow_user(alice(), "alice".to_string()).await,
        FollowUserResponse::Err(did::user::FollowUserError::CannotFollowSelf)
    );

    // alice's following list should be empty
    let GetFollowingResponse::Ok(following) = alice_client.get_following(alice(), 0, 10).await
    else {
        panic!("Failed to get following list for alice");
    };
    assert_eq!(following.len(), 0);

    // alice's followers list should be empty
    let GetFollowersResponse::Ok(followers) = alice_client.get_followers(alice(), 0, 10).await
    else {
        panic!("Failed to get followers list for alice");
    };
    assert_eq!(followers.len(), 0);

    // alice's follow requests should be empty
    let GetFollowRequestsResponse::Ok(follow_requests) =
        alice_client.get_follow_requests(alice(), 0, 10).await
    else {
        panic!("Failed to get follow requests for alice");
    };
    assert_eq!(follow_requests.len(), 0);
}

#[pocket_ic_harness::test]
async fn test_should_not_allow_duplicate_follow(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister_id =
        integration_tests::helpers::sign_up_user(&env, alice(), "alice".to_string()).await;
    let bob_canister_id =
        integration_tests::helpers::sign_up_user(&env, bob(), "bob".to_string()).await;

    let alice_client = UserClient::new(&env, alice_canister_id);
    let bob_client = UserClient::new(&env, bob_canister_id);

    let alice_uri = actor_uri("alice");
    let bob_uri = actor_uri("bob");

    // alice calls follow user "bob"
    assert_eq!(
        alice_client.follow_user(alice(), "bob".to_string()).await,
        FollowUserResponse::Ok
    );

    // bob checks his follow requests, and must have one from alice
    let GetFollowRequestsResponse::Ok(bob_follow_requests) =
        bob_client.get_follow_requests(bob(), 0, 10).await
    else {
        panic!("Failed to get follow requests for bob");
    };
    assert_eq!(bob_follow_requests.len(), 1);
    let alice_actor_uri = bob_follow_requests[0].clone();
    assert_eq!(alice_actor_uri, alice_uri);

    // bob accepts alice's follow request
    assert_eq!(
        bob_client
            .accept_follow(bob(), alice_actor_uri.clone())
            .await,
        AcceptFollowResponse::Ok
    );

    // get alice's following list, and it should include bob
    let GetFollowingResponse::Ok(following) = alice_client.get_following(alice(), 0, 10).await
    else {
        panic!("Failed to get following list for alice");
    };
    assert_eq!(following.len(), 1);
    assert_eq!(following[0], bob_uri);

    // bob's followers list should include alice
    let GetFollowersResponse::Ok(followers) = bob_client.get_followers(alice(), 0, 10).await else {
        panic!("Failed to get followers list for bob");
    };
    assert_eq!(followers.len(), 1);
    assert_eq!(followers[0], alice_uri);

    // verify the follow request is no longer in bob's follow requests
    let GetFollowRequestsResponse::Ok(bob_follow_requests) =
        bob_client.get_follow_requests(bob(), 0, 10).await
    else {
        panic!("Failed to get follow requests for bob");
    };
    assert_eq!(bob_follow_requests.len(), 0);

    // alice tries to follow bob again, and should get an error
    assert_eq!(
        alice_client.follow_user(alice(), "bob".to_string()).await,
        FollowUserResponse::Err(did::user::FollowUserError::AlreadyFollowing)
    );
}
