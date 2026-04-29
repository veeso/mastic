//! Integration tests for the Directory `search_profiles` query (UC8).

use std::collections::HashSet;

use did::directory::{SearchProfilesArgs, SearchProfilesResponse};
use integration_tests::helpers::sign_up_user;
use integration_tests::{DirectoryClient, MasticCanisterSetup, carol, charlie};
use pocket_ic_harness::{PocketIcTestEnv, bob};

fn args(query: &str, offset: u64, limit: u64) -> SearchProfilesArgs {
    SearchProfilesArgs {
        query: query.to_string(),
        offset,
        limit,
    }
}

#[pocket_ic_harness::test]
async fn test_should_match_exact_handle(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    sign_up_user(&env, bob(), "bob".to_string()).await;
    sign_up_user(&env, charlie(), "charlie".to_string()).await;

    let SearchProfilesResponse::Ok(results) =
        client.search_profiles(bob(), args("bob", 0, 50)).await
    else {
        panic!("expected Ok");
    };
    let handles: HashSet<_> = results.iter().map(|e| e.handle.as_str()).collect();
    assert!(handles.contains("bob"));
    assert!(!handles.contains("charlie"));
}

#[pocket_ic_harness::test]
async fn test_should_match_prefix_substring(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    sign_up_user(&env, bob(), "alice".to_string()).await;
    sign_up_user(&env, charlie(), "alicia".to_string()).await;
    sign_up_user(&env, carol(), "bob_smith".to_string()).await;

    let SearchProfilesResponse::Ok(results) =
        client.search_profiles(bob(), args("ali", 0, 50)).await
    else {
        panic!("expected Ok");
    };
    let handles: HashSet<_> = results.iter().map(|e| e.handle.clone()).collect();
    assert_eq!(handles.len(), 2);
    assert!(handles.contains("alice"));
    assert!(handles.contains("alicia"));
}

#[pocket_ic_harness::test]
async fn test_should_match_middle_substring(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    sign_up_user(&env, bob(), "alice".to_string()).await;
    sign_up_user(&env, charlie(), "malice".to_string()).await;

    let SearchProfilesResponse::Ok(results) =
        client.search_profiles(bob(), args("lic", 0, 50)).await
    else {
        panic!("expected Ok");
    };
    let handles: HashSet<_> = results.iter().map(|e| e.handle.clone()).collect();
    assert!(handles.contains("alice"));
    assert!(handles.contains("malice"));
}

#[pocket_ic_harness::test]
async fn test_should_match_case_insensitively(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    sign_up_user(&env, bob(), "alice".to_string()).await;

    let SearchProfilesResponse::Ok(results) =
        client.search_profiles(bob(), args("ALICE", 0, 50)).await
    else {
        panic!("expected Ok");
    };
    let handles: HashSet<_> = results.iter().map(|e| e.handle.clone()).collect();
    assert!(handles.contains("alice"));
}

#[pocket_ic_harness::test]
async fn test_should_sanitize_at_prefix(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    sign_up_user(&env, bob(), "alice".to_string()).await;

    let SearchProfilesResponse::Ok(results) =
        client.search_profiles(bob(), args("@Alice", 0, 50)).await
    else {
        panic!("expected Ok");
    };
    let handles: HashSet<_> = results.iter().map(|e| e.handle.clone()).collect();
    assert!(handles.contains("alice"));
}

#[pocket_ic_harness::test]
async fn test_empty_query_returns_all_active(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    sign_up_user(&env, bob(), "bob".to_string()).await;
    sign_up_user(&env, charlie(), "charlie".to_string()).await;
    sign_up_user(&env, carol(), "carol".to_string()).await;

    let SearchProfilesResponse::Ok(results) = client.search_profiles(bob(), args("", 0, 50)).await
    else {
        panic!("expected Ok");
    };
    let handles: HashSet<_> = results.iter().map(|e| e.handle.clone()).collect();
    assert!(handles.contains("bob"));
    assert!(handles.contains("charlie"));
    assert!(handles.contains("carol"));
    assert_eq!(handles.len(), 3);
}

#[pocket_ic_harness::test]
async fn test_pagination(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    sign_up_user(&env, bob(), "bob".to_string()).await;
    sign_up_user(&env, charlie(), "charlie".to_string()).await;
    sign_up_user(&env, carol(), "carol".to_string()).await;

    let SearchProfilesResponse::Ok(page1) = client.search_profiles(bob(), args("", 0, 2)).await
    else {
        panic!("expected Ok");
    };
    assert_eq!(page1.len(), 2);

    let SearchProfilesResponse::Ok(page2) = client.search_profiles(bob(), args("", 2, 2)).await
    else {
        panic!("expected Ok");
    };
    assert_eq!(page2.len(), 1);

    let page1_handles: HashSet<_> = page1.iter().map(|e| e.handle.clone()).collect();
    let page2_handles: HashSet<_> = page2.iter().map(|e| e.handle.clone()).collect();
    assert!(page1_handles.is_disjoint(&page2_handles));
}

#[pocket_ic_harness::test]
async fn test_no_match_returns_empty(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let client = DirectoryClient::new(&env);

    sign_up_user(&env, bob(), "bob".to_string()).await;

    let SearchProfilesResponse::Ok(results) =
        client.search_profiles(bob(), args("zorblax", 0, 50)).await
    else {
        panic!("expected Ok");
    };
    assert!(results.is_empty());
}

#[pocket_ic_harness::test]
async fn test_should_exclude_deletion_pending(env: PocketIcTestEnv<MasticCanisterSetup>) {
    use std::time::Duration;

    use did::directory::{DeleteProfileResponse, WhoAmIError, WhoAmIResponse};

    let client = DirectoryClient::new(&env);

    sign_up_user(&env, bob(), "bob".to_string()).await;
    sign_up_user(&env, charlie(), "charlie".to_string()).await;

    // Trigger profile deletion for charlie. The state machine sets
    // canister_status = DeletionPending, then asynchronously stops/destroys
    // the canister and removes the row.
    assert_eq!(
        client.delete_profile(charlie()).await,
        DeleteProfileResponse::Ok
    );

    // Wait until charlie is fully gone.
    let started_at = std::time::Instant::now();
    loop {
        if started_at.elapsed() > Duration::from_secs(60) {
            panic!("timeout waiting for charlie deletion");
        }
        if let WhoAmIResponse::Err(WhoAmIError::NotRegistered) = client.whoami(charlie()).await {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        env.pic.advance_time(Duration::from_secs(1)).await;
        env.pic.tick().await;
    }

    // After deletion: search must not return charlie at all.
    let SearchProfilesResponse::Ok(results) = client.search_profiles(bob(), args("", 0, 50)).await
    else {
        panic!("expected Ok");
    };
    let handles: HashSet<_> = results.iter().map(|e| e.handle.clone()).collect();
    assert!(handles.contains("bob"));
    assert!(!handles.contains("charlie"));
}

/// Calls `search_profiles` directly through the env so a trap surfaces as an
/// error rather than a panic in the [`DirectoryClient`] wrapper.
async fn raw_search(
    env: &PocketIcTestEnv<MasticCanisterSetup>,
    args: SearchProfilesArgs,
) -> Result<SearchProfilesResponse, String> {
    use candid::Encode;
    use integration_tests::MasticCanister;

    let canister = env.canister_id(&MasticCanister::Directory);
    env.query(
        canister,
        bob(),
        "search_profiles",
        Encode!(&args).expect("encode"),
    )
    .await
    .map_err(|e| e.to_string())
}

#[pocket_ic_harness::test]
async fn test_should_reject_zero_limit(env: PocketIcTestEnv<MasticCanisterSetup>) {
    sign_up_user(&env, bob(), "bob".to_string()).await;

    let result = raw_search(&env, args("bob", 0, 0)).await;
    assert!(result.is_err(), "limit=0 must trap");
}

#[pocket_ic_harness::test]
async fn test_should_reject_oversize_limit(env: PocketIcTestEnv<MasticCanisterSetup>) {
    sign_up_user(&env, bob(), "bob".to_string()).await;

    let result = raw_search(&env, args("bob", 0, 51)).await;
    assert!(result.is_err(), "limit > 50 must trap");
}

#[pocket_ic_harness::test]
async fn test_anonymous_caller_can_search(env: PocketIcTestEnv<MasticCanisterSetup>) {
    use candid::Principal;

    let client = DirectoryClient::new(&env);

    sign_up_user(&env, bob(), "bob".to_string()).await;

    // Search is a public query — anonymous calls must succeed.
    let SearchProfilesResponse::Ok(results) = client
        .search_profiles(Principal::anonymous(), args("bob", 0, 50))
        .await
    else {
        panic!("expected Ok");
    };
    let handles: HashSet<_> = results.iter().map(|e| e.handle.clone()).collect();
    assert!(handles.contains("bob"));
}
