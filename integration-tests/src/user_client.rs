use candid::{Encode, Principal};
use did::common::Visibility;
use did::user::{
    AcceptFollowArgs, AcceptFollowResponse, BoostStatusArgs, BoostStatusResponse, DeleteStatusArgs,
    DeleteStatusResponse, FollowUserArgs, FollowUserResponse, GetFollowRequestsArgs,
    GetFollowRequestsResponse, GetFollowersArgs, GetFollowersResponse, GetFollowingArgs,
    GetFollowingResponse, GetLikedArgs, GetLikedResponse, GetProfileResponse, GetStatusesArgs,
    GetStatusesResponse, LikeStatusArgs, LikeStatusResponse, PublishStatusArgs,
    PublishStatusResponse, ReadFeedArgs, ReadFeedResponse, RejectFollowArgs, RejectFollowResponse,
    UndoBoostArgs, UndoBoostResponse, UnfollowUserArgs, UnfollowUserResponse, UnlikeStatusArgs,
    UnlikeStatusResponse, UpdateProfileArgs, UpdateProfileResponse,
};
use pocket_ic_harness::PocketIcTestEnv;

use crate::MasticCanisterSetup;

pub struct UserClient<'a> {
    env: &'a PocketIcTestEnv<MasticCanisterSetup>,
    canister_id: Principal,
}

impl<'a> UserClient<'a> {
    pub fn new(env: &'a PocketIcTestEnv<MasticCanisterSetup>, canister_id: Principal) -> Self {
        Self { env, canister_id }
    }
}

impl UserClient<'_> {
    pub async fn get_profile(&self, caller: Principal) -> GetProfileResponse {
        self.env
            .query(self.canister_id, caller, "get_profile", vec![])
            .await
            .expect("Failed to call get_profile")
    }

    pub async fn publish_status(
        &self,
        caller: Principal,
        content: String,
        visibility: Visibility,
        mentions: Vec<String>,
    ) -> PublishStatusResponse {
        let args = PublishStatusArgs {
            content,
            visibility,
            mentions,
        };

        self.env
            .update(
                self.canister_id,
                caller,
                "publish_status",
                Encode!(&args).expect("Failed to encode publish_status arguments"),
            )
            .await
            .expect("Failed to call publish_status")
    }

    pub async fn follow_user(&self, caller: Principal, handle: String) -> FollowUserResponse {
        let args = FollowUserArgs { handle };

        self.env
            .update(
                self.canister_id,
                caller,
                "follow_user",
                Encode!(&args).expect("Failed to encode follow_user arguments"),
            )
            .await
            .expect("Failed to call follow_user")
    }

    pub async fn get_following(
        &self,
        caller: Principal,
        offset: u64,
        limit: u64,
    ) -> GetFollowingResponse {
        let args = GetFollowingArgs { offset, limit };

        self.env
            .query(
                self.canister_id,
                caller,
                "get_following",
                Encode!(&args).expect("Failed to encode get_following arguments"),
            )
            .await
            .expect("Failed to call get_following")
    }

    pub async fn get_followers(
        &self,
        caller: Principal,
        offset: u64,
        limit: u64,
    ) -> GetFollowersResponse {
        let args = GetFollowersArgs { offset, limit };

        self.env
            .query(
                self.canister_id,
                caller,
                "get_followers",
                Encode!(&args).expect("Failed to encode get_followers arguments"),
            )
            .await
            .expect("Failed to call get_followers")
    }

    pub async fn get_follow_requests(
        &self,
        caller: Principal,
        offset: u64,
        limit: u64,
    ) -> GetFollowRequestsResponse {
        let args = GetFollowRequestsArgs { offset, limit };

        self.env
            .query(
                self.canister_id,
                caller,
                "get_follow_requests",
                Encode!(&args).expect("Failed to encode get_follow_requests arguments"),
            )
            .await
            .expect("Failed to call get_follow_requests")
    }

    pub async fn accept_follow(
        &self,
        caller: Principal,
        actor_uri: String,
    ) -> AcceptFollowResponse {
        let args = AcceptFollowArgs { actor_uri };

        self.env
            .update(
                self.canister_id,
                caller,
                "accept_follow",
                Encode!(&args).expect("Failed to encode accept_follow arguments"),
            )
            .await
            .expect("Failed to call accept_follow")
    }

    pub async fn reject_follow(
        &self,
        caller: Principal,
        actor_uri: String,
    ) -> RejectFollowResponse {
        let args = RejectFollowArgs { actor_uri };

        self.env
            .update(
                self.canister_id,
                caller,
                "reject_follow",
                Encode!(&args).expect("Failed to encode reject_follow arguments"),
            )
            .await
            .expect("Failed to call reject_follow")
    }

    pub async fn unfollow_user(
        &self,
        caller: Principal,
        actor_uri: String,
    ) -> UnfollowUserResponse {
        let args = UnfollowUserArgs { actor_uri };

        self.env
            .update(
                self.canister_id,
                caller,
                "unfollow_user",
                Encode!(&args).expect("Failed to encode unfollow_user arguments"),
            )
            .await
            .expect("Failed to call unfollow_user")
    }

    pub async fn read_feed(&self, caller: Principal, offset: u64, limit: u64) -> ReadFeedResponse {
        let args = ReadFeedArgs { offset, limit };

        self.env
            .query(
                self.canister_id,
                caller,
                "read_feed",
                Encode!(&args).expect("Failed to encode read_feed arguments"),
            )
            .await
            .expect("Failed to call read_feed")
    }

    pub async fn get_statuses(
        &self,
        caller: Principal,
        offset: u64,
        limit: u64,
    ) -> GetStatusesResponse {
        let args = GetStatusesArgs { offset, limit };

        self.env
            .query(
                self.canister_id,
                caller,
                "get_statuses",
                Encode!(&args).expect("Failed to encode get_statuses arguments"),
            )
            .await
            .expect("Failed to call get_statuses")
    }

    pub async fn like_status(&self, caller: Principal, status_url: String) -> LikeStatusResponse {
        let args = LikeStatusArgs { status_url };

        self.env
            .update(
                self.canister_id,
                caller,
                "like_status",
                Encode!(&args).expect("Failed to encode like_status arguments"),
            )
            .await
            .expect("Failed to call like_status")
    }

    pub async fn unlike_status(
        &self,
        caller: Principal,
        status_url: String,
    ) -> UnlikeStatusResponse {
        let args = UnlikeStatusArgs { status_url };

        self.env
            .update(
                self.canister_id,
                caller,
                "unlike_status",
                Encode!(&args).expect("Failed to encode unlike_status arguments"),
            )
            .await
            .expect("Failed to call unlike_status")
    }

    pub async fn boost_status(&self, caller: Principal, status_url: String) -> BoostStatusResponse {
        let args = BoostStatusArgs { status_url };

        self.env
            .update(
                self.canister_id,
                caller,
                "boost_status",
                Encode!(&args).expect("Failed to encode boost_status arguments"),
            )
            .await
            .expect("Failed to call boost_status")
    }

    pub async fn delete_status(
        &self,
        caller: Principal,
        status_uri: String,
    ) -> DeleteStatusResponse {
        let args = DeleteStatusArgs { status_uri };

        self.env
            .update(
                self.canister_id,
                caller,
                "delete_status",
                Encode!(&args).expect("Failed to encode delete_status arguments"),
            )
            .await
            .expect("Failed to call delete_status")
    }

    pub async fn undo_boost(&self, caller: Principal, status_url: String) -> UndoBoostResponse {
        let args = UndoBoostArgs { status_url };

        self.env
            .update(
                self.canister_id,
                caller,
                "undo_boost",
                Encode!(&args).expect("Failed to encode undo_boost arguments"),
            )
            .await
            .expect("Failed to call undo_boost")
    }

    pub async fn get_liked(&self, caller: Principal, offset: u64, limit: u64) -> GetLikedResponse {
        let args = GetLikedArgs { offset, limit };

        self.env
            .query(
                self.canister_id,
                caller,
                "get_liked",
                Encode!(&args).expect("Failed to encode get_liked arguments"),
            )
            .await
            .expect("Failed to call get_liked")
    }

    pub async fn update_profile(
        &self,
        caller: Principal,
        display_name: did::common::FieldUpdate<String>,
        bio: did::common::FieldUpdate<String>,
    ) -> UpdateProfileResponse {
        let args = UpdateProfileArgs { display_name, bio };

        self.env
            .update(
                self.canister_id,
                caller,
                "update_profile",
                Encode!(&args).expect("Failed to encode update_profile arguments"),
            )
            .await
            .expect("Failed to call update_profile")
    }
}
