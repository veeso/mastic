//! Get follow requests domain logic.

use did::user::{GetFollowRequestsArgs, GetFollowRequestsError, GetFollowRequestsResponse};

use crate::domain::follow_request::FollowRequestRepository;
use crate::error::{CanisterError, CanisterResult};

/// Gets a paginated list of pending follow requests.
///
/// Reads from the database to retrieve the list of pending follow requests for the user by offset and count.
///
/// The `limit` must not exceed [`MAX_PAGE_LIMIT`](crate::domain::MAX_PAGE_LIMIT) (50).
///
/// In case of success it returns a list of actor URIs of the users who sent the follow requests.
/// In case of error it returns an appropriate error message.
pub fn get_follow_requests(args: GetFollowRequestsArgs) -> GetFollowRequestsResponse {
    if args.limit > crate::domain::MAX_PAGE_LIMIT {
        return GetFollowRequestsResponse::Err(GetFollowRequestsError::LimitExceeded);
    }

    match inner_get_follow_requests(args) {
        Ok(handles) => GetFollowRequestsResponse::Ok(handles),
        Err(e) => GetFollowRequestsResponse::Err(e.into()),
    }
}

fn inner_get_follow_requests(
    GetFollowRequestsArgs { limit, offset }: GetFollowRequestsArgs,
) -> CanisterResult<Vec<String>> {
    FollowRequestRepository::get_paginated(offset as usize, limit as usize)
        .map(|requests| requests.into_iter().map(|r| r.actor_uri.0).collect())
}

impl From<CanisterError> for GetFollowRequestsError {
    fn from(e: CanisterError) -> Self {
        GetFollowRequestsError::Internal(e.to_string())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::domain::follow_request::FollowRequestRepository;
    use crate::test_utils::setup;

    #[test]
    fn test_should_reject_limit_exceeding_max_page_limit() {
        setup();

        let response = get_follow_requests(GetFollowRequestsArgs {
            offset: 0,
            limit: crate::domain::MAX_PAGE_LIMIT + 1,
        });

        assert_eq!(
            response,
            GetFollowRequestsResponse::Err(GetFollowRequestsError::LimitExceeded)
        );
    }

    #[test]
    fn test_should_accept_limit_at_max_page_limit() {
        setup();

        let response = get_follow_requests(GetFollowRequestsArgs {
            offset: 0,
            limit: crate::domain::MAX_PAGE_LIMIT,
        });

        let GetFollowRequestsResponse::Ok(requests) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert!(requests.is_empty());
    }

    #[test]
    fn test_should_return_empty_list_when_no_follow_requests() {
        setup();

        let response = get_follow_requests(GetFollowRequestsArgs {
            offset: 0,
            limit: 10,
        });

        let GetFollowRequestsResponse::Ok(requests) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert!(requests.is_empty());
    }

    #[test]
    fn test_should_return_follow_requests() {
        setup();

        FollowRequestRepository::insert("https://mastic.social/users/alice")
            .expect("should insert");
        FollowRequestRepository::insert("https://mastic.social/users/bob").expect("should insert");

        let response = get_follow_requests(GetFollowRequestsArgs {
            offset: 0,
            limit: 10,
        });

        let GetFollowRequestsResponse::Ok(requests) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(requests.len(), 2);
        assert!(requests.contains(&"https://mastic.social/users/alice".to_string()));
        assert!(requests.contains(&"https://mastic.social/users/bob".to_string()));
    }

    #[test]
    fn test_should_paginate_follow_requests_with_limit() {
        setup();

        FollowRequestRepository::insert("https://mastic.social/users/alice")
            .expect("should insert");
        FollowRequestRepository::insert("https://mastic.social/users/bob").expect("should insert");
        FollowRequestRepository::insert("https://mastic.social/users/charlie")
            .expect("should insert");

        let response = get_follow_requests(GetFollowRequestsArgs {
            offset: 0,
            limit: 2,
        });

        let GetFollowRequestsResponse::Ok(requests) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(requests.len(), 2);
    }

    #[test]
    fn test_should_paginate_follow_requests_with_offset() {
        setup();

        FollowRequestRepository::insert("https://mastic.social/users/alice")
            .expect("should insert");
        FollowRequestRepository::insert("https://mastic.social/users/bob").expect("should insert");
        FollowRequestRepository::insert("https://mastic.social/users/charlie")
            .expect("should insert");

        let response = get_follow_requests(GetFollowRequestsArgs {
            offset: 2,
            limit: 10,
        });

        let GetFollowRequestsResponse::Ok(requests) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(requests.len(), 1);
    }

    #[test]
    fn test_should_return_empty_list_when_offset_exceeds_total() {
        setup();

        FollowRequestRepository::insert("https://mastic.social/users/alice")
            .expect("should insert");

        let response = get_follow_requests(GetFollowRequestsArgs {
            offset: 10,
            limit: 10,
        });

        let GetFollowRequestsResponse::Ok(requests) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert!(requests.is_empty());
    }
}
