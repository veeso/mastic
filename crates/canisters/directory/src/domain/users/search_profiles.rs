//! Search profiles domain flow (UC8).
//!
//! Implements the `search_profiles` query exposed by the Directory canister.
//! Argument validation lives in [`crate::api::inspect::inspect_search_profiles`]
//! and runs before this function via either `inspect_message` (update calls) or
//! a defensive trap on the query path.
//!
//! Behavior:
//! - The raw query is normalized by [`HandleSanitizer`] (strips leading `@`,
//!   trims whitespace, lowercases) — same pipeline as handle insertion, so
//!   `@Alice` matches `alice`.
//! - The repository performs a case-insensitive substring (`LIKE %x%`) match
//!   over `users.handle`, filtered to `canister_status = Active` and
//!   `canister_id IS NOT NULL`.
//! - Suspended, DeletionPending, CreationPending and CreationFailed users are
//!   excluded by construction.
//! - Empty queries return all Active users, paginated.

use db_utils::handle::HandleSanitizer;
use did::directory::{SearchProfileEntry, SearchProfilesArgs, SearchProfilesResponse};

use crate::domain::users::repository::UserRepository;

/// Implements the `search_profiles` Directory query. See module docs for
/// behavior. Returns [`SearchProfilesResponse::Err`] only on internal storage
/// errors; argument validation is upstream.
pub fn search_profiles(
    SearchProfilesArgs {
        query,
        limit,
        offset,
    }: SearchProfilesArgs,
) -> SearchProfilesResponse {
    let handle = HandleSanitizer::sanitize_handle(&query);
    ic_utils::log!(
        "search_profiles called with query: {query}, sanitized handle: {handle}, limit: {limit}, offset: {offset}"
    );

    match UserRepository::search_profiles(&handle, offset as usize, limit as usize) {
        Err(err) => {
            ic_utils::log!("Error searching profiles: {err}");
            SearchProfilesResponse::Err(did::directory::SearchProfilesError::Internal(
                err.to_string(),
            ))
        }
        Ok(profiles) => {
            ic_utils::log!("Found {} profiles for query '{query}'", profiles.len());
            SearchProfilesResponse::Ok(
                profiles
                    .into_iter()
                    .map(|profile| SearchProfileEntry {
                        handle: profile.handle.0,
                        // Repository filters `canister_id IS NOT NULL`, so this is always Some.
                        canister_id: profile.canister_id.into_opt().unwrap_or_default().0,
                    })
                    .collect(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use candid::Principal;
    use did::directory::{SearchProfilesArgs, SearchProfilesResponse, UserCanisterStatus};

    use super::*;
    use crate::domain::users::repository::UserRepository;
    use crate::test_utils::{bob, rey_canisteryo, setup, setup_registered_user_with_canister};

    fn args(query: &str, offset: u64, limit: u64) -> SearchProfilesArgs {
        SearchProfilesArgs {
            query: query.to_string(),
            offset,
            limit,
        }
    }

    #[test]
    fn test_should_return_active_users() {
        setup();
        setup_registered_user_with_canister(rey_canisteryo(), "rey_canisteryo", bob());

        let SearchProfilesResponse::Ok(results) = search_profiles(args("rey", 0, 50)) else {
            panic!("expected Ok");
        };
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].handle, "rey_canisteryo");
        assert_eq!(results[0].canister_id, bob());
    }

    #[test]
    fn test_should_sanitize_query_at_prefix() {
        setup();
        setup_registered_user_with_canister(rey_canisteryo(), "rey_canisteryo", bob());

        let SearchProfilesResponse::Ok(results) = search_profiles(args("@REY", 0, 50)) else {
            panic!("expected Ok");
        };
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].handle, "rey_canisteryo");
    }

    #[test]
    fn test_should_exclude_pending_users() {
        setup();
        // CreationPending: signed up but no canister set.
        crate::test_utils::setup_registered_user(rey_canisteryo(), "rey_canisteryo");

        let SearchProfilesResponse::Ok(results) = search_profiles(args("rey", 0, 50)) else {
            panic!("expected Ok");
        };
        assert!(results.is_empty());
    }

    #[test]
    fn test_should_paginate() {
        setup();
        setup_registered_user_with_canister(
            rey_canisteryo(),
            "rey_canisteryo",
            Principal::self_authenticating([42u8; 32]),
        );
        setup_registered_user_with_canister(bob(), "bobby_canister", bob());

        let SearchProfilesResponse::Ok(page1) = search_profiles(args("", 0, 1)) else {
            panic!("expected Ok");
        };
        assert_eq!(page1.len(), 1);

        let SearchProfilesResponse::Ok(page2) = search_profiles(args("", 1, 1)) else {
            panic!("expected Ok");
        };
        assert_eq!(page2.len(), 1);
        assert_ne!(page1[0].handle, page2[0].handle);
    }

    #[test]
    fn test_should_exclude_suspended_user() {
        setup();
        setup_registered_user_with_canister(rey_canisteryo(), "rey_canisteryo", bob());

        // Move to Suspended via repository update directly (no public flow yet).
        use ic_dbms_canister::prelude::DBMS_CONTEXT;
        use wasm_dbms::WasmDbmsDatabase;
        use wasm_dbms_api::prelude::*;

        use crate::schema::{Schema, User, UserUpdateRequest};

        let update = UserUpdateRequest {
            canister_status: Some(crate::schema::UserCanisterStatus::from(
                UserCanisterStatus::Suspended,
            )),
            where_clause: Some(Filter::eq(
                User::primary_key(),
                ic_dbms_canister::prelude::Principal(rey_canisteryo()).into(),
            )),
            ..Default::default()
        };
        DBMS_CONTEXT.with(|ctx| {
            let dbms = WasmDbmsDatabase::oneshot(ctx, Schema);
            dbms.update::<User>(update).unwrap();
        });

        let SearchProfilesResponse::Ok(results) = search_profiles(args("rey", 0, 50)) else {
            panic!("expected Ok");
        };
        assert!(results.is_empty(), "suspended user must not be returned");
    }

    #[test]
    fn test_should_exclude_deletion_pending_user() {
        setup();
        setup_registered_user_with_canister(rey_canisteryo(), "rey_canisteryo", bob());
        UserRepository::mark_user_for_deletion(rey_canisteryo()).unwrap();

        let SearchProfilesResponse::Ok(results) = search_profiles(args("rey", 0, 50)) else {
            panic!("expected Ok");
        };
        assert!(
            results.is_empty(),
            "deletion-pending user must not be returned"
        );
    }
}
