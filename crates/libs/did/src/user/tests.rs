use candid::{Decode, Encode};

use super::*;

#[test]
fn test_should_roundtrip_user_install_args_init() {
    let args = UserInstallArgs::Init {
        owner: candid::Principal::anonymous(),
        federation_canister: candid::Principal::anonymous(),
        handle: "rey_canisteryo".to_string(),
        public_url: "https://mastic.social".to_string(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, UserInstallArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_user_install_args_upgrade() {
    let args = UserInstallArgs::Upgrade {};
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, UserInstallArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_get_profile_response_ok() {
    let resp = GetProfileResponse::Ok(crate::common::UserProfile {
        handle: "alice".to_string(),
        display_name: Some("Alice".to_string()),
        bio: Some("Hello".to_string()),
        avatar: None,
        header: None,
        created_at: 1_000_000_000,
    });
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, GetProfileResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_get_profile_response_err() {
    let resp = GetProfileResponse::Err(GetProfileError::NotFound);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, GetProfileResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_update_profile_args() {
    let args = UpdateProfileArgs {
        display_name: Some("Alice".to_string()),
        bio: None,
        avatar_url: Some("https://example.com/avatar.png".to_string()),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, UpdateProfileArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_update_profile_response_ok() {
    let resp = UpdateProfileResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, UpdateProfileResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_update_profile_response_err() {
    let resp = UpdateProfileResponse::Err(UpdateProfileError::Unauthorized);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, UpdateProfileResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_follow_user_args() {
    let args = FollowUserArgs {
        handle: "alice".to_string(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, FollowUserArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_follow_user_response_ok() {
    let resp = FollowUserResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, FollowUserResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_follow_user_response_err() {
    for error in [
        FollowUserError::Unauthorized,
        FollowUserError::AlreadyFollowing,
        FollowUserError::CannotFollowSelf,
    ] {
        let resp = FollowUserResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, FollowUserResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_accept_follow_args() {
    let args = AcceptFollowArgs {
        actor_uri: "https://mastic.social/users/alice".to_string(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, AcceptFollowArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_accept_follow_response_ok() {
    let resp = AcceptFollowResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, AcceptFollowResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_accept_follow_response_err() {
    for error in [
        AcceptFollowError::Unauthorized,
        AcceptFollowError::RequestNotFound,
        AcceptFollowError::Internal("db error".to_string()),
    ] {
        let resp = AcceptFollowResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, AcceptFollowResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_reject_follow_args() {
    let args = RejectFollowArgs {
        actor_uri: "https://mastic.social/users/alice".to_string(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, RejectFollowArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_reject_follow_response_ok() {
    let resp = RejectFollowResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, RejectFollowResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_reject_follow_response_err() {
    for error in [
        RejectFollowError::Unauthorized,
        RejectFollowError::RequestNotFound,
        RejectFollowError::Internal("db error".to_string()),
    ] {
        let resp = RejectFollowResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, RejectFollowResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_unfollow_user_args() {
    let args = UnfollowUserArgs {
        actor_uri: "https://mastic.social/users/alice".to_string(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, UnfollowUserArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_unfollow_user_response_ok() {
    let resp = UnfollowUserResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, UnfollowUserResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_unfollow_user_response_err() {
    for error in [
        UnfollowUserError::Unauthorized,
        UnfollowUserError::NotFollowing,
        UnfollowUserError::Internal("db error".to_string()),
    ] {
        let resp = UnfollowUserResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, UnfollowUserResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_block_user_args() {
    let args = BlockUserArgs {
        actor_uri: "https://mastic.social/users/alice".to_string(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, BlockUserArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_block_user_response_ok() {
    let resp = BlockUserResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, BlockUserResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_block_user_response_err() {
    for error in [
        BlockUserError::Unauthorized,
        BlockUserError::Internal("db error".to_string()),
    ] {
        let resp = BlockUserResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, BlockUserResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_get_follow_requests_args() {
    let args = GetFollowRequestsArgs {
        offset: 0,
        limit: 20,
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, GetFollowRequestsArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_get_follow_requests_response_ok() {
    let resp = GetFollowRequestsResponse::Ok(vec!["https://mastic.social/users/alice".to_string()]);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, GetFollowRequestsResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_get_follow_requests_response_err() {
    for error in [
        GetFollowRequestsError::LimitExceeded,
        GetFollowRequestsError::Internal("db error".to_string()),
    ] {
        let resp = GetFollowRequestsResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, GetFollowRequestsResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_get_followers_args() {
    let args = GetFollowersArgs {
        offset: 0,
        limit: 20,
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, GetFollowersArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_get_followers_response_ok() {
    let resp = GetFollowersResponse::Ok(vec!["https://mastic.social/users/alice".to_string()]);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, GetFollowersResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_get_followers_response_err() {
    for error in [
        GetFollowersError::LimitExceeded,
        GetFollowersError::Internal("db error".to_string()),
    ] {
        let resp = GetFollowersResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, GetFollowersResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_get_following_args() {
    let args = GetFollowingArgs {
        offset: 5,
        limit: 10,
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, GetFollowingArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_get_following_response_ok() {
    let resp = GetFollowingResponse::Ok(vec!["https://mastic.social/users/alice".to_string()]);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, GetFollowingResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_get_following_response_err() {
    for error in [
        GetFollowingError::LimitExceeded,
        GetFollowingError::Internal("db error".to_string()),
    ] {
        let resp = GetFollowingResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, GetFollowingResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_publish_status_args() {
    let args = PublishStatusArgs {
        content: "Hello, world!".to_string(),
        visibility: crate::common::Visibility::Public,
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, PublishStatusArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_publish_status_response_ok() {
    let resp = PublishStatusResponse::Ok(crate::common::Status {
        id: 2,
        content: "Hello".to_string(),
        author: candid::Principal::anonymous(),
        created_at: 42,
        visibility: crate::common::Visibility::Public,
    });
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, PublishStatusResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_publish_status_response_err() {
    for error in [
        PublishStatusError::Unauthorized,
        PublishStatusError::ContentEmpty,
        PublishStatusError::ContentTooLong,
    ] {
        let resp = PublishStatusResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, PublishStatusResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_delete_status_args() {
    let args = DeleteStatusArgs {
        status_id: "test-id".to_string(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, DeleteStatusArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_delete_status_response_ok() {
    let resp = DeleteStatusResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, DeleteStatusResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_delete_status_response_err() {
    for error in [DeleteStatusError::Unauthorized, DeleteStatusError::NotFound] {
        let resp = DeleteStatusResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, DeleteStatusResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_like_status_args() {
    let args = LikeStatusArgs {
        status_id: "test-id".to_string(),
        author_canister: candid::Principal::anonymous(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, LikeStatusArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_like_status_response_ok() {
    let resp = LikeStatusResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, LikeStatusResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_like_status_response_err() {
    for error in [LikeStatusError::Unauthorized, LikeStatusError::AlreadyLiked] {
        let resp = LikeStatusResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, LikeStatusResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_undo_like_args() {
    let args = UndoLikeArgs {
        status_id: "test-id".to_string(),
        author_canister: candid::Principal::anonymous(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, UndoLikeArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_undo_like_response_ok() {
    let resp = UndoLikeResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, UndoLikeResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_undo_like_response_err() {
    for error in [UndoLikeError::Unauthorized, UndoLikeError::NotFound] {
        let resp = UndoLikeResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, UndoLikeResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_boost_status_args() {
    let args = BoostStatusArgs {
        status_id: "test-id".to_string(),
        author_canister: candid::Principal::anonymous(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, BoostStatusArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_boost_status_response_ok() {
    let resp = BoostStatusResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, BoostStatusResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_boost_status_response_err() {
    for error in [
        BoostStatusError::Unauthorized,
        BoostStatusError::AlreadyBoosted,
    ] {
        let resp = BoostStatusResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, BoostStatusResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_undo_boost_args() {
    let args = UndoBoostArgs {
        status_id: "test-id".to_string(),
        author_canister: candid::Principal::anonymous(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, UndoBoostArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_undo_boost_response_ok() {
    let resp = UndoBoostResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, UndoBoostResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_undo_boost_response_err() {
    for error in [UndoBoostError::Unauthorized, UndoBoostError::NotFound] {
        let resp = UndoBoostResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, UndoBoostResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_get_liked_args() {
    let args = GetLikedArgs {
        offset: 0,
        limit: 50,
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, GetLikedArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_get_liked_response_ok() {
    let resp = GetLikedResponse::Ok(vec!["status-1".to_string(), "status-2".to_string()]);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, GetLikedResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_get_liked_response_err() {
    let resp = GetLikedResponse::Err(GetLikedError::Unauthorized);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, GetLikedResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_read_feed_args() {
    let args = ReadFeedArgs {
        offset: 0,
        limit: 20,
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, ReadFeedArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_read_feed_response_ok() {
    let resp = ReadFeedResponse::Ok(vec![crate::common::FeedItem {
        status: crate::common::Status {
            id: 2,
            content: "Hello".to_string(),
            author: candid::Principal::anonymous(),
            created_at: 42,
            visibility: crate::common::Visibility::Public,
        },
        boosted_by: None,
    }]);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, ReadFeedResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_read_feed_response_err() {
    let resp = ReadFeedResponse::Err(ReadFeedError::Unauthorized);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, ReadFeedResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_receive_activity_args() {
    let args = ReceiveActivityArgs {
        activity_json: r#"{"type":"Follow"}"#.to_string(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, ReceiveActivityArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_receive_activity_response_ok() {
    let resp = ReceiveActivityResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, ReceiveActivityResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_receive_activity_response_err() {
    for error in [
        ReceiveActivityError::Unauthorized,
        ReceiveActivityError::InvalidActivity,
        ReceiveActivityError::ProcessingFailed,
    ] {
        let resp = ReceiveActivityResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, ReceiveActivityResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}
