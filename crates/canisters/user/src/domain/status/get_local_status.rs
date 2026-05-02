//! `get_local_status` flow.
//!
//! Returns a single [`Status`] by id, applying caller-scoped visibility rules.
//! See the visibility table in the design doc:
//! - Owner: any visibility.
//! - Federation principal + `Some(requester)`: Public/Unlisted always;
//!   FollowersOnly iff `requester` is a follower; Direct → `NotFound`.
//! - Federation principal + `None`: Public/Unlisted only.
//! - Anonymous / other: Public/Unlisted only.

use candid::Principal;
use did::common::{Status, Visibility};
use did::user::{GetLocalStatusArgs, GetLocalStatusError, GetLocalStatusResponse};

use crate::domain::follower::FollowerRepository;
use crate::domain::status::StatusRepository;
use crate::domain::urls;
use crate::error::CanisterResult;

/// Public entry point. Pulls `ic_utils::caller()` and forwards to
/// [`get_local_status_with_caller`].
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "wired up by the get_local_status API endpoint")
)]
pub fn get_local_status(args: GetLocalStatusArgs) -> GetLocalStatusResponse {
    get_local_status_with_caller(ic_utils::caller(), args)
}

/// Caller-explicit entry — separated from [`get_local_status`] so tests can
/// stub the caller principal (the non-wasm `ic_utils::caller()` is fixed).
pub fn get_local_status_with_caller(
    caller: Principal,
    args: GetLocalStatusArgs,
) -> GetLocalStatusResponse {
    match get_local_status_inner(caller, args) {
        Ok(Some(status)) => GetLocalStatusResponse::Ok(status),
        Ok(None) => GetLocalStatusResponse::Err(GetLocalStatusError::NotFound),
        Err(err) => {
            ic_utils::log!("get_local_status: {err}");
            GetLocalStatusResponse::Err(GetLocalStatusError::Internal(err.to_string()))
        }
    }
}

fn get_local_status_inner(
    caller: Principal,
    GetLocalStatusArgs {
        id,
        requester_actor_uri,
    }: GetLocalStatusArgs,
) -> CanisterResult<Option<Status>> {
    let scope = resolve_scope(caller, requester_actor_uri.as_deref());
    let Some(record) = StatusRepository::find_by_id(id)? else {
        return Ok(None);
    };

    let record_visibility: Visibility = record.visibility.into();
    if !scope.allows(&record_visibility)? {
        return Ok(None);
    }

    let own_profile = crate::domain::profile::ProfileRepository::get_profile()?;
    let author_uri = urls::actor_uri(&own_profile.handle.0)?;

    Ok(Some(Status {
        id: record.id.0,
        content: record.content.0,
        author: author_uri,
        created_at: record.created_at.0,
        visibility: record_visibility,
        like_count: record.like_count.0,
        boost_count: record.boost_count.0,
        spoiler_text: record.spoiler_text.into_opt().map(|t| t.0),
        sensitive: record.sensitive.0,
    }))
}

enum Scope {
    Owner,
    FederationWithRequester(String),
    PublicOnly,
}

impl Scope {
    fn allows(&self, vis: &Visibility) -> CanisterResult<bool> {
        match (self, vis) {
            (Scope::Owner, _) => Ok(true),
            (_, Visibility::Public) | (_, Visibility::Unlisted) => Ok(true),
            (Scope::FederationWithRequester(uri), Visibility::FollowersOnly) => {
                FollowerRepository::is_follower(uri)
            }
            _ => Ok(false),
        }
    }
}

fn resolve_scope(caller: Principal, requester_actor_uri: Option<&str>) -> Scope {
    if crate::api::inspect::is_owner(caller) {
        return Scope::Owner;
    }
    if crate::api::inspect::is_federation_canister(caller) {
        return match requester_actor_uri {
            Some(uri) => Scope::FederationWithRequester(uri.to_string()),
            None => Scope::PublicOnly,
        };
    }
    Scope::PublicOnly
}

#[cfg(test)]
mod tests {
    use did::common::Visibility;
    use did::user::{GetLocalStatusArgs, GetLocalStatusError, GetLocalStatusResponse};

    use super::get_local_status_with_caller;
    use crate::domain::follower::FollowerRepository;
    use crate::test_utils::{admin, federation, insert_status, setup};

    const FOLLOWER_URI: &str = "https://remote.example/users/bob";
    const NON_FOLLOWER_URI: &str = "https://remote.example/users/charlie";

    fn insert_follower(uri: &str) {
        FollowerRepository::insert(uri).expect("insert follower");
    }

    #[test]
    fn test_owner_can_read_any_visibility() {
        setup();
        insert_status(1, "secret", Visibility::Direct, 1_000);

        let resp = get_local_status_with_caller(
            admin(),
            GetLocalStatusArgs {
                id: 1,
                requester_actor_uri: None,
            },
        );
        assert!(matches!(resp, GetLocalStatusResponse::Ok(_)));
    }

    #[test]
    fn test_federation_with_follower_uri_can_see_followers_only() {
        setup();
        insert_follower(FOLLOWER_URI);
        insert_status(2, "fo", Visibility::FollowersOnly, 1_000);

        let resp = get_local_status_with_caller(
            federation(),
            GetLocalStatusArgs {
                id: 2,
                requester_actor_uri: Some(FOLLOWER_URI.to_string()),
            },
        );
        assert!(matches!(resp, GetLocalStatusResponse::Ok(_)));
    }

    #[test]
    fn test_federation_with_non_follower_blocks_followers_only() {
        setup();
        insert_follower(FOLLOWER_URI);
        insert_status(3, "fo", Visibility::FollowersOnly, 1_000);

        let resp = get_local_status_with_caller(
            federation(),
            GetLocalStatusArgs {
                id: 3,
                requester_actor_uri: Some(NON_FOLLOWER_URI.to_string()),
            },
        );
        assert_eq!(
            resp,
            GetLocalStatusResponse::Err(GetLocalStatusError::NotFound)
        );
    }

    #[test]
    fn test_federation_without_requester_blocks_followers_only() {
        setup();
        insert_status(4, "fo", Visibility::FollowersOnly, 1_000);

        let resp = get_local_status_with_caller(
            federation(),
            GetLocalStatusArgs {
                id: 4,
                requester_actor_uri: None,
            },
        );
        assert_eq!(
            resp,
            GetLocalStatusResponse::Err(GetLocalStatusError::NotFound)
        );
    }

    #[test]
    fn test_anonymous_can_read_public() {
        setup();
        insert_status(5, "hi", Visibility::Public, 1_000);

        let resp = get_local_status_with_caller(
            candid::Principal::anonymous(),
            GetLocalStatusArgs {
                id: 5,
                requester_actor_uri: None,
            },
        );
        assert!(matches!(resp, GetLocalStatusResponse::Ok(_)));
    }

    #[test]
    fn test_direct_blocked_for_anyone_but_owner() {
        setup();
        insert_follower(FOLLOWER_URI);
        insert_status(6, "dm", Visibility::Direct, 1_000);

        let resp = get_local_status_with_caller(
            federation(),
            GetLocalStatusArgs {
                id: 6,
                requester_actor_uri: Some(FOLLOWER_URI.to_string()),
            },
        );
        assert_eq!(
            resp,
            GetLocalStatusResponse::Err(GetLocalStatusError::NotFound)
        );
    }

    #[test]
    fn test_returns_not_found_for_missing_id() {
        setup();

        let resp = get_local_status_with_caller(
            admin(),
            GetLocalStatusArgs {
                id: 999,
                requester_actor_uri: None,
            },
        );
        assert_eq!(
            resp,
            GetLocalStatusResponse::Err(GetLocalStatusError::NotFound)
        );
    }
}
