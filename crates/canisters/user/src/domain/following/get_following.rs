//! Get following domain logic.

use did::user::{GetFollowingArgs, GetFollowingError, GetFollowingResponse};

use crate::domain::following::FollowingRepository;
use crate::error::CanisterResult;

/// Gets a paginated list of following.
///
/// The `limit` must not exceed [`MAX_PAGE_LIMIT`](crate::domain::MAX_PAGE_LIMIT) (50).
pub fn get_following(args: GetFollowingArgs) -> GetFollowingResponse {
    if args.limit > crate::domain::MAX_PAGE_LIMIT {
        return GetFollowingResponse::Err(GetFollowingError::LimitExceeded);
    }

    match inner_get_following(args) {
        Ok(following) => GetFollowingResponse::Ok(following),
        Err(e) => GetFollowingResponse::Err(GetFollowingError::Internal(e.to_string())),
    }
}

fn inner_get_following(
    GetFollowingArgs { offset, limit }: GetFollowingArgs,
) -> CanisterResult<Vec<String>> {
    FollowingRepository::get_accepted_following(offset as usize, limit as usize)
        .map(|following| following.into_iter().map(|f| f.actor_uri.0).collect())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::domain::following::FollowingRepository;
    use crate::schema::FollowStatus;
    use crate::test_utils::setup;

    /// Helper: insert a pending entry and accept it.
    fn insert_accepted(actor_uri: &str) {
        FollowingRepository::insert_pending(actor_uri).expect("should insert");
        FollowingRepository::update_status(actor_uri, FollowStatus::Accepted)
            .expect("should accept");
    }

    #[test]
    fn test_should_reject_limit_exceeding_max_page_limit() {
        setup();

        let response = get_following(GetFollowingArgs {
            offset: 0,
            limit: crate::domain::MAX_PAGE_LIMIT + 1,
        });

        assert_eq!(
            response,
            GetFollowingResponse::Err(GetFollowingError::LimitExceeded)
        );
    }

    #[test]
    fn test_should_accept_limit_at_max_page_limit() {
        setup();

        let response = get_following(GetFollowingArgs {
            offset: 0,
            limit: crate::domain::MAX_PAGE_LIMIT,
        });

        let GetFollowingResponse::Ok(following) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert!(following.is_empty());
    }

    #[test]
    fn test_should_return_empty_list_when_no_following() {
        setup();

        let response = get_following(GetFollowingArgs {
            offset: 0,
            limit: 10,
        });

        let GetFollowingResponse::Ok(following) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert!(following.is_empty());
    }

    #[test]
    fn test_should_return_following() {
        setup();

        insert_accepted("https://mastic.social/users/alice");
        insert_accepted("https://mastic.social/users/bob");

        let response = get_following(GetFollowingArgs {
            offset: 0,
            limit: 10,
        });

        let GetFollowingResponse::Ok(following) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(following.len(), 2);
        assert!(following.contains(&"https://mastic.social/users/alice".to_string()));
        assert!(following.contains(&"https://mastic.social/users/bob".to_string()));
    }

    #[test]
    fn test_should_only_return_accepted_following() {
        setup();

        // insert one accepted, one pending, one rejected
        insert_accepted("https://mastic.social/users/alice");

        FollowingRepository::insert_pending("https://mastic.social/users/bob")
            .expect("should insert");
        // bob stays pending

        FollowingRepository::insert_pending("https://mastic.social/users/charlie")
            .expect("should insert");
        FollowingRepository::update_status(
            "https://mastic.social/users/charlie",
            FollowStatus::Rejected,
        )
        .expect("should reject");

        let response = get_following(GetFollowingArgs {
            offset: 0,
            limit: 10,
        });

        let GetFollowingResponse::Ok(following) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(following.len(), 1);
        assert_eq!(following[0], "https://mastic.social/users/alice");
    }

    #[test]
    fn test_should_paginate_following_with_limit() {
        setup();

        insert_accepted("https://mastic.social/users/alice");
        insert_accepted("https://mastic.social/users/bob");
        insert_accepted("https://mastic.social/users/charlie");

        let response = get_following(GetFollowingArgs {
            offset: 0,
            limit: 2,
        });

        let GetFollowingResponse::Ok(following) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(following.len(), 2);
    }

    #[test]
    fn test_should_paginate_following_with_offset() {
        setup();

        insert_accepted("https://mastic.social/users/alice");
        insert_accepted("https://mastic.social/users/bob");
        insert_accepted("https://mastic.social/users/charlie");

        let response = get_following(GetFollowingArgs {
            offset: 2,
            limit: 10,
        });

        let GetFollowingResponse::Ok(following) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(following.len(), 1);
    }

    #[test]
    fn test_should_return_empty_list_when_offset_exceeds_total() {
        setup();

        insert_accepted("https://mastic.social/users/alice");

        let response = get_following(GetFollowingArgs {
            offset: 10,
            limit: 10,
        });

        let GetFollowingResponse::Ok(following) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert!(following.is_empty());
    }
}
