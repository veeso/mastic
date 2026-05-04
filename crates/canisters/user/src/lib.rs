mod adapters;
mod api;
mod domain;
mod error;
mod inspect;
mod schema;
mod settings;
#[cfg(test)]
mod test_utils;

use did::user::*;

#[ic_cdk::init]
fn init(args: UserInstallArgs) {
    api::init(args);
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: UserInstallArgs) {
    api::post_upgrade(args);
}

#[ic_cdk::inspect_message]
fn inspect_message() {
    inspect::inspect();
}

#[ic_cdk::update]
async fn accept_follow(args: AcceptFollowArgs) -> AcceptFollowResponse {
    api::accept_follow(args).await
}

#[ic_cdk::update]
async fn boost_status(args: BoostStatusArgs) -> BoostStatusResponse {
    api::boost_status(args).await
}

#[ic_cdk::update]
async fn delete_status(args: DeleteStatusArgs) -> DeleteStatusResponse {
    api::delete_status(args).await
}

#[ic_cdk::update]
async fn emit_delete_profile_activity() -> EmitDeleteProfileActivityResponse {
    api::emit_delete_profile_activity().await
}

#[ic_cdk::update]
async fn follow_user(args: FollowUserArgs) -> FollowUserResponse {
    api::follow_user(args).await
}

#[ic_cdk::query]
fn get_follow_requests(args: GetFollowRequestsArgs) -> GetFollowRequestsResponse {
    api::get_follow_requests(args)
}

#[ic_cdk::query]
fn get_followers(args: GetFollowersArgs) -> GetFollowersResponse {
    api::get_followers(args)
}

#[ic_cdk::query]
fn get_following(args: GetFollowingArgs) -> GetFollowingResponse {
    api::get_following(args)
}

#[ic_cdk::query]
fn get_liked(args: GetLikedArgs) -> GetLikedResponse {
    api::get_liked(args)
}

#[ic_cdk::query]
fn get_local_status(args: GetLocalStatusArgs) -> GetLocalStatusResponse {
    api::get_local_status(args)
}

#[ic_cdk::query]
fn get_profile() -> GetProfileResponse {
    api::get_profile()
}

#[ic_cdk::query(composite = true)]
async fn get_statuses(args: GetStatusesArgs) -> GetStatusesResponse {
    api::get_statuses(args).await
}

#[ic_cdk::update]
async fn like_status(args: LikeStatusArgs) -> LikeStatusResponse {
    api::like_status(args).await
}

#[ic_cdk::update]
async fn publish_status(args: PublishStatusArgs) -> PublishStatusResponse {
    api::publish_status(args).await
}

#[ic_cdk::query]
fn read_feed(args: ReadFeedArgs) -> ReadFeedResponse {
    api::read_feed(args)
}

#[ic_cdk::update]
fn receive_activity(args: ReceiveActivityArgs) -> ReceiveActivityResponse {
    api::receive_activity(args)
}

#[ic_cdk::update]
async fn reject_follow(args: RejectFollowArgs) -> RejectFollowResponse {
    api::reject_follow(args).await
}

#[ic_cdk::update]
async fn undo_boost(args: UndoBoostArgs) -> UndoBoostResponse {
    api::undo_boost(args).await
}

#[ic_cdk::update]
async fn unfollow_user(args: UnfollowUserArgs) -> UnfollowUserResponse {
    api::unfollow_user(args).await
}

#[ic_cdk::update]
async fn unlike_status(args: UnlikeStatusArgs) -> UnlikeStatusResponse {
    api::unlike_status(args).await
}

#[ic_cdk::update]
async fn update_profile(args: UpdateProfileArgs) -> UpdateProfileResponse {
    api::update_profile(args).await
}

ic_cdk::export_candid!();
