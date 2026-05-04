//! Domain logic for the `get_liked` query.
//!
//! Returns the paginated list of status URIs the canister owner has liked,
//! ordered as stored in the `liked` table. The response only contains URIs
//! — the canister has no authoritative copy of remote statuses, and no
//! cheap way to verify a URI still resolves on the author's instance.
//! Clients are expected to fetch each status individually if they need to
//! render content.

use db_utils::repository::Repository;
use did::user::{GetLikedArgs, GetLikedError, GetLikedResponse};

use crate::repository::liked::LikedRepository;

/// Execute the get-liked flow.
///
/// `offset` and `limit` are forwarded as a simple slice over the stored
/// rows; no `LimitExceeded` guard is enforced here because the row payload
/// (a single URI) is small and bounded by the schema validators.
pub fn get_liked(GetLikedArgs { offset, limit }: GetLikedArgs) -> GetLikedResponse {
    ic_utils::log!("Getting liked statuses with offset {offset} and limit {limit}");

    match LikedRepository::oneshot().get_liked(offset as usize, limit as usize) {
        Ok(liked) => GetLikedResponse::Ok(liked),
        Err(err) => {
            ic_utils::log!("Failed to get liked statuses: {err}");
            GetLikedResponse::Err(GetLikedError::Internal(err.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {

    use db_utils::repository::Repository;
    use did::user::{GetLikedArgs, GetLikedResponse};

    use super::get_liked;
    use crate::repository::liked::LikedRepository;
    use crate::test_utils::setup;

    #[test]
    fn test_should_return_empty_when_no_likes() {
        setup();

        let response = get_liked(GetLikedArgs {
            offset: 0,
            limit: 10,
        });
        let GetLikedResponse::Ok(liked) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert!(liked.is_empty());
    }

    #[test]
    fn test_should_return_liked_status_uris() {
        setup();
        LikedRepository::oneshot()
            .like_status("https://mastic.social/users/alice/statuses/1")
            .expect("should insert");
        LikedRepository::oneshot()
            .like_status("https://mastic.social/users/bob/statuses/2")
            .expect("should insert");

        let response = get_liked(GetLikedArgs {
            offset: 0,
            limit: 10,
        });
        let GetLikedResponse::Ok(liked) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(liked.len(), 2);
        assert!(liked.contains(&"https://mastic.social/users/alice/statuses/1".to_string()));
        assert!(liked.contains(&"https://mastic.social/users/bob/statuses/2".to_string()));
    }

    #[test]
    fn test_should_paginate_liked_results() {
        setup();
        for i in 0..5 {
            LikedRepository::oneshot()
                .like_status(&format!("https://mastic.social/users/a/statuses/{i}"))
                .expect("should insert");
        }

        let GetLikedResponse::Ok(page) = get_liked(GetLikedArgs {
            offset: 1,
            limit: 2,
        }) else {
            panic!("expected Ok");
        };
        assert_eq!(page.len(), 2);
    }
}
