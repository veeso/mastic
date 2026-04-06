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
    }
}
