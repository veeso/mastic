# Candid Types

- [Candid Types](#candid-types)
  - [Common Types](#common-types)
    - [Visibility](#visibility)
    - [UserProfile](#userprofile)
    - [Status](#status)
    - [FeedItem](#feeditem)
  - [Directory Canister Types](#directory-canister-types)
    - [DirectoryInstallArgs](#directoryinstallargs)
    - [SignUp](#signup)
    - [RetrySignUp](#retrysignup)
    - [UserCanisterStatus](#usercanisterstatus)
    - [WhoAmI](#whoami)
    - [UserCanister](#usercanister)
    - [GetUser](#getuser)
    - [AddModerator](#addmoderator)
    - [RemoveModerator](#removemoderator)
    - [Suspend](#suspend)
    - [SearchProfiles](#searchprofiles)
    - [DeleteProfile (Directory)](#deleteprofile-directory)
  - [User Canister Types](#user-canister-types)
    - [UserInstallArgs](#userinstallargs)
    - [GetProfile](#getprofile)
    - [UpdateProfile](#updateprofile)
    - [FollowUser](#followuser)
    - [AcceptFollow](#acceptfollow)
    - [RejectFollow](#rejectfollow)
    - [UnfollowUser](#unfollowuser)
    - [BlockUser](#blockuser)
    - [GetFollowers](#getfollowers)
    - [GetFollowing](#getfollowing)
    - [PublishStatus](#publishstatus)
    - [DeleteStatus](#deletestatus)
    - [LikeStatus](#likestatus)
    - [UnlikeStatus](#undolike)
    - [BoostStatus](#booststatus)
    - [UndoBoost](#undoboost)
    - [GetLiked](#getliked)
    - [ReadFeed](#readfeed)
    - [ReceiveActivity](#receiveactivity)
  - [Federation Canister Types](#federation-canister-types)
    - [FederationInstallArgs](#federationinstallargs)
    - [SendActivity](#sendactivity)

This document defines all shared Candid types used across the Directory, Federation, and User canisters.

## Common Types

### Visibility

Controls the audience of a status post. Maps to ActivityPub addressing:

- **Public**: visible to everyone, appears in public timelines.
- **Unlisted**: visible to everyone via direct link, but excluded from public timelines.
- **FollowersOnly**: visible only to the author's followers.
- **Direct**: visible only to explicitly mentioned users.

```candid
type Visibility = variant {
  Public;
  Unlisted;
  FollowersOnly;
  Direct;
};
```

### UserProfile

A user's public profile information. Stored in the User Canister and returned
by profile queries.

| Field          | Description                                              |
| -------------- | -------------------------------------------------------- |
| `handle`       | Unique username chosen at sign-up (e.g. `alice`).        |
| `display_name` | Optional human-readable name shown in the UI.            |
| `bio`          | Optional free-text biography.                            |
| `avatar_url`   | Optional URL pointing to the user's avatar image.        |
| `created_at`   | Timestamp (nanoseconds since epoch) of account creation. |

```candid
type UserProfile = record {
  handle : text;
  display_name : opt text;
  bio : opt text;
  avatar_url : opt text;
  created_at : nat64;
};
```

### Status

A single post authored by a user. Each status has a unique ID, content body,
author principal, creation timestamp, and visibility setting.

| Field         | Description                                                        |
| ------------- | ------------------------------------------------------------------ |
| `id`          | Snowflake identifier of the status assigned by the User Canister.  |
| `content`     | The text content of the post.                                      |
| `author`      | ActivityPub actor URI of the status author.                        |
| `created_at`  | Timestamp (milliseconds since epoch) when the status was created.  |
| `visibility`  | Audience control for this status (see [Visibility](#visibility)).  |
| `like_count`  | Cached count of `Like` activities received for this status.        |
| `boost_count` | Cached count of `Announce` (boost) activities received.            |

```candid
type Status = record {
  id : nat64;
  content : text;
  author : text;
  created_at : nat64;
  visibility : Visibility;
  like_count : nat64;
  boost_count : nat64;
};
```

### FeedItem

A single entry in a user's feed. Wraps a `Status` and optionally indicates
that it was boosted (reblogged) by another user.

| Field        | Description                                                                  |
| ------------ | ---------------------------------------------------------------------------- |
| `status`     | The status being displayed.                                                  |
| `boosted_by` | If present, the principal of the user who boosted this status into the feed. |

```candid
type FeedItem = record {
  status : Status;
  boosted_by : opt principal;
};
```

## Directory Canister Types

### DirectoryInstallArgs

Install arguments for the Directory Canister. Uses the `Init`/`Upgrade`
variant pattern required by IC canister lifecycle.

- **Init**: provided on first install. Sets the initial moderator, the
  principal of the Federation Canister this directory cooperates with,
  and the public URL of the instance.
- **Upgrade**: provided on subsequent upgrades (currently empty).

```candid
type DirectoryInstallArgs = variant {
  Init : record {
    initial_moderator : principal;
    federation_canister : principal;
    public_url : text;
  };
  Upgrade : record {};
};
```

### SignUp

Response, request and error types for the `sign_up` method. Registers a new user in the
directory, creating a User Canister and mapping the caller's principal to the
chosen handle.

- **AlreadyRegistered**: the caller already has an account.
- **HandleTaken**: the requested handle is in use by another user.
- **InvalidHandle**: the handle does not meet validation rules (e.g. length,
  allowed characters).
- **AnonymousPrincipal**: anonymous users are not allowed to sign up.
- **InternalError**: an unexpected internal error occurred.

```candid
type SignUpRequest = record {
  handle : text;
};

type SignUpResponse = variant {
  Ok;
  Err : SignUpError;
};

type SignUpError = variant {
  AlreadyRegistered;
  HandleTaken;
  InvalidHandle;
  AnonymousPrincipal;
  InternalError : text;
};
```

### RetrySignUp

Response and error types for the `retry_sign_up` method. Retries canister
creation for a user whose canister creation failed during the sign-up process.

- **NotRegistered**: the caller has no account to retry.
- **CanisterNotInFailedState**: the caller's canister is not in a failed
  state, so retrying is not allowed.
- **InternalError**: an unexpected internal error occurred.

```candid
type RetrySignUpResponse = variant {
  Ok;
  Err : RetrySignUpError;
};

type RetrySignUpError = variant {
  NotRegistered;
  CanisterNotInFailedState;
  InternalError : text;
};
```

### UserCanisterStatus

The lifecycle state of a user's canister. Used in the [WhoAmI](#whoami)
response and as the eligibility filter for [SearchProfiles](#searchprofiles).

- **Active**: the canister is created and operational.
- **CreationPending**: canister creation is in progress.
- **CreationFailed**: canister creation failed; the user may retry via
  `retry_sign_up`.
- **DeletionPending**: the user has requested account deletion and the
  canister is being torn down asynchronously.
- **Suspended**: a moderator has suspended the user; the canister exists
  but is hidden from public discovery.

```candid
type UserCanisterStatus = variant {
  Active;
  CreationPending;
  CreationFailed;
  DeletionPending;
  Suspended;
};
```

### WhoAmI

Response and error types for the `who_am_i` method. Returns the caller's handle,
User Canister ID, and canister status, allowing a logged-in user to discover
their own identity and check canister readiness.

| Field              | Description                                                                              |
| :----------------- | :--------------------------------------------------------------------------------------- |
| `handle`           | The caller's registered handle.                                                          |
| `user_canister`    | Principal of the caller's User Canister. (Optional)                                      |
| `canister_status`  | Status of the caller's User Canister (see [UserCanisterStatus](#usercanisterstatus)).    |

- **NotRegistered**: the caller has no account in the directory.

```candid
type WhoAmI = record {
  handle : text;
  user_canister : principal;
  canister_status : UserCanisterStatus;
};

type WhoAmIResponse = variant {
  Ok : WhoAmI;
  Err : WhoAmIError;
};

type WhoAmIError = variant {
  NotRegistered;
};
```

### UserCanister

Response and error types for the `user_canister` method. Resolves the caller's
principal to their User Canister ID.

- **NotRegistered**: the caller has no account in the directory.

```candid
type UserCanisterResponse = variant {
  Ok : principal;
  Err : UserCanisterError;
};

type UserCanisterError = variant {
  NotRegistered;
};
```

### GetUser

Request, response, and error types for the `get_user` method. Looks up a user
by handle and returns their handle and User Canister ID.

| Field         | Description                                  |
| ------------- | -------------------------------------------- |
| `handle`      | The handle to look up.                       |
| `canister_id` | Principal of the looked-up user's canister.  |

- **NotFound**: no user exists with the given handle.

```candid
type GetUserArgs = record {
  handle : text;
};

type GetUser = record {
  handle : text;
  canister_id : principal;
};

type GetUserResponse = variant {
  Ok : GetUser;
  Err : GetUserError;
};

type GetUserError = variant {
  NotFound;
};
```

### AddModerator

Request, response, and error types for the `add_moderator` method. Grants
moderator privileges to a principal. Only existing moderators may call this.

| Field       | Description                              |
| ----------- | ---------------------------------------- |
| `principal` | The principal to promote to moderator.   |

- **Unauthorized**: the caller is not a moderator.
- **AlreadyModerator**: the target principal is already a moderator.

```candid
type AddModeratorArgs = record {
  principal : principal;
};

type AddModeratorResponse = variant {
  Ok;
  Err : AddModeratorError;
};

type AddModeratorError = variant {
  Unauthorized;
  AlreadyModerator;
};
```

### RemoveModerator

Request, response, and error types for the `remove_moderator` method. Revokes
moderator privileges from a principal. Only existing moderators may call this.

| Field       | Description              |
| ----------- | ------------------------ |
| `principal` | The principal to demote. |

- **Unauthorized**: the caller is not a moderator.
- **NotModerator**: the target principal is not currently a moderator.

```candid
type RemoveModeratorArgs = record {
  principal : principal;
};

type RemoveModeratorResponse = variant {
  Ok;
  Err : RemoveModeratorError;
};

type RemoveModeratorError = variant {
  Unauthorized;
  NotModerator;
};
```

### Suspend

Request, response, and error types for the `suspend` method. Suspends a user
account, preventing further activity. Only moderators may call this.

| Field       | Description                             |
| ----------- | --------------------------------------- |
| `principal` | The principal of the user to suspend.   |

- **Unauthorized**: the caller is not a moderator.
- **NotFound**: no user exists with the given principal.

```candid
type SuspendArgs = record {
  principal : principal;
};

type SuspendResponse = variant {
  Ok;
  Err : SuspendError;
};

type SuspendError = variant {
  Unauthorized;
  NotFound;
};
```

### SearchProfiles

Request, response, and error types for the `search_profiles` query. Searches
registered user handles by case-insensitive substring match with pagination.

The `query` is sanitized before matching: a leading `@` is stripped,
whitespace is trimmed, and the string is lowercased — the same pipeline
applied to handles on insert. So `@Alice`, `alice`, and `  ALICE  ` all
match the handle `alice`.

Only users with `canister_status = Active` and a non-null `canister_id`
are returned. `CreationPending`, `CreationFailed`, `DeletionPending`, and
`Suspended` users are excluded by construction.

An empty `query` returns all eligible users, paginated.

| Field    | Description                                              |
| :------- | :------------------------------------------------------- |
| `query`  | Free-text search string matched against handles.         |
| `offset` | Number of results to skip (for pagination).              |
| `limit`  | Maximum results to return; must be in `1..=50`.          |

Each result entry contains:

| Field         | Description                                |
| :------------ | :----------------------------------------- |
| `handle`      | The matched user's handle.                 |
| `canister_id` | Principal of the matched user's canister.  |

Errors:

- **BadArgs**: the request was rejected (`limit == 0`, `limit > 50`, or
  the sanitized query failed handle validation). In practice the canister
  traps on these inputs at message-inspect time; the variant is reserved
  for parity with other endpoints.
- **Internal**: an internal storage error occurred while running the
  query.

```candid
type SearchProfilesArgs = record {
  query : text;
  offset : nat64;
  limit : nat64;
};

type SearchProfileEntry = record {
  handle : text;
  canister_id : principal;
};

type SearchProfilesResponse = variant {
  Ok : vec SearchProfileEntry;
  Err : SearchProfilesError;
};

type SearchProfilesError = variant {
  BadArgs;
  Internal : text;
};
```

### DeleteProfile (Directory)

Response and error types for the `delete_profile` method on the Directory
Canister. Removes the caller's account and handle mapping from the directory.

- **NotRegistered**: the caller has no account to delete.

```candid
type DeleteProfileResponse = variant {
  Ok;
  Err : DeleteProfileError;
};

type DeleteProfileError = variant {
  NotRegistered;
};
```

## User Canister Types

### UserInstallArgs

Install arguments for the User Canister. Uses the `Init`/`Upgrade` variant
pattern required by IC canister lifecycle.

- **Init**: provided on first install. Sets the owner principal (the user's
  Internet Identity), the Federation Canister principal used for outbound
  ActivityPub delivery, the user handle, and the instance public URL.
- **Upgrade**: provided on subsequent upgrades (currently empty).

```candid
type UserInstallArgs = variant {
  Init : record {
    owner : principal;
    federation_canister : principal;
    handle : text;
    public_url : text;
  };
  Upgrade : record {};
};
```

### GetProfile

Response and error types for the `get_profile` method. Returns the full
`UserProfile` for this canister's owner.

- **NotFound**: the profile has not been initialized yet.

```candid
type GetProfileResponse = variant {
  Ok : UserProfile;
  Err : GetProfileError;
};

type GetProfileError = variant {
  NotFound;
};
```

### UpdateProfile

Request, response, and error types for the `update_profile` method. Updates
the caller's profile fields. Only the canister owner may call this. All fields
are optional; only provided fields are updated.

| Field          | Description                                     |
| -------------- | ----------------------------------------------- |
| `display_name` | New display name, or `null` to leave unchanged. |
| `bio`          | New biography, or `null` to leave unchanged.    |
| `avatar_url`   | New avatar URL, or `null` to leave unchanged.   |

- **Unauthorized**: the caller is not the canister owner.

```candid
type UpdateProfileArgs = record {
  display_name : opt text;
  bio : opt text;
  avatar_url : opt text;
};

type UpdateProfileResponse = variant {
  Ok;
  Err : UpdateProfileError;
};

type UpdateProfileError = variant {
  Unauthorized;
};
```

### FollowUser

Request, response, and error types for the `follow_user` method. Sends a
follow request to another user by handle.

| Field    | Description                    |
| -------- | ------------------------------ |
| `handle` | Handle of the user to follow.  |

- **Unauthorized**: the caller is not the canister owner.
- **AlreadyFollowing**: the caller already follows the target user.
- **CannotFollowSelf**: the caller attempted to follow themselves.
- **Internal**: an internal error occurred while processing the request.

```candid
type FollowUserArgs = record {
  handle : text;
};

type FollowUserResponse = variant {
  Ok;
  Err : FollowUserError;
};

type FollowUserError = variant {
  Unauthorized;
  AlreadyFollowing;
  CannotFollowSelf;
  Internal : text;
};
```

### AcceptFollow

Request, response, and error types for the `accept_follow` method. Accepts a
pending follow request from another user, adding them to the followers list.

| Field      | Description                                                     |
| ---------- | --------------------------------------------------------------- |
| `follower` | Principal of the User Canister whose follow request to accept.  |

- **Unauthorized**: the caller is not the canister owner.
- **RequestNotFound**: no pending follow request exists from the given principal.

```candid
type AcceptFollowArgs = record {
  follower : principal;
};

type AcceptFollowResponse = variant {
  Ok;
  Err : AcceptFollowError;
};

type AcceptFollowError = variant {
  Unauthorized;
  RequestNotFound;
};
```

### RejectFollow

Request, response, and error types for the `reject_follow` method. Rejects a
pending follow request from another user.

| Field      | Description                                                     |
| ---------- | --------------------------------------------------------------- |
| `follower` | Principal of the User Canister whose follow request to reject.  |

- **Unauthorized**: the caller is not the canister owner.
- **RequestNotFound**: no pending follow request exists from the given principal.

```candid
type RejectFollowArgs = record {
  follower : principal;
};

type RejectFollowResponse = variant {
  Ok;
  Err : RejectFollowError;
};

type RejectFollowError = variant {
  Unauthorized;
  RequestNotFound;
};
```

### UnfollowUser

Request, response, and error types for the `unfollow_user` method. Removes the
caller from the target user's followers list and removes the target from the
caller's following list.

| Field         | Description                                  |
| ------------- | -------------------------------------------- |
| `canister_id` | Principal of the User Canister to unfollow.  |

- **Unauthorized**: the caller is not the canister owner.
- **NotFollowing**: the caller does not currently follow the target user.

```candid
type UnfollowUserArgs = record {
  canister_id : principal;
};

type UnfollowUserResponse = variant {
  Ok;
  Err : UnfollowUserError;
};

type UnfollowUserError = variant {
  Unauthorized;
  NotFollowing;
};
```

### BlockUser

Request, response, and error types for the `block_user` method. Blocks another
user, preventing them from following or interacting with the caller.

| Field         | Description                               |
| ------------- | ----------------------------------------- |
| `canister_id` | Principal of the User Canister to block.  |

- **Unauthorized**: the caller is not the canister owner.

```candid
type BlockUserArgs = record {
  canister_id : principal;
};

type BlockUserResponse = variant {
  Ok;
  Err : BlockUserError;
};

type BlockUserError = variant {
  Unauthorized;
};
```

### GetFollowers

Request, response, and error types for the `get_followers` method. Returns a
paginated list of actor URIs that follow this user. The `limit` must not exceed
**50** (the maximum page size).

| Field    | Description                                              |
| -------- | -------------------------------------------------------- |
| `offset` | Number of results to skip (for pagination).              |
| `limit`  | Maximum number of results to return (max 50).            |

- **LimitExceeded**: the requested `limit` exceeds the maximum page size (50).
- **Internal**: an internal error occurred while querying followers.

```candid
type GetFollowersArgs = record {
  offset : nat64;
  limit : nat64;
};

type GetFollowersResponse = variant {
  Ok : vec text;
  Err : GetFollowersError;
};

type GetFollowersError = variant {
  LimitExceeded;
  Internal : text;
};
```

### GetFollowing

Request, response, and error types for the `get_following` method. Returns a
paginated list of actor URIs that this user follows. The `limit` must not exceed
**50** (the maximum page size).

| Field    | Description                                              |
| -------- | -------------------------------------------------------- |
| `offset` | Number of results to skip (for pagination).              |
| `limit`  | Maximum number of results to return (max 50).            |

- **LimitExceeded**: the requested `limit` exceeds the maximum page size (50).
- **Internal**: an internal error occurred while querying the following list.

```candid
type GetFollowingArgs = record {
  offset : nat64;
  limit : nat64;
};

type GetFollowingResponse = variant {
  Ok : vec text;
  Err : GetFollowingError;
};

type GetFollowingError = variant {
  LimitExceeded;
  Internal : text;
};
```

### PublishStatus

Request, response, and error types for the `publish_status` method. Creates a
new status post in the caller's outbox and distributes it via the Federation
Canister. For `Public`, `Unlisted`, and `FollowersOnly` visibilities, the
recipients are the author's followers. For `Direct` visibility, the recipients
are the explicitly listed `mentions` — followers are not addressed.

| Field        | Description                                                                             |
| ------------ | --------------------------------------------------------------------------------------- |
| `content`    | The text content of the new post.                                                       |
| `visibility` | Audience control for this status (see [Visibility](#visibility)).                       |
| `mentions`   | Actor URIs explicitly mentioned. Required (non-empty) when `visibility` is `Direct`.    |

On success, returns the created `Status` with its assigned ID and timestamp.

- **Unauthorized**: the caller is not the canister owner.
- **ContentEmpty**: the content is empty or contains only whitespace.
- **ContentTooLong**: the content exceeds the maximum allowed length.
- **NoRecipients**: a `Direct` status was published with an empty `mentions`
  list.
- **Internal**: an internal error occurred while publishing the status.

```candid
type PublishStatusArgs = record {
  content : text;
  visibility : Visibility;
  mentions : vec text;
};

type PublishStatusResponse = variant {
  Ok : Status;
  Err : PublishStatusError;
};

type PublishStatusError = variant {
  Unauthorized;
  ContentEmpty;
  ContentTooLong;
  NoRecipients;
  Internal : text;
};
```

### DeleteStatus

Request, response, and error types for the `delete_status` method. Removes a
status post from the caller's outbox.

| Field       | Description                             |
| ----------- | --------------------------------------- |
| `status_id` | The unique ID of the status to delete.  |

- **Unauthorized**: the caller is not the canister owner.
- **NotFound**: no status exists with the given ID.

```candid
type DeleteStatusArgs = record {
  status_id : text;
};

type DeleteStatusResponse = variant {
  Ok;
  Err : DeleteStatusError;
};

type DeleteStatusError = variant {
  Unauthorized;
  NotFound;
};
```

### LikeStatus

Request, response, and error types for the `like_status` method. Records a
like on a status authored by another user.

`like_status` is **idempotent**: calling it for a status the caller has
already liked returns `Ok` without recording a duplicate row in the
liked collection and without re-emitting a `Like` activity. Only the
caller (canister owner) is authorized; non-owner calls are rejected at
the inspect layer.

| Field        | Description                              |
| ------------ | ---------------------------------------- |
| `status_url` | ActivityPub URI of the status to like.   |

- **Internal**: an unexpected internal error occurred (database access
  failure, federation dispatch failure, etc.).

```candid
type LikeStatusArgs = record {
  status_url : text;
};

type LikeStatusResponse = variant {
  Ok;
  Err : LikeStatusError;
};

type LikeStatusError = variant {
  Internal : text;
};
```

### UnlikeStatus

Request, response, and error types for the `undo_like` method. Removes a
previously recorded like from a status.

| Field        | Description                              |
| ------------ | ---------------------------------------- |
| `status_url` | ActivityPub URI of the status to unlike. |

- **Unauthorized**: the caller is not the canister owner.
- **NotFound**: no like exists for the given status.

```candid
type UnlikeStatusArgs = record {
  status_url : text;
};

type UnlikeStatusResponse = variant {
  Ok;
  Err : UnlikeStatusError;
};

type UnlikeStatusError = variant {
  Unauthorized;
  NotFound;
};
```

### BoostStatus

Request, response, and error types for the `boost_status` method. Boosts
(reblogs) a status authored by another user, sharing it with the caller's
followers.

`boost_status` is **idempotent**: calling it for a status the caller has
already boosted returns `Ok` without recording a duplicate row in the
boosts collection and without re-emitting an `Announce` activity. Only the
caller (canister owner) is authorized; non-owner calls are rejected at the
inspect layer.

| Field        | Description                             |
| ------------ | --------------------------------------- |
| `status_url` | ActivityPub URI of the status to boost. |

- **Internal**: an unexpected internal error occurred (database access
  failure, federation dispatch failure, etc.).

```candid
type BoostStatusArgs = record {
  status_url : text;
};

type BoostStatusResponse = variant {
  Ok;
  Err : BoostStatusError;
};

type BoostStatusError = variant {
  Internal : text;
};
```

### UndoBoost

Request, response, and error types for the `undo_boost` method. Removes a
previously recorded boost from a status.

`undo_boost` is **idempotent**: calling it for a status the caller has
not boosted (or has already un-boosted) returns `Ok` without emitting a
second `Undo(Announce)` activity. Only the caller (canister owner) is
authorized; non-owner calls are rejected at the inspect layer.

| Field        | Description                                |
| ------------ | ------------------------------------------ |
| `status_url` | ActivityPub URI of the status to un-boost. |

- **Internal**: an unexpected internal error occurred (database access
  failure, federation dispatch failure, etc.).

```candid
type UndoBoostArgs = record {
  status_url : text;
};

type UndoBoostResponse = variant {
  Ok;
  Err : UndoBoostError;
};

type UndoBoostError = variant {
  Internal : text;
};
```

### GetLiked

Request, response, and error types for the `get_liked` method. Returns a
paginated list of status IDs that the caller has liked.

| Field    | Description                                 |
| -------- | ------------------------------------------- |
| `offset` | Number of results to skip (for pagination). |
| `limit`  | Maximum number of results to return.        |

- **Unauthorized**: the caller is not the canister owner.

```candid
type GetLikedArgs = record {
  offset : nat64;
  limit : nat64;
};

type GetLikedResponse = variant {
  Ok : vec text;
  Err : GetLikedError;
};

type GetLikedError = variant {
  Unauthorized;
};
```

### ReadFeed

Request, response, and error types for the `read_feed` method. Returns a
paginated list of feed items from the caller's home timeline, including
statuses from followed users and boosted content.

| Field    | Description                                 |
| -------- | ------------------------------------------- |
| `offset` | Number of results to skip (for pagination). |
| `limit`  | Maximum number of results to return.        |

- **Unauthorized**: the caller is not the canister owner.

```candid
type ReadFeedArgs = record {
  offset : nat64;
  limit : nat64;
};

type ReadFeedResponse = variant {
  Ok : vec FeedItem;
  Err : ReadFeedError;
};

type ReadFeedError = variant {
  Unauthorized;
};
```

### ReceiveActivity

Request, response, and error types for the `receive_activity` method. Called by
the Federation Canister to deliver an incoming ActivityPub activity (encoded as
JSON) to this User Canister's inbox.

| Field           | Description                                |
| --------------- | ------------------------------------------ |
| `activity_json` | JSON-encoded ActivityPub activity object.  |

- **Unauthorized**: the caller is not the Federation Canister.
- **InvalidActivity**: the JSON could not be parsed as a valid ActivityPub
  activity.
- **ProcessingFailed**: the activity was valid but could not be processed
  (e.g. references a non-existent status).

```candid
type ReceiveActivityArgs = record {
  activity_json : text;
};

type ReceiveActivityResponse = variant {
  Ok;
  Err : ReceiveActivityError;
};

type ReceiveActivityError = variant {
  Unauthorized;
  InvalidActivity;
  ProcessingFailed;
};
```

## Federation Canister Types

### FederationInstallArgs

Install arguments for the Federation Canister. Uses the `Init`/`Upgrade`
variant pattern required by IC canister lifecycle.

- **Init**: provided on first install. Sets the Directory Canister principal
  (used to resolve handles to User Canisters) and the public URL used for
  constructing ActivityPub actor URIs and WebFinger responses.
- **Upgrade**: provided on subsequent upgrades (currently empty).

```candid
type FederationInstallArgs = variant {
  Init : record {
    directory_canister : principal;
    public_url : text;
  };
  Upgrade : record {};
};
```

### SendActivity

Request, response, and error types for the `send_activity` method. Called by a
registered User Canister to deliver an outbound ActivityPub activity. Supports
a single activity (`One`) or a batch (`Batch`) per call. Local targets are
routed to the recipient User Canister via the Directory Canister; remote
targets are skipped (remote HTTP delivery is Milestone 2).

| Field           | Description                                                 |
| --------------- | ----------------------------------------------------------- |
| `activity_json` | JSON-encoded ActivityPub activity object to send.           |
| `target_inbox`  | URL of the actor's inbox to deliver the activity to.        |

`SendActivityError`:

- **InvalidTargetInbox(text)**: `target_inbox` URL failed to parse or has an
  unexpected path shape.
- **UnknownLocalUser(text)**: local inbox references a handle that is not
  registered in the Directory Canister.
- **DeliveryFailed(text)**: inter-canister call to the target User Canister
  failed (transport or decode).
- **Rejected(text)**: target User Canister accepted the call but rejected the
  activity.

```candid
type SendActivityArgsObject = record {
  activity_json : text;
  target_inbox : text;
};

type SendActivityArgs = variant {
  One : SendActivityArgsObject;
  Batch : vec SendActivityArgsObject;
};

type SendActivityResult = variant {
  Ok;
  Err : SendActivityError;
};

type SendActivityResponse = variant {
  One : SendActivityResult;
  Batch : vec SendActivityResult;
};

type SendActivityError = variant {
  InvalidTargetInbox : text;
  UnknownLocalUser : text;
  DeliveryFailed : text;
  Rejected : text;
};
```
