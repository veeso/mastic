//! Canister API

pub mod inspect;

use did::user::*;
use ic_dbms_canister::prelude::DBMS_CONTEXT;

use crate::schema::Schema;

/// Initializes the canister with the given arguments.
pub fn init(args: UserInstallArgs) {
    ic_utils::log!("Initializing user canister");

    let UserInstallArgs::Init {
        owner,
        federation_canister,
        directory_canister,
        handle,
        public_url,
    } = args
    else {
        ic_utils::trap!("Invalid initialization arguments");
    };

    // register database schema
    ic_utils::log!("Registering database schema");
    DBMS_CONTEXT.with(|ctx| {
        if let Err(err) = crate::schema::Schema::register_tables(ctx) {
            ic_utils::trap!("Failed to register database schema: {err}");
        }
    });

    // set owner
    ic_utils::log!("Setting owner principal to {owner}");
    if let Err(err) = crate::settings::set_owner_principal(owner) {
        ic_utils::trap!("Failed to set owner principal: {:?}", err);
    }

    // set federation canister
    ic_utils::log!("Setting federation canister to {federation_canister}");
    if let Err(err) = crate::settings::set_federation_canister(federation_canister) {
        ic_utils::trap!("Failed to set federation canister: {:?}", err);
    }

    // set directory canister
    ic_utils::log!("Setting directory canister to {directory_canister}");
    if let Err(err) = crate::settings::set_directory_canister(directory_canister) {
        ic_utils::trap!("Failed to set directory canister: {:?}", err);
    }

    // set public url
    ic_utils::log!("Setting public URL to {public_url}");
    if let Err(err) = crate::settings::set_public_url(public_url) {
        ic_utils::trap!("Failed to set public URL: {:?}", err);
    }

    // init profile
    ic_utils::log!("Creating user profile with handle {handle}");
    if let Err(err) = crate::domain::profile::create_profile(owner, &handle) {
        ic_utils::trap!("Failed to create user profile: {:?}", err);
    }

    ic_utils::log!("User canister initialized successfully for owner {owner}");
}

/// Post-upgrade function for the canister.
pub fn post_upgrade(args: UserInstallArgs) {
    ic_utils::log!("Post-upgrade user canister");

    let UserInstallArgs::Upgrade { .. } = args else {
        ic_utils::trap!("Invalid post-upgrade arguments");
    };

    db_utils::migration::run_post_upgrade_migration(&DBMS_CONTEXT, Schema);

    ic_utils::log!("User canister post-upgrade completed successfully");
}

/// Accepts a pending follow request.
///
/// This function can only be called by the owner of the canister.
pub async fn accept_follow(args: AcceptFollowArgs) -> AcceptFollowResponse {
    if !inspect::is_owner(ic_utils::caller()) {
        ic_utils::trap!("Only the owner can accept follow requests");
    }

    crate::domain::follower::accept_follow(args).await
}

/// Boost of a status.
pub async fn boost_status(args: BoostStatusArgs) -> BoostStatusResponse {
    if !inspect::is_owner(ic_utils::caller()) {
        ic_utils::trap!("Only the owner can boost statuses");
    }

    crate::domain::boost::boost_status(args).await
}

/// Emit a `Delete(Person)` activity to followers on profile deletion.
///
/// This function can only be called by the directory canister.
pub async fn emit_delete_profile_activity() -> EmitDeleteProfileActivityResponse {
    if !inspect::is_directory_canister(ic_utils::caller()) {
        ic_utils::trap!("Only the directory canister can emit delete profile activity");
    }

    crate::domain::profile::emit_delete_profile_activity().await
}

/// Follows another user.
///
/// This function can only be called by the owner of the canister.
pub async fn follow_user(args: FollowUserArgs) -> FollowUserResponse {
    if !inspect::is_owner(ic_utils::caller()) {
        ic_utils::trap!("Only the owner can follow other users");
    }

    crate::domain::following::follow_user(args).await
}

/// Gets a paginated list of pending follow requests.
pub fn get_follow_requests(args: GetFollowRequestsArgs) -> GetFollowRequestsResponse {
    if !inspect::is_owner(ic_utils::caller()) {
        ic_utils::trap!("Only the owner can view follow requests");
    }

    crate::domain::follow_request::get_follow_requests(args)
}

/// Gets a paginated list of followers.
pub fn get_followers(args: GetFollowersArgs) -> GetFollowersResponse {
    crate::domain::follower::get_followers(args)
}

/// Gets a paginated list of following.
pub fn get_following(args: GetFollowingArgs) -> GetFollowingResponse {
    crate::domain::following::get_following(args)
}

/// Gets a single status by id, applying caller-scoped visibility rules.
pub fn get_local_status(args: GetLocalStatusArgs) -> GetLocalStatusResponse {
    crate::domain::status::get_local_status(args)
}

/// Likes a status.
pub fn get_liked(args: GetLikedArgs) -> GetLikedResponse {
    crate::domain::liked::get_liked(args)
}

/// Gets the user profile.
pub fn get_profile() -> GetProfileResponse {
    crate::domain::profile::get_profile()
}

/// Gets a paginated list of the user's statuses.
pub async fn get_statuses(args: GetStatusesArgs) -> GetStatusesResponse {
    let caller = ic_utils::caller();

    crate::domain::status::get_statuses(args, caller).await
}

/// Likes a status.
pub async fn like_status(args: LikeStatusArgs) -> LikeStatusResponse {
    let caller = ic_utils::caller();
    if !inspect::is_owner(caller) {
        ic_utils::trap!("Only the owner can like statuses");
    }

    crate::domain::liked::like_status(args).await
}

/// Publishes a new status.
///
/// This function can only be called by the owner of the canister.
pub async fn publish_status(args: PublishStatusArgs) -> PublishStatusResponse {
    if !inspect::is_owner(ic_utils::caller()) {
        ic_utils::trap!("Only the owner can publish status updates");
    }

    crate::domain::status::publish_status(args).await
}

/// Reads the user's feed, which includes status updates from followed users.
pub fn read_feed(args: ReadFeedArgs) -> ReadFeedResponse {
    if !inspect::is_owner(ic_utils::caller()) {
        ic_utils::trap!("Only the owner can read their own feed");
    }

    crate::domain::feed::read_feed(args)
}

/// Handles an incoming activity from the federation canister.
pub fn receive_activity(args: ReceiveActivityArgs) -> ReceiveActivityResponse {
    if !inspect::is_federation_canister(ic_utils::caller()) {
        ic_utils::trap!("Only the federation canister can send activities");
    }

    crate::domain::activity::handle_incoming(args)
}

/// Rejects a pending follow request.
///
/// This function can only be called by the owner of the canister.
pub async fn reject_follow(args: RejectFollowArgs) -> RejectFollowResponse {
    if !inspect::is_owner(ic_utils::caller()) {
        ic_utils::trap!("Only the owner can reject follow requests");
    }

    crate::domain::follower::reject_follow(args).await
}

/// Undoes a boost of a status.
pub async fn undo_boost(args: UndoBoostArgs) -> UndoBoostResponse {
    if !inspect::is_owner(ic_utils::caller()) {
        ic_utils::trap!("Only the owner can undo boosts");
    }

    crate::domain::boost::undo_boost(args).await
}

/// Unfollows a user.
pub async fn unfollow_user(args: UnfollowUserArgs) -> UnfollowUserResponse {
    if !inspect::is_owner(ic_utils::caller()) {
        ic_utils::trap!("Only the owner can unfollow other users");
    }

    crate::domain::following::unfollow_user(args).await
}

/// Unlikes a status.
pub async fn unlike_status(args: UnlikeStatusArgs) -> UnlikeStatusResponse {
    let caller = ic_utils::caller();
    if !inspect::is_owner(caller) {
        ic_utils::trap!("Only the owner can unlike statuses");
    }

    crate::domain::liked::unlike_status(args).await
}

/// Updates the user's profile.
pub async fn update_profile(args: UpdateProfileArgs) -> UpdateProfileResponse {
    if !inspect::is_owner(ic_utils::caller()) {
        ic_utils::trap!("Only the owner can update their profile");
    }

    crate::domain::profile::update_profile(args).await
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::{admin, directory, federation, setup};

    #[test]
    fn test_should_init_canister() {
        setup();

        assert_eq!(
            crate::settings::get_owner_principal().expect("should read owner principal"),
            admin()
        );
        assert_eq!(
            crate::settings::get_federation_canister().expect("should read federation canister"),
            federation()
        );
    }

    #[test]
    #[should_panic(expected = "Invalid initialization arguments")]
    fn test_should_trap_on_init_with_upgrade_args() {
        init(UserInstallArgs::Upgrade {});
    }

    #[test]
    fn test_should_post_upgrade_with_upgrade_args() {
        setup();
        post_upgrade(UserInstallArgs::Upgrade {});
    }

    #[test]
    #[should_panic(expected = "Invalid post-upgrade arguments")]
    fn test_should_trap_on_post_upgrade_with_init_args() {
        setup();
        post_upgrade(UserInstallArgs::Init {
            owner: admin(),
            federation_canister: federation(),
            directory_canister: directory(),
            handle: "rey_canisteryo".to_string(),
            public_url: "https://mastic.social".to_string(),
        });
    }

    #[test]
    fn test_should_init_canister_with_profile() {
        setup();

        let response = get_profile();

        let GetProfileResponse::Ok(profile) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(profile.handle, "rey_canisteryo");
        assert!(profile.display_name.is_none());
        assert!(profile.bio.is_none());
        assert!(profile.avatar.is_none());
        assert!(profile.header.is_none());
    }
}
