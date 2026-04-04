//! Get followers domain logic.

use did::user::{GetFollowersArgs, GetFollowersError, GetFollowersResponse};

use crate::domain::follower::FollowerRepository;
use crate::error::CanisterResult;

/// Gets a paginated list of followers.
///
/// The `limit` must not exceed [`MAX_PAGE_LIMIT`](crate::domain::MAX_PAGE_LIMIT) (50).
pub fn get_followers(args: GetFollowersArgs) -> GetFollowersResponse {
    if args.limit > crate::domain::MAX_PAGE_LIMIT {
        return GetFollowersResponse::Err(GetFollowersError::LimitExceeded);
    }

    match inner_get_followers(args) {
        Ok(followers) => GetFollowersResponse::Ok(followers),
        Err(e) => GetFollowersResponse::Err(GetFollowersError::Internal(e.to_string())),
    }
}

fn inner_get_followers(
    GetFollowersArgs { offset, limit }: GetFollowersArgs,
) -> CanisterResult<Vec<String>> {
    FollowerRepository::get_paginated(offset as usize, limit as usize)
        .map(|followers| followers.into_iter().map(|f| f.actor_uri.0).collect())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::domain::follower::FollowerRepository;
    use crate::test_utils::setup;

    #[test]
    fn test_should_reject_limit_exceeding_max_page_limit() {
        setup();

        let response = get_followers(GetFollowersArgs {
            offset: 0,
            limit: crate::domain::MAX_PAGE_LIMIT + 1,
        });

        assert_eq!(
            response,
            GetFollowersResponse::Err(GetFollowersError::LimitExceeded)
        );
    }

    #[test]
    fn test_should_accept_limit_at_max_page_limit() {
        setup();

        let response = get_followers(GetFollowersArgs {
            offset: 0,
            limit: crate::domain::MAX_PAGE_LIMIT,
        });

        let GetFollowersResponse::Ok(followers) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert!(followers.is_empty());
    }

    #[test]
    fn test_should_return_empty_list_when_no_followers() {
        setup();

        let response = get_followers(GetFollowersArgs {
            offset: 0,
            limit: 10,
        });

        let GetFollowersResponse::Ok(followers) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert!(followers.is_empty());
    }

    #[test]
    fn test_should_return_followers() {
        setup();

        FollowerRepository::insert("https://mastic.social/users/alice").expect("should insert");
        FollowerRepository::insert("https://mastic.social/users/bob").expect("should insert");

        let response = get_followers(GetFollowersArgs {
            offset: 0,
            limit: 10,
        });

        let GetFollowersResponse::Ok(followers) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(followers.len(), 2);
        assert!(followers.contains(&"https://mastic.social/users/alice".to_string()));
        assert!(followers.contains(&"https://mastic.social/users/bob".to_string()));
    }

    #[test]
    fn test_should_paginate_followers_with_limit() {
        setup();

        FollowerRepository::insert("https://mastic.social/users/alice").expect("should insert");
        FollowerRepository::insert("https://mastic.social/users/bob").expect("should insert");
        FollowerRepository::insert("https://mastic.social/users/charlie").expect("should insert");

        let response = get_followers(GetFollowersArgs {
            offset: 0,
            limit: 2,
        });

        let GetFollowersResponse::Ok(followers) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(followers.len(), 2);
    }

    #[test]
    fn test_should_paginate_followers_with_offset() {
        setup();

        FollowerRepository::insert("https://mastic.social/users/alice").expect("should insert");
        FollowerRepository::insert("https://mastic.social/users/bob").expect("should insert");
        FollowerRepository::insert("https://mastic.social/users/charlie").expect("should insert");

        let response = get_followers(GetFollowersArgs {
            offset: 2,
            limit: 10,
        });

        let GetFollowersResponse::Ok(followers) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(followers.len(), 1);
    }

    #[test]
    fn test_should_return_empty_list_when_offset_exceeds_total() {
        setup();

        FollowerRepository::insert("https://mastic.social/users/alice").expect("should insert");

        let response = get_followers(GetFollowersArgs {
            offset: 10,
            limit: 10,
        });

        let GetFollowersResponse::Ok(followers) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert!(followers.is_empty());
    }
}
