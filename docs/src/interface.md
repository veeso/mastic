
# Interface

This directory contains the Candid interface definitions for the various canisters in the Mastic project.

- [Directory](./directory.did)

    ```candid
    service : (DirectoryInstallArgs) -> {
      add_moderator : (AddModeratorArgs) -> (AddModeratorResponse);
      delete_profile : () -> (DeleteProfileResponse);
      get_user : (GetUserArgs) -> (GetUserResponse) query;
      remove_moderator : (RemoveModeratorArgs) -> (RemoveModeratorResponse);
      search_profiles : (SearchProfilesArgs) -> (SearchProfilesResponse) query;
      sign_up : (text) -> (SignUpResponse);
      suspend : (SuspendArgs) -> (SuspendResponse);
      user_canister : (opt Principal) -> (UserCanisterResponse) query;
      whoami : () -> (WhoAmIResponse) query
    }
    ```

- [Federation](./federation.did)

    ```candid
    service : (FederationInstallArgs) -> {
      http_request : (HttpRequest) -> (HttpResponse) query;
      http_request_update : (HttpRequest) -> (HttpResponse);
      send_activity : (SendActivityArgs) -> (SendActivityResponse)
    }
    ```

- [User](./user.did)

    ```candid
    service : (UserInstallArgs) -> {
      accept_follow : (AcceptFollowArgs) -> (AcceptFollowResponse);
      block_user : (BlockUserArgs) -> (BlockUserResponse);
      boost_status : (BoostStatusArgs) -> (BoostStatusResponse);
      delete_profile : () -> (DeleteProfileResponse);
      delete_status : (DeleteStatusArgs) -> (DeleteStatusResponse);
      follow_user : (FollowUserArgs) -> (FollowUserResponse);
      get_follow_requests : (GetFollowRequestsArgs) -> (GetFollowRequestsResponse) query;
      get_followers : (GetFollowersArgs) -> (GetFollowersResponse) query;
      get_following : (GetFollowingArgs) -> (GetFollowingResponse) query;
      get_liked : (GetLikedArgs) -> (GetLikedResponse) query;
      get_profile : () -> (GetProfileResponse) query;
      get_statuses : (GetStatusesArgs) -> (GetStatusesResponse) composite_query;
      like_status : (LikeStatusArgs) -> (LikeStatusResponse);
      publish_status : (PublishStatusArgs) -> (PublishStatusResponse);
      read_feed : (ReadFeedArgs) -> (ReadFeedResponse) query;
      receive_activity : (ReceiveActivityArgs) -> (ReceiveActivityResponse);
      reject_follow : (RejectFollowArgs) -> (RejectFollowResponse);
      undo_boost : (UndoBoostArgs) -> (UndoBoostResponse);
      unfollow_user : (UnfollowUserArgs) -> (UnfollowUserResponse);
      unlike_status : (UnlikeStatusArgs) -> (UnlikeStatusResponse);
      update_profile : (UpdateProfileArgs) -> (UpdateProfileResponse)
    }
    ```
