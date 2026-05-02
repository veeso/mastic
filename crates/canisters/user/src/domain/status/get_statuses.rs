//! Get statuses flow logic.

use candid::Principal;
use did::common::{Status, Visibility};
use did::user::{GetStatusesArgs, GetStatusesError, GetStatusesResponse};

use crate::domain::follower::FollowerRepository;
use crate::domain::status::StatusRepository;
use crate::error::CanisterResult;

enum CallerRelationship {
    Owner,
    Follower,
    Other,
}

/// Gets a paginated list of the user's statuses.
///
/// Returned [`Status`]es are ordered from the most recent to the oldest.
///
/// Visibility is applied to the returned statuses:
///
/// - Owner: see all statuses.
/// - Follower: see `Public`, `Unlisted`, and `FollowersOnly` statuses.
/// - Anonymous/Other: see `Public` and `Unlisted` statuses.
pub async fn get_statuses(args: GetStatusesArgs, caller: Principal) -> GetStatusesResponse {
    ic_utils::log!("Getting statuses for caller: {caller}",);

    if args.limit > crate::domain::MAX_PAGE_LIMIT {
        return GetStatusesResponse::Err(GetStatusesError::LimitExceeded);
    }

    match get_statuses_inner(args, caller).await {
        Ok(response) => response,
        Err(err) => {
            ic_utils::log!("Error getting statuses: {err}");
            GetStatusesResponse::Err(GetStatusesError::Internal(err.to_string()))
        }
    }
}

/// Internal implementation of `get_statuses`, which returns a `CanisterResult` to properly handle errors.
async fn get_statuses_inner(
    GetStatusesArgs { limit, offset }: GetStatusesArgs,
    caller: Principal,
) -> CanisterResult<GetStatusesResponse> {
    let relationship = determine_relationship(caller).await?;

    // build owner actor URI
    let own_profile = crate::domain::profile::ProfileRepository::get_profile()?;
    let owner_actor_uri = crate::domain::urls::actor_uri(&own_profile.handle.0)?;

    // query statuses
    let visibility_filter = visibility_filter_for_relationship(relationship);
    StatusRepository::get_paginated_by_visibility(
        &visibility_filter,
        offset as usize,
        limit as usize,
    )
    .map(|statuses| {
        statuses
            .into_iter()
            .map(|s| status_to_did(&owner_actor_uri, s))
            .collect()
    })
    .map(GetStatusesResponse::Ok)
}

/// Determines the relationship of the caller with the user, which affects the visibility of statuses.
async fn determine_relationship(caller: Principal) -> CanisterResult<CallerRelationship> {
    // short-circuit if caller is anonymous
    if caller == Principal::anonymous() {
        ic_utils::log!("Caller is anonymous");
        return Ok(CallerRelationship::Other);
    }

    // check if caller is the owner
    if caller == crate::settings::get_owner_principal()? {
        ic_utils::log!("Caller {caller} is the owner");
        return Ok(CallerRelationship::Owner);
    }

    // resolve handle via directory
    let handle = match crate::adapters::directory::resolve_handle(caller).await? {
        Some(handle) => handle,
        None => {
            ic_utils::log!("Caller {caller} is not registered in the directory");
            return Ok(CallerRelationship::Other);
        }
    };
    ic_utils::log!("Resolved handle for caller {caller}: {handle}");

    let actor_uri = crate::domain::urls::actor_uri(&handle)?;

    if FollowerRepository::is_follower(&actor_uri)? {
        ic_utils::log!("Caller {caller} is a follower");
        return Ok(CallerRelationship::Follower);
    }

    ic_utils::log!("Caller {caller} is not a follower");
    Ok(CallerRelationship::Other)
}

/// Returns the list of [`Visibility`]es that the caller is allowed to see based on their relationship with the user.
fn visibility_filter_for_relationship(relationship: CallerRelationship) -> Vec<Visibility> {
    match relationship {
        CallerRelationship::Owner => vec![
            Visibility::Public,
            Visibility::Unlisted,
            Visibility::FollowersOnly,
            Visibility::Direct,
        ],
        CallerRelationship::Follower => vec![
            Visibility::Public,
            Visibility::Unlisted,
            Visibility::FollowersOnly,
        ],
        CallerRelationship::Other => vec![Visibility::Public, Visibility::Unlisted],
    }
}

fn status_to_did(owner_actor_uri: &str, status: crate::schema::Status) -> Status {
    Status {
        id: status.id.0,
        content: status.content.0,
        visibility: status.visibility.into(),
        created_at: status.created_at.0,
        author: owner_actor_uri.to_string(),
        like_count: status.like_count.0,
        boost_count: status.boost_count.0,
        spoiler_text: status.spoiler_text.into_opt().map(|t| t.0),
        sensitive: status.sensitive.0,
    }
}

#[cfg(test)]
mod tests {

    use did::common::Visibility;
    use did::user::{GetStatusesArgs, GetStatusesError, GetStatusesResponse};
    use ic_dbms_canister::prelude::DBMS_CONTEXT;
    use wasm_dbms::WasmDbmsDatabase;
    use wasm_dbms_api::prelude::Database;

    use super::get_statuses;
    use crate::schema::{Follower, FollowerInsertRequest, Schema};
    use crate::test_utils::{admin, alice, insert_status, setup};

    fn insert_follower(actor_uri: &str) {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Follower>(FollowerInsertRequest {
                actor_uri: actor_uri.into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert follower");
        });
    }

    fn unwrap_ok(response: GetStatusesResponse) -> Vec<did::common::Status> {
        let GetStatusesResponse::Ok(items) = response else {
            panic!("expected Ok, got {response:?}");
        };
        items
    }

    #[tokio::test]
    async fn test_should_return_empty_when_no_statuses() {
        setup();

        let items = unwrap_ok(
            get_statuses(
                GetStatusesArgs {
                    limit: 10,
                    offset: 0,
                },
                admin(),
            )
            .await,
        );

        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn test_should_return_limit_exceeded() {
        setup();

        let response = get_statuses(
            GetStatusesArgs {
                limit: crate::domain::MAX_PAGE_LIMIT + 1,
                offset: 0,
            },
            admin(),
        )
        .await;

        assert_eq!(
            response,
            GetStatusesResponse::Err(GetStatusesError::LimitExceeded)
        );
    }

    #[tokio::test]
    async fn test_should_return_statuses_ordered_by_created_at_desc() {
        setup();
        insert_status(1, "First", Visibility::Public, 1000);
        insert_status(2, "Second", Visibility::Public, 2000);
        insert_status(3, "Third", Visibility::Public, 3000);

        let items = unwrap_ok(
            get_statuses(
                GetStatusesArgs {
                    limit: 10,
                    offset: 0,
                },
                admin(),
            )
            .await,
        );

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].content, "Third");
        assert_eq!(items[1].content, "Second");
        assert_eq!(items[2].content, "First");
    }

    #[tokio::test]
    async fn test_should_paginate_with_offset_and_limit() {
        setup();
        for i in 1..=5 {
            insert_status(i, &format!("Status {i}"), Visibility::Public, i * 1000);
        }

        // skip 1, take 2 → should get statuses 4 and 3 (newest-first)
        let items = unwrap_ok(
            get_statuses(
                GetStatusesArgs {
                    limit: 2,
                    offset: 1,
                },
                admin(),
            )
            .await,
        );

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].content, "Status 4");
        assert_eq!(items[1].content, "Status 3");
    }

    #[tokio::test]
    async fn test_owner_should_see_all_visibilities() {
        setup();
        insert_status(1, "Public", Visibility::Public, 1000);
        insert_status(2, "Unlisted", Visibility::Unlisted, 2000);
        insert_status(3, "FollowersOnly", Visibility::FollowersOnly, 3000);
        insert_status(4, "Direct", Visibility::Direct, 4000);

        let items = unwrap_ok(
            get_statuses(
                GetStatusesArgs {
                    limit: 10,
                    offset: 0,
                },
                admin(),
            )
            .await,
        );

        assert_eq!(items.len(), 4);
    }

    #[tokio::test]
    async fn test_follower_should_see_public_unlisted_followers_only() {
        setup();
        // The mock directory adapter returns "testuser" for any non-owner, non-anonymous caller.
        // Insert a follower with that actor URI.
        insert_follower("https://mastic.social/users/testuser");

        insert_status(1, "Public", Visibility::Public, 1000);
        insert_status(2, "Unlisted", Visibility::Unlisted, 2000);
        insert_status(3, "FollowersOnly", Visibility::FollowersOnly, 3000);
        insert_status(4, "Direct", Visibility::Direct, 4000);

        let items = unwrap_ok(
            get_statuses(
                GetStatusesArgs {
                    limit: 10,
                    offset: 0,
                },
                alice(),
            )
            .await,
        );

        assert_eq!(items.len(), 3);
        let visibilities: Vec<_> = items.iter().map(|s| s.visibility).collect();
        assert!(visibilities.contains(&Visibility::Public));
        assert!(visibilities.contains(&Visibility::Unlisted));
        assert!(visibilities.contains(&Visibility::FollowersOnly));
        assert!(!visibilities.contains(&Visibility::Direct));
    }

    #[tokio::test]
    async fn test_anonymous_should_see_only_public_and_unlisted() {
        setup();
        insert_status(1, "Public", Visibility::Public, 1000);
        insert_status(2, "Unlisted", Visibility::Unlisted, 2000);
        insert_status(3, "FollowersOnly", Visibility::FollowersOnly, 3000);
        insert_status(4, "Direct", Visibility::Direct, 4000);

        let items = unwrap_ok(
            get_statuses(
                GetStatusesArgs {
                    limit: 10,
                    offset: 0,
                },
                candid::Principal::anonymous(),
            )
            .await,
        );

        assert_eq!(items.len(), 2);
        let visibilities: Vec<_> = items.iter().map(|s| s.visibility).collect();
        assert!(visibilities.contains(&Visibility::Public));
        assert!(visibilities.contains(&Visibility::Unlisted));
    }

    #[tokio::test]
    async fn test_non_follower_should_see_only_public_and_unlisted() {
        setup();
        // alice resolves to "testuser" via mock but is NOT in the followers table

        insert_status(1, "Public", Visibility::Public, 1000);
        insert_status(2, "Unlisted", Visibility::Unlisted, 2000);
        insert_status(3, "FollowersOnly", Visibility::FollowersOnly, 3000);
        insert_status(4, "Direct", Visibility::Direct, 4000);

        let items = unwrap_ok(
            get_statuses(
                GetStatusesArgs {
                    limit: 10,
                    offset: 0,
                },
                alice(),
            )
            .await,
        );

        assert_eq!(items.len(), 2);
        let visibilities: Vec<_> = items.iter().map(|s| s.visibility).collect();
        assert!(visibilities.contains(&Visibility::Public));
        assert!(visibilities.contains(&Visibility::Unlisted));
    }

    #[tokio::test]
    async fn test_should_set_author_to_owner_actor_uri() {
        setup();
        insert_status(1, "Hello", Visibility::Public, 1000);

        let items = unwrap_ok(
            get_statuses(
                GetStatusesArgs {
                    limit: 10,
                    offset: 0,
                },
                admin(),
            )
            .await,
        );

        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].author,
            "https://mastic.social/users/rey_canisteryo"
        );
    }

    #[tokio::test]
    async fn test_should_return_empty_page_beyond_data() {
        setup();
        insert_status(1, "Only status", Visibility::Public, 1000);

        let items = unwrap_ok(
            get_statuses(
                GetStatusesArgs {
                    limit: 10,
                    offset: 100,
                },
                admin(),
            )
            .await,
        );

        assert!(items.is_empty());
    }
}
