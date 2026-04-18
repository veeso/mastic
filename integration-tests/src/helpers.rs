//! Helpers functions for integration tests.

use std::time::{Duration, Instant};

use candid::Principal;
use did::directory::{SignUpResponse, UserCanisterStatus, WhoAmI, WhoAmIResponse};
use pocket_ic_harness::PocketIcTestEnv;

use crate::{DirectoryClient, MasticCanisterSetup};

/// Advance pocket-ic time by `secs` seconds and tick once.
///
/// Used to make consecutive canister calls produce distinct `ic_cdk::api::time()` values
/// so tests that rely on chronological ordering can distinguish entries.
pub async fn advance_time_secs(env: &PocketIcTestEnv<MasticCanisterSetup>, secs: u64) {
    env.pic.advance_time(Duration::from_secs(secs)).await;
    env.pic.tick().await;
}

/// Build the actor URI for a local user handle.
pub fn actor_uri(handle: &str) -> String {
    format!("{url}/users/{handle}", url = crate::PUBLIC_URL)
}

/// Sign up `follower_principal` as `follower_handle` and have them follow
/// `target_handle`'s canister (`target_canister`), and accept the follow
/// request as the target (`target_principal`).
///
/// Returns the follower's user canister ID.
#[allow(clippy::too_many_arguments)]
pub async fn follow_and_accept(
    env: &PocketIcTestEnv<MasticCanisterSetup>,
    follower_principal: Principal,
    follower_handle: &str,
    target_principal: Principal,
    target_canister: Principal,
    target_handle: &str,
) -> Principal {
    use did::user::{AcceptFollowResponse, FollowUserResponse};

    use crate::UserClient;

    let follower_canister =
        sign_up_user(env, follower_principal, follower_handle.to_string()).await;
    let follower_client = UserClient::new(env, follower_canister);
    let target_client = UserClient::new(env, target_canister);

    let resp = follower_client
        .follow_user(follower_principal, target_handle.to_string())
        .await;
    assert_eq!(resp, FollowUserResponse::Ok, "follow_user failed");

    let follower_actor_uri = actor_uri(follower_handle);
    let resp = target_client
        .accept_follow(target_principal, follower_actor_uri)
        .await;
    assert_eq!(resp, AcceptFollowResponse::Ok, "accept_follow failed");

    follower_canister
}

/// Signs up a user with the given principal in the User canister and return the user canister ID.
pub async fn sign_up_user(
    env: &PocketIcTestEnv<MasticCanisterSetup>,
    user_principal: Principal,
    handle: String,
) -> Principal {
    let directory_client = DirectoryClient::new(env);

    // Sign up the user in the Directory canister to get a user canister ID
    if let SignUpResponse::Err(err) = directory_client.sign_up(user_principal, handle).await {
        panic!("Failed to sign up user: {err:?}");
    }

    let started_at = Instant::now();
    loop {
        if started_at.elapsed().as_secs() > 30 {
            panic!("Timed out waiting for user canister to be created");
        }

        match directory_client.whoami(user_principal).await {
            WhoAmIResponse::Ok(WhoAmI {
                user_canister: Some(user_canister),
                ..
            }) => {
                return user_canister;
            }
            WhoAmIResponse::Ok(WhoAmI {
                canister_status: UserCanisterStatus::CreationFailed,
                ..
            }) => {
                panic!("User canister creation failed");
            }
            WhoAmIResponse::Ok(_) => {}
            WhoAmIResponse::Err(e) => panic!("expected Ok, got Err({e:?})"),
        }
        // Sleep for a bit before retrying
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        env.pic
            .advance_time(std::time::Duration::from_secs(1))
            .await;
        env.pic.tick().await;
    }
}
