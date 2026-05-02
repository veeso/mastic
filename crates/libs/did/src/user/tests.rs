use candid::{Decode, Encode};

use super::*;

#[test]
fn test_should_roundtrip_user_install_args_init() {
    let args = UserInstallArgs::Init {
        owner: candid::Principal::anonymous(),
        federation_canister: candid::Principal::anonymous(),
        directory_canister: candid::Principal::anonymous(),
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
        display_name: FieldUpdate::Set("Alice".to_string()),
        bio: FieldUpdate::Clear,
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
    let resp = UpdateProfileResponse::Err(UpdateProfileError::Internal("db error".to_string()));
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
    let resp = UnfollowUserResponse::Err(UnfollowUserError::Internal("db error".to_string()));
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, UnfollowUserResponse).unwrap();
    assert_eq!(resp, decoded);
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
        mentions: vec!["https://mastic.social/users/alice".to_string()],
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
        author: "https://mastic.social/users/alice".to_string(),
        created_at: 42,
        visibility: crate::common::Visibility::Public,
        like_count: 0,
        boost_count: 0,
        spoiler_text: None,
        sensitive: false,
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
        PublishStatusError::NoRecipients,
        PublishStatusError::Internal("db".to_string()),
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
        status_url: "https://example.com/users/alice/statuses/123".to_string(),
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
    let resp = LikeStatusResponse::Err(LikeStatusError::Internal("err".to_string()));
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, LikeStatusResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_undo_like_args() {
    let args = UnlikeStatusArgs {
        status_url: "https://example.com/users/alice/statuses/123".to_string(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, UnlikeStatusArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_undo_like_response_ok() {
    let resp = UnlikeStatusResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, UnlikeStatusResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_undo_like_response_err() {
    let resp = UnlikeStatusResponse::Err(UnlikeStatusError::Internal("test".to_string()));
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, UnlikeStatusResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_boost_status_args() {
    let args = BoostStatusArgs {
        status_url: "https://example.com/users/alice/statuses/123".to_string(),
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
    for error in [BoostStatusError::Internal("err".to_string())] {
        let resp = BoostStatusResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, BoostStatusResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_undo_boost_args() {
    let args = UndoBoostArgs {
        status_url: "https://example.com/users/alice/statuses/123".to_string(),
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
    for error in [UndoBoostError::Internal("err".to_string())] {
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
    let resp = GetLikedResponse::Err(GetLikedError::Internal("test".to_string()));
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, GetLikedResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_get_statuses_args() {
    let args = GetStatusesArgs {
        offset: 0,
        limit: 20,
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, GetStatusesArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_get_statuses_response_ok() {
    let resp = GetStatusesResponse::Ok(vec![crate::common::Status {
        id: 2,
        content: "Hello".to_string(),
        author: "https://mastic.social/users/alice".to_string(),
        created_at: 42,
        visibility: crate::common::Visibility::Public,
        like_count: 0,
        boost_count: 0,
        spoiler_text: None,
        sensitive: false,
    }]);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, GetStatusesResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_get_statuses_response_err() {
    for error in [
        GetStatusesError::LimitExceeded,
        GetStatusesError::Internal("db error".to_string()),
    ] {
        let resp = GetStatusesResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, GetStatusesResponse).unwrap();
        assert_eq!(resp, decoded);
    }
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
            author: "https://mastic.social/users/alice".to_string(),
            created_at: 42,
            visibility: crate::common::Visibility::Public,
            like_count: 0,
            boost_count: 0,
            spoiler_text: None,
            sensitive: false,
        },
        boosted_by: None,
    }]);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, ReadFeedResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_read_feed_response_err() {
    for error in [
        ReadFeedError::LimitExceeded,
        ReadFeedError::Internal("db error".to_string()),
    ] {
        let resp = ReadFeedResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, ReadFeedResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_get_local_status_args_with_requester() {
    let args = GetLocalStatusArgs {
        id: 123456789,
        requester_actor_uri: Some("https://mastic.social/users/alice".to_string()),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, GetLocalStatusArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_get_local_status_args_no_requester() {
    let args = GetLocalStatusArgs {
        id: 42,
        requester_actor_uri: None,
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, GetLocalStatusArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_get_local_status_response_ok() {
    let resp = GetLocalStatusResponse::Ok(crate::common::Status {
        id: 2,
        content: "Hello".to_string(),
        author: "https://mastic.social/users/alice".to_string(),
        created_at: 42,
        visibility: crate::common::Visibility::Public,
        like_count: 0,
        boost_count: 0,
        spoiler_text: None,
        sensitive: false,
    });
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, GetLocalStatusResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_get_local_status_response_err() {
    for error in [
        GetLocalStatusError::NotFound,
        GetLocalStatusError::Internal("db error".to_string()),
    ] {
        let resp = GetLocalStatusResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, GetLocalStatusResponse).unwrap();
        assert_eq!(resp, decoded);
    }
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
        ReceiveActivityError::Internal("Internal error".to_string()),
        ReceiveActivityError::InvalidActivity,
        ReceiveActivityError::ProcessingFailed,
    ] {
        let resp = ReceiveActivityResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, ReceiveActivityResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}
