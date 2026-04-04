//! Type definitions for the User canister

#[cfg(test)]
mod tests;

use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::common::{FeedItem, Status, UserProfile, Visibility};

/// Install arguments for the User Canister.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum UserInstallArgs {
    /// Initial installation argument, provided on `init`.
    Init {
        /// The owner principal (the user's Internet Identity).
        owner: candid::Principal,
        /// Principal of the Federation Canister used for outbound ActivityPub delivery.
        federation_canister: candid::Principal,
        /// User handle
        handle: String,
        /// The public URL of the Mastic instance (e.g. `https://mastic.social`)
        public_url: String,
    },
    /// Upgrade argument, provided on `upgrade`.
    Upgrade {},
}

/// Error types for the `get_profile` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum GetProfileError {
    /// The profile has not been initialized yet.
    NotFound,
    /// Internal error occurred while fetching the profile.
    Internal(String),
}

/// Response type for the `get_profile` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum GetProfileResponse {
    Ok(UserProfile),
    Err(GetProfileError),
}

/// Request arguments for the `update_profile` method.
/// All fields are optional; only provided fields are updated.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct UpdateProfileArgs {
    /// New display name, or `None` to leave unchanged.
    pub display_name: Option<String>,
    /// New biography, or `None` to leave unchanged.
    pub bio: Option<String>,
    /// New avatar URL, or `None` to leave unchanged.
    pub avatar_url: Option<String>,
}

/// Error types for the `update_profile` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum UpdateProfileError {
    /// The caller is not the canister owner.
    Unauthorized,
}

/// Response type for the `update_profile` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum UpdateProfileResponse {
    Ok,
    Err(UpdateProfileError),
}

/// Request arguments for the `follow_user` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct FollowUserArgs {
    /// Handle of the user to follow.
    pub handle: String,
}

/// Error types for the `follow_user` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum FollowUserError {
    /// The caller is not the canister owner.
    Unauthorized,
    /// The caller already follows the target user.
    AlreadyFollowing,
    /// The caller attempted to follow their own canister.
    CannotFollowSelf,
    /// Internal error occurred while processing the follow request.
    Internal(String),
}

/// Response type for the `follow_user` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum FollowUserResponse {
    Ok,
    Err(FollowUserError),
}

/// Request arguments for the `accept_follow` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct AcceptFollowArgs {
    /// Actor URI of the user whose follow request to accept.
    pub actor_uri: String,
}

/// Error types for the `accept_follow` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum AcceptFollowError {
    /// The caller is not the canister owner.
    Unauthorized,
    /// No pending follow request exists from the given actor URI.
    RequestNotFound,
    /// Internal error occurred while processing the accept request.
    Internal(String),
}

/// Response type for the `accept_follow` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum AcceptFollowResponse {
    Ok,
    Err(AcceptFollowError),
}

/// Request arguments for the `reject_follow` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct RejectFollowArgs {
    /// Actor URI of the user whose follow request to reject.
    pub actor_uri: String,
}

/// Error types for the `reject_follow` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum RejectFollowError {
    /// The caller is not the canister owner.
    Unauthorized,
    /// No pending follow request exists from the given actor URI.
    RequestNotFound,
    /// Internal error occurred while processing the reject request.
    Internal(String),
}

/// Response type for the `reject_follow` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum RejectFollowResponse {
    Ok,
    Err(RejectFollowError),
}

/// Request arguments for the `unfollow_user` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct UnfollowUserArgs {
    /// Actor URI of the user to unfollow.
    pub actor_uri: String,
}

/// Error types for the `unfollow_user` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum UnfollowUserError {
    /// The caller is not the canister owner.
    Unauthorized,
    /// The caller does not currently follow the target user.
    NotFollowing,
    /// Internal error occurred while processing the unfollow request.
    Internal(String),
}

/// Response type for the `unfollow_user` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum UnfollowUserResponse {
    Ok,
    Err(UnfollowUserError),
}

/// Request arguments for the `block_user` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct BlockUserArgs {
    /// Actor URI of the user to block.
    pub actor_uri: String,
}

/// Error types for the `block_user` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum BlockUserError {
    /// The caller is not the canister owner.
    Unauthorized,
    /// Internal error occurred while processing the block request.
    Internal(String),
}

/// Response type for the `block_user` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum BlockUserResponse {
    Ok,
    Err(BlockUserError),
}

/// Request arguments for the `get_follow_requests` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct GetFollowRequestsArgs {
    /// Number of results to skip (for pagination).
    pub offset: u64,
    /// Maximum number of results to return.
    pub limit: u64,
}

/// Error types for the `get_follow_requests` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum GetFollowRequestsError {
    /// The requested limit exceeds the maximum allowed page size.
    LimitExceeded,
    /// Internal error occurred while querying follow requests.
    Internal(String),
}

/// Response type for the `get_follow_requests` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum GetFollowRequestsResponse {
    Ok(Vec<String>),
    Err(GetFollowRequestsError),
}

/// Request arguments for the `get_followers` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct GetFollowersArgs {
    /// Number of results to skip (for pagination).
    pub offset: u64,
    /// Maximum number of results to return.
    pub limit: u64,
}

/// Error types for the `get_followers` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum GetFollowersError {
    /// The requested limit exceeds the maximum allowed page size.
    LimitExceeded,
    /// Internal error occurred while querying followers.
    Internal(String),
}

/// Response type for the `get_followers` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum GetFollowersResponse {
    Ok(Vec<String>),
    Err(GetFollowersError),
}

/// Request arguments for the `get_following` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct GetFollowingArgs {
    /// Number of results to skip (for pagination).
    pub offset: u64,
    /// Maximum number of results to return.
    pub limit: u64,
}

/// Error types for the `get_following` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum GetFollowingError {
    /// The requested limit exceeds the maximum allowed page size.
    LimitExceeded,
    /// Internal error occurred while querying following list.
    Internal(String),
}

/// Response type for the `get_following` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum GetFollowingResponse {
    Ok(Vec<String>),
    Err(GetFollowingError),
}

/// Request arguments for the `publish_status` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct PublishStatusArgs {
    /// The text content of the new post.
    pub content: String,
    /// Audience control for this status.
    pub visibility: Visibility,
}

/// Error types for the `publish_status` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum PublishStatusError {
    /// The caller is not the canister owner.
    Unauthorized,
    /// The content is empty or contains only whitespace.
    ContentEmpty,
    /// The content exceeds the maximum allowed length.
    ContentTooLong,
    /// Internal error occurred while publishing the status.
    Internal(String),
}

/// Response type for the `publish_status` method.
/// On success, returns the created [`Status`] with its assigned ID and timestamp.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum PublishStatusResponse {
    Ok(Status),
    Err(PublishStatusError),
}

/// Request arguments for the `delete_status` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct DeleteStatusArgs {
    /// The unique ID of the status to delete.
    pub status_id: String,
}

/// Error types for the `delete_status` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum DeleteStatusError {
    /// The caller is not the canister owner.
    Unauthorized,
    /// No status exists with the given ID.
    NotFound,
}

/// Response type for the `delete_status` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum DeleteStatusResponse {
    Ok,
    Err(DeleteStatusError),
}

/// Request arguments for the `like_status` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct LikeStatusArgs {
    /// The unique ID of the status to like.
    pub status_id: String,
    /// Principal of the User Canister that authored the status.
    pub author_canister: candid::Principal,
}

/// Error types for the `like_status` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum LikeStatusError {
    /// The caller is not the canister owner.
    Unauthorized,
    /// The caller has already liked this status.
    AlreadyLiked,
}

/// Response type for the `like_status` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum LikeStatusResponse {
    Ok,
    Err(LikeStatusError),
}

/// Request arguments for the `undo_like` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct UndoLikeArgs {
    /// The unique ID of the status to unlike.
    pub status_id: String,
    /// Principal of the User Canister that authored the status.
    pub author_canister: candid::Principal,
}

/// Error types for the `undo_like` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum UndoLikeError {
    /// The caller is not the canister owner.
    Unauthorized,
    /// No like exists for the given status.
    NotFound,
}

/// Response type for the `undo_like` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum UndoLikeResponse {
    Ok,
    Err(UndoLikeError),
}

/// Request arguments for the `boost_status` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct BoostStatusArgs {
    /// The unique ID of the status to boost.
    pub status_id: String,
    /// Principal of the User Canister that authored the status.
    pub author_canister: candid::Principal,
}

/// Error types for the `boost_status` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum BoostStatusError {
    /// The caller is not the canister owner.
    Unauthorized,
    /// The caller has already boosted this status.
    AlreadyBoosted,
}

/// Response type for the `boost_status` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum BoostStatusResponse {
    Ok,
    Err(BoostStatusError),
}

/// Request arguments for the `undo_boost` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct UndoBoostArgs {
    /// The unique ID of the status to un-boost.
    pub status_id: String,
    /// Principal of the User Canister that authored the status.
    pub author_canister: candid::Principal,
}

/// Error types for the `undo_boost` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum UndoBoostError {
    /// The caller is not the canister owner.
    Unauthorized,
    /// No boost exists for the given status.
    NotFound,
}

/// Response type for the `undo_boost` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum UndoBoostResponse {
    Ok,
    Err(UndoBoostError),
}

/// Request arguments for the `get_liked` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct GetLikedArgs {
    /// Number of results to skip (for pagination).
    pub offset: u64,
    /// Maximum number of results to return.
    pub limit: u64,
}

/// Error types for the `get_liked` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum GetLikedError {
    /// The caller is not the canister owner.
    Unauthorized,
}

/// Response type for the `get_liked` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum GetLikedResponse {
    Ok(Vec<String>),
    Err(GetLikedError),
}

/// Request arguments for the `read_feed` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct ReadFeedArgs {
    /// Number of results to skip (for pagination).
    pub offset: u64,
    /// Maximum number of results to return.
    pub limit: u64,
}

/// Error types for the `read_feed` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum ReadFeedError {
    /// The caller is not the canister owner.
    Unauthorized,
}

/// Response type for the `read_feed` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum ReadFeedResponse {
    Ok(Vec<FeedItem>),
    Err(ReadFeedError),
}

/// Request arguments for the `receive_activity` method.
/// Called by the Federation Canister to deliver an incoming ActivityPub activity.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct ReceiveActivityArgs {
    /// JSON-encoded ActivityPub activity object.
    pub activity_json: String,
}

/// Error types for the `receive_activity` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum ReceiveActivityError {
    /// The JSON could not be parsed as a valid ActivityPub activity.
    InvalidActivity,
    /// The activity was valid but could not be processed (e.g. references a non-existent status).
    ProcessingFailed,
    /// Internal error occurred while handling the activity.
    Internal(String),
}

/// Response type for the `receive_activity` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum ReceiveActivityResponse {
    Ok,
    Err(ReceiveActivityError),
}
