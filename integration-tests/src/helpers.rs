//! Helpers functions for integration tests.

use std::time::Instant;

use candid::Principal;
use did::directory::{SignUpResponse, UserCanisterStatus, WhoAmI, WhoAmIResponse};
use pocket_ic_harness::PocketIcTestEnv;

use crate::{DirectoryClient, MasticCanisterSetup};

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
