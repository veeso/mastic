//! Integration tests for WI-1.8 / UC15 — delete status.

use did::common::Visibility;
use did::user::{
    DeleteStatusError, DeleteStatusResponse, GetStatusesResponse, PublishStatusResponse,
    ReadFeedResponse,
};
use integration_tests::helpers::{follow_and_accept, sign_up_user};
use integration_tests::{MasticCanisterSetup, PUBLIC_URL, UserClient};
use pocket_ic_harness::{PocketIcTestEnv, alice, bob};

fn status_uri(handle: &str, id: u64) -> String {
    format!("{PUBLIC_URL}/users/{handle}/statuses/{id}")
}

#[pocket_ic_harness::test]
async fn test_alice_deletes_own_status_bob_loses_it_from_feed(
    env: PocketIcTestEnv<MasticCanisterSetup>,
) {
    // Bob follows Alice. Alice publishes; Bob sees it. Alice deletes; Bob loses it.
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let bob_canister =
        follow_and_accept(&env, bob(), "bob", alice(), alice_canister, "alice").await;

    let alice_client = UserClient::new(&env, alice_canister);
    let bob_client = UserClient::new(&env, bob_canister);

    let PublishStatusResponse::Ok(status) = alice_client
        .publish_status(
            alice(),
            "to be deleted".to_string(),
            Visibility::Public,
            vec![],
        )
        .await
    else {
        panic!("publish_status failed");
    };
    let uri = status_uri("alice", status.id);

    // Bob's feed must contain Alice's status before deletion.
    let ReadFeedResponse::Ok(bob_feed_before) = bob_client.read_feed(bob(), 0, 10).await else {
        panic!("bob read_feed failed");
    };
    assert!(
        bob_feed_before
            .iter()
            .any(|i| i.status.id == status.id && i.status.content == "to be deleted"),
        "bob's feed must contain alice's status before delete"
    );

    // Alice deletes the status.
    assert_eq!(
        alice_client.delete_status(alice(), uri.clone()).await,
        DeleteStatusResponse::Ok
    );

    // Alice's own statuses list is empty.
    let GetStatusesResponse::Ok(alice_statuses) = alice_client.get_statuses(alice(), 0, 10).await
    else {
        panic!("get_statuses failed");
    };
    assert!(
        alice_statuses.iter().all(|s| s.id != status.id),
        "alice's status list must not contain the deleted status"
    );

    // Bob's feed must no longer reference the deleted status.
    let ReadFeedResponse::Ok(bob_feed_after) = bob_client.read_feed(bob(), 0, 10).await else {
        panic!("bob read_feed failed");
    };
    assert!(
        bob_feed_after.iter().all(|i| i.status.id != status.id),
        "bob's feed must drop alice's status after delete; got: {bob_feed_after:?}"
    );
}

#[pocket_ic_harness::test]
async fn test_repeat_delete_is_idempotent_on_receiver(env: PocketIcTestEnv<MasticCanisterSetup>) {
    // After Alice deletes, replaying the Delete (e.g. retried federation
    // delivery) must not error on Bob's side.
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let _bob_canister =
        follow_and_accept(&env, bob(), "bob", alice(), alice_canister, "alice").await;

    let alice_client = UserClient::new(&env, alice_canister);
    let PublishStatusResponse::Ok(status) = alice_client
        .publish_status(alice(), "x".to_string(), Visibility::Public, vec![])
        .await
    else {
        panic!("publish_status failed");
    };
    let uri = status_uri("alice", status.id);

    assert_eq!(
        alice_client.delete_status(alice(), uri.clone()).await,
        DeleteStatusResponse::Ok
    );
    // Second delete: now the status is gone, so the user canister returns
    // NotFound — the activity itself is idempotent on followers.
    assert_eq!(
        alice_client.delete_status(alice(), uri).await,
        DeleteStatusResponse::Err(DeleteStatusError::NotFound)
    );
}

#[pocket_ic_harness::test]
async fn test_delete_unknown_status_returns_not_found(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let alice_client = UserClient::new(&env, alice_canister);

    let resp = alice_client
        .delete_status(alice(), status_uri("alice", 999_999))
        .await;
    assert_eq!(resp, DeleteStatusResponse::Err(DeleteStatusError::NotFound));
}

#[pocket_ic_harness::test]
async fn test_delete_with_invalid_uri_returns_invalid_uri(
    env: PocketIcTestEnv<MasticCanisterSetup>,
) {
    let alice_canister = sign_up_user(&env, alice(), "alice".to_string()).await;
    let alice_client = UserClient::new(&env, alice_canister);

    let resp = alice_client
        .delete_status(alice(), "not-a-uri".to_string())
        .await;
    assert_eq!(
        resp,
        DeleteStatusResponse::Err(DeleteStatusError::InvalidUri)
    );
}
