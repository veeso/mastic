
# Milestone 1 - Standalone Mastic Node

**Duration:** 3 months

**Goal:** Build all remaining local-node features: profile management (update,
delete), status interactions (like, boost, delete), user search, and
moderation. After this milestone, Mastic is a fully usable social network
within a single node, ready for Fediverse integration.

**User Stories:** UC3, UC4, UC6, UC8, UC10, UC11, UC15, UC16, UC17, UC18, UC19,
UC20, UC21

**Prerequisites:** Milestone 0 completed.

## Work Items

### WI-1.1: Extend database schema for Milestone 1

**Description:** Extend the `wasm-dbms` schema to support all new entities
introduced in Milestone 1: likes, boosts, blocks, tombstones, and status
deletion tracking.

**What should be done:**

- **User Canister schema additions:**
  - `liked` table: `status_uri` (TEXT PK), `created_at` (INTEGER NOT NULL)
  - `blocks` table: `actor_uri` (TEXT PK), `created_at` (INTEGER NOT NULL)
  - Add `like_count` (INTEGER DEFAULT 0) and `boost_count` (INTEGER DEFAULT
    0) columns to the `statuses` table
  - Add `is_boost` (INTEGER DEFAULT 0) and `original_status_uri` (TEXT)
    columns to the `inbox` table for boost tracking
  - `media` table: `id` (TEXT PK), `status_id` (TEXT NOT NULL, FK →
    statuses.id), `media_type` (TEXT NOT NULL), `description` (TEXT),
    `blurhash` (TEXT), `data` (BLOB NOT NULL), `created_at` (INTEGER NOT
    NULL)
  - Add index on `media.status_id`
- **Directory Canister schema additions:**
  - `tombstones` table: `handle` (TEXT PK), `deleted_at` (INTEGER NOT NULL),
    `expires_at` (INTEGER NOT NULL)
  - Add index on `tombstones.expires_at` for cleanup
- Run schema migrations on canister upgrade (add new tables/columns without
  losing existing data)

**Acceptance Criteria:**

- New tables and columns are created on upgrade from Milestone 0 schema
- Existing data is preserved during migration
- Unit tests verify migration from M0 schema to M1 schema
- All new queries needed by M1 work items are supported

### WI-1.2: Implement User Canister - update profile (UC3)

**Description:** Allow the user to update their profile fields and propagate
the change to followers via an Update activity.

**What should be done:**

- Implement `update_profile(UpdateProfileArgs)`:
  - Authorize the caller (owner only)
  - Accept optional fields: display name, bio, avatar URL, header URL
  - Update the profile in stable memory (only fields that are `Some`)
  - Build an `Update(Person)` activity
  - Send the activity to the Federation Canister via `send_activity` so
    followers (local for now) receive the updated profile
- Define `UpdateProfileArgs` and `UpdateProfileResponse` in the `did` crate
  if not already present

**Acceptance Criteria:**

- Only the owner can update their profile
- Partial updates work (e.g., updating only the bio leaves other fields
  unchanged)
- The updated profile is returned by `get_profile`
- An `Update` activity is sent to the Federation Canister
- Integration test: update profile, verify `get_profile` returns new values

### WI-1.3: Implement delete profile flow (UC4)

**Description:** Implement account deletion across Directory, User, and
Federation canisters.

**What should be done:**

- **Directory Canister:** Implement `delete_profile()`:
  - Authorize the caller (must be a registered user)
  - Create a tombstone record for the user (prevents handle reuse for a
    grace period)
  - Notify the User Canister to aggregate Delete activities
  - After activities are sent, delete the User Canister via the IC
    management canister (`stop_canister` + `delete_canister`)
  - Remove the user record from the directory
- **User Canister:** Implement `delete_profile()`:
  - Authorize the caller (owner only)
  - Aggregate a `Delete(Person)` activity for all followers
  - Send activities to the Federation Canister
  - Return success
- **Federation Canister:** Handle `Delete(Person)` activities:
  - Buffer the activity data before forwarding (the User Canister will be
    destroyed)
  - Route to local followers via the Directory Canister
  - For remote: skip (Milestone 2)
- Define `DeleteProfileResponse` in the `did` crate

**Acceptance Criteria:**

- Calling `delete_profile` on the Directory removes the user record
- The User Canister is stopped and deleted via the IC management canister
- A `Delete` activity is delivered to local followers
- The deleted user's handle cannot be reused immediately (tombstone)
- `whoami` returns an error after deletion
- `get_user` returns an error for the deleted handle
- Integration test: create user, delete, verify canister is gone

### WI-1.4: Implement User Canister - unfollow user (UC6)

**Description:** Allow a user to unfollow another user and notify the target
via an Undo(Follow) activity.

**What should be done:**

- Implement `unfollow_user(UnfollowUserArgs)`:
  - Authorize the caller (owner only)
  - Remove the target from the following list
  - Build an `Undo(Follow)` activity
  - Send the activity to the Federation Canister
- **User Canister** `receive_activity` handler: handle incoming
  `Undo(Follow)`:
  - Remove the requester from the followers list
- Define `UnfollowUserArgs`, `UnfollowUserResponse` in the `did` crate
- Add `Undo` to `ActivityType` in the `did` crate

**Acceptance Criteria:**

- After unfollowing, the target is removed from the following list
- The target's follower list no longer contains the caller
- An `Undo(Follow)` activity is delivered to the target
- Unfollowing a user you don't follow returns a descriptive error
- Integration test: follow, then unfollow, verify lists are updated

### WI-1.5: Implement Directory Canister - search profiles (UC8)

**Description:** Implement the `search_profiles` method for user discovery.

**What should be done:**

- Implement `search_profiles(SearchProfilesArgs)` query:
  - Accept a search query string and pagination parameters
  - Search by handle prefix or substring match
  - Return a paginated list of matching users (handle + canister ID)
- Define `SearchProfilesArgs`, `SearchProfilesResponse` in the `did` crate

**Acceptance Criteria:**

- Searching by exact handle returns the correct user
- Searching by prefix returns all matching users
- Empty query returns a paginated list of all users
- Pagination works correctly
- Results do not include suspended or deleted users
- Integration test: create multiple users, search, verify results

### WI-1.6: Implement User Canister - like status (UC10)

**Description:** Allow a user to like a status and notify the author.

**What should be done:**

- Implement `like_status(LikeStatusArgs)`:
  - Authorize the caller (owner only)
  - Record the like in the user's liked collection (stable memory)
  - Build a `Like` activity targeting the status
  - Send the activity to the Federation Canister
  - **Idempotent**: if the caller has already liked the status, return
    `Ok` without inserting a duplicate row and without re-sending the
    `Like` activity
- Implement `get_liked(GetLikedArgs)` query:
  - Return the paginated list of statuses liked by the user
- Implement `undo_like(UnlikeStatusArgs)`:
  - Remove the like from the liked collection
  - Send an `Undo(Like)` activity to the Federation Canister
- **User Canister** `receive_activity` handler: handle incoming `Like`:
  - Increment the like count on the target status
- Handle incoming `Undo(Like)`:
  - Decrement the like count on the target status
- Define `LikeStatusArgs`, `LikeStatusResponse`, `UnlikeStatusArgs`,
  `UnlikeStatusResponse`, `GetLikedArgs`, `GetLikedResponse` in the `did` crate
- Add `Like` to `ActivityType`

**Acceptance Criteria:**

- Liking a status records it in the liked collection
- A `Like` activity is sent to the status author
- The author's status like count is incremented
- `get_liked` returns the correct list
- Undoing a like removes it and sends an `Undo(Like)` activity
- Liking a status the caller already liked is idempotent: returns `Ok`,
  does not insert a duplicate, and does not re-send the `Like` activity
- Integration test: Alice likes Bob's status, verify like count and liked list
- Integration test: Alice calls `like_status` twice on the same status,
  verify second call returns `Ok`, liked list contains a single entry,
  and Bob's status like count is incremented exactly once

### WI-1.7: Implement User Canister - boost status (UC11)

**Description:** Allow a user to boost (reblog) a status and notify both the
author and the user's followers.

**What should be done:**

- Implement `boost_status(BoostStatusArgs)`:
  - Authorize the caller (owner only)
  - Record the boost in the user's outbox
  - Build an `Announce` activity
  - Send the activity to the Federation Canister (targets: status author +
    all of the booster's followers)
- Implement `undo_boost(UndoBoostArgs)`:
  - Remove the boost from the outbox
  - Send an `Undo(Announce)` activity
- **User Canister** `receive_activity` handler: handle incoming `Announce`:
  - Store the boosted status in the inbox (as a boost, not a new status)
- Handle incoming `Undo(Announce)`:
  - Remove the boosted status from the inbox
- Define `BoostStatusArgs`, `BoostStatusResponse`, `UndoBoostArgs`,
  `UndoBoostResponse` in the `did` crate
- Add `Announce` to `ActivityType`

**Acceptance Criteria:**

- Boosting a status records it in the outbox
- An `Announce` activity is sent to the author and the booster's followers
- Followers see the boost in their feed
- Undoing a boost removes it and sends an `Undo(Announce)` activity
- Cannot boost the same status twice
- Integration test: Alice boosts Bob's status, Charlie (Alice's follower) sees
  it in their feed

### WI-1.8: Implement User Canister - delete status (UC15)

**Description:** Allow both the status owner and moderators to delete a status.

**What should be done:**

- Implement `delete_status(DeleteStatusArgs)`:
  - Authorize the caller: must be the owner **or** a moderator (the
    moderator list is resolved from the Directory Canister)
  - Remove the status from the outbox
  - Build a `Delete(Note)` activity
  - Send the activity to the Federation Canister to notify followers
- Define `DeleteStatusArgs`, `DeleteStatusResponse` in the `did` crate
- Add `Delete` to `ActivityType` if not already present

**Acceptance Criteria:**

- The owner can delete their own status
- A moderator can delete any user's status
- Non-owner, non-moderator callers are rejected
- A `Delete(Note)` activity is sent to followers
- The status no longer appears in feeds after deletion
- Integration test: publish status, delete it, verify it's gone from feeds

### WI-1.9: Implement Directory Canister - moderation (UC16)

**Description:** Implement moderator management and user suspension on the
Directory Canister.

**What should be done:**

- Implement `add_moderator(AddModeratorArgs)`:
  - Authorize the caller (must be an existing moderator)
  - Add the target principal to the moderator list
- Implement `remove_moderator(RemoveModeratorArgs)`:
  - Authorize the caller (must be an existing moderator)
  - Prevent removing the last moderator
  - Remove the target principal from the moderator list
- Implement `suspend(SuspendArgs)`:
  - Authorize the caller (must be a moderator)
  - Mark the user as suspended in the directory
  - Notify the User Canister to send a `Delete` activity to followers
  - Suspended users cannot call any methods on their User Canister
- Define `AddModeratorArgs`, `AddModeratorResponse`, `RemoveModeratorArgs`,
  `RemoveModeratorResponse`, `SuspendArgs`, `SuspendResponse` in the `did`
  crate

**Acceptance Criteria:**

- Only moderators can add/remove moderators
- The last moderator cannot be removed
- Suspending a user marks them as inactive in the directory
- Suspended users cannot interact with their User Canister
- A `Delete` activity is sent to the suspended user's followers
- `search_profiles` excludes suspended users
- Integration test: add moderator, suspend user, verify user is locked out

### WI-1.10: Implement User Canister - block user

**Description:** Allow a user to block another user, preventing interactions.

**What should be done:**

- Implement `block_user(BlockUserArgs)`:
  - Authorize the caller (owner only)
  - Record the block locally (block list in stable memory)
  - If the blocked user is a follower, remove them from the followers list
  - If the owner follows the blocked user, remove from following list
  - Send a `Block` activity to the Federation Canister
- **User Canister** `receive_activity` handler: handle incoming `Block`:
  - Hide the blocking user's content from the blocked user
- Activities from blocked users should be silently dropped in
  `receive_activity`
- Define `BlockUserArgs`, `BlockUserResponse` in the `did` crate
- Add `Block` to `ActivityType`

**Acceptance Criteria:**

- Blocking a user removes mutual follow relationships
- Activities from a blocked user are dropped
- A `Block` activity is sent via the Federation Canister
- The blocked user does not appear in the blocker's feeds
- Integration test: Alice blocks Bob, verify follow removed and activities
  dropped

### WI-1.12: Implement Directory Canister - upgrade user canisters (UC17)

**Description:** Implement a controller-only method to batch-upgrade all User
Canister WASMs via a timer-based state machine.

**What should be done:**

- Implement `upgrade_user_canisters(UpgradeUserCanistersArgs)`:
  - Authorize the caller (must be a controller via `ic_cdk::api::is_controller`)
  - Reject if an upgrade is already in progress
  - Store the provided WASM blob
  - Build an upgrade queue containing all registered user canister IDs
  - Start a recurring timer that processes canisters in batches (5-10 per
    tick)
- Implement the upgrade state machine (mirrors sign-up flow pattern):
  - Per-canister states: `Pending`, `Upgrading`, `Completed`,
    `Failed(attempts)`, `PermanentlyFailed`
  - On each tick: pick next batch of `Pending` or `Failed(n < 5)` canisters,
    call `install_code` (mode: upgrade) via the management canister
  - On success: mark `Completed`
  - On failure: increment attempt counter; if attempts >= 5, mark
    `PermanentlyFailed`
  - When all canisters are processed, stop the timer and mark the batch as
    completed
- Implement `get_upgrade_status()` query:
  - Return `UpgradeStatus` with: total count, completed count, failed count,
    permanently failed count, and whether an upgrade is in progress
- Define `UpgradeUserCanistersArgs`, `UpgradeUserCanistersResponse`,
  `UpgradeStatus` in the `did` crate

**Acceptance Criteria:**

- Only the controller can call `upgrade_user_canisters`
- Concurrent upgrade requests are rejected
- Canisters are upgraded in batches without hitting instruction limits
- Failed canisters are retried up to 5 times
- After 5 failures a canister is marked as permanently failed and skipped
- `get_upgrade_status` accurately reports progress
- Integration test: deploy user canisters, trigger upgrade with new WASM,
  verify all canisters run the new version

### WI-1.13: Implement Directory Canister - sign-up fee (UC18)

**Description:** Require callers to attach cycles when signing up to cover
the cost of User Canister creation, preventing spam account creation.

**What should be done:**

- Modify the existing `sign_up` flow in the Directory Canister:
  - Before any processing, check `ic_cdk::api::call::msg_cycles_available()`
    against the required fee (canister creation fee + initial cycles, both
    existing constants)
  - If insufficient: reject with `InsufficientCycles { required, provided }`
    error
  - If sufficient: accept cycles via `msg_cycles_accept()` and proceed with
    the existing sign-up flow, forwarding cycles to the management canister
    for canister creation
- Add `InsufficientCycles` variant to the sign-up error type in the `did`
  crate
- Update existing sign-up integration tests to attach the required cycles

**Acceptance Criteria:**

- Sign-up without cycles is rejected with a clear error showing required
  amount
- Sign-up with insufficient cycles is rejected
- Sign-up with exact or excess cycles succeeds
- Accepted cycles are forwarded to the management canister for User Canister
  creation
- Existing sign-up tests are updated and pass
- Integration test: attempt sign-up without cycles, verify rejection; sign up
  with cycles, verify success

### WI-1.14: Implement User Canister - action rate limiting (UC19)

**Description:** Enforce a per-user rate limit on mutating social actions to
prevent action spam.

**What should be done:**

- Implement a rate limiter module in the User Canister:
  - Circular buffer of 20 timestamps stored in heap memory
  - On each rate-limited call: check if the oldest entry is less than 60
    seconds ago
  - If yes: reject with `RateLimitExceeded` error
  - If no: record the current timestamp and proceed
- Apply the rate limiter at the top of these methods:
  - `post_status`, `delete_status`
  - `follow_user`, `unfollow_user`
  - `like_status`, `undo_like`
  - `boost_status`, `undo_boost`
  - `block_user`
- Add `RateLimitExceeded` variant to the relevant error types in the `did`
  crate
- Constants: 20 actions per 60-second window (compile-time)

**Acceptance Criteria:**

- Actions within the limit succeed normally
- The 21st action within 60 seconds is rejected with `RateLimitExceeded`
- After 60 seconds the window slides and actions succeed again
- Rate limit state resets on canister upgrade (heap-only)
- All rate-limited methods enforce the check
- Unit test: simulate rapid actions, verify rejection at threshold
- Integration test: call 20 actions in quick succession, verify 21st fails

### WI-1.15: Implement User Canister - reply to status (UC20)

**Description:** Allow a user to reply to an existing status, creating a
thread. This is essential for meaningful social interaction and for Fediverse
compatibility (M3), where remote instances expect `inReplyTo` on `Note`
objects.

**What should be done:**

- Extend `PublishStatusArgs` in the `did` crate:
  - Add `in_reply_to` field (optional status URI string)
- Extend the `Status` type in the `did` crate:
  - Add `in_reply_to` field (optional status URI string)
- Extend the `statuses` database table:
  - Add `in_reply_to_uri` column (TEXT, nullable)
  - Add index on `in_reply_to_uri` for thread lookups
- Update `publish_status` in the User Canister:
  - If `in_reply_to` is provided, validate the URI format
  - Store the `in_reply_to_uri` in the statuses table
  - Include `inReplyTo` in the `Create(Note)` activity sent via the
    Federation Canister
  - Send the activity to both the original author and the caller's followers
- Implement `get_thread(GetThreadArgs)` query:
  - Accept a status ID
  - Return the status and all replies (statuses where `in_reply_to_uri`
    matches), sorted chronologically
  - Support pagination
- Update `receive_activity` handler for `Create(Note)`:
  - Parse and store the `inReplyTo` field from incoming notes
- Define `GetThreadArgs`, `GetThreadResponse` in the `did` crate

**Acceptance Criteria:**

- A reply status is created with a valid `in_reply_to` reference
- The reply appears in the author's outbox and followers' inboxes
- The original status author receives the reply in their inbox
- `get_thread` returns the original status and all its replies in order
- Replying to a non-existent status URI still succeeds (the URI is stored
  as-is for federation compatibility)
- The `Create(Note)` activity includes the `inReplyTo` field
- Incoming `Create(Note)` activities with `inReplyTo` are stored correctly

### WI-1.16: Implement User Canister - media attachments (UC21)

**Description:** Allow users to attach media files (images, video, audio) to
statuses. Media is stored as blobs in the User Canister's `media` table,
linked to a status via foreign key.

**What should be done:**

- Define types in the `did` crate:
  - `MediaAttachment`: id, media_type, description (alt text), blurhash
  - Add `media` field (Vec of media bytes + metadata) to `PublishStatusArgs`
  - Add `media` field (Vec of `MediaAttachment`) to `Status`
- Update `publish_status` in the User Canister:
  - For each attachment: generate an ID, store the blob and metadata in the
    `media` table with the status ID as foreign key
  - Include attachment metadata in the `Create(Note)` activity (as
    ActivityPub `Attachment` objects with media type, name/description, and
    a URL pointing to the media retrieval endpoint)
- Implement `get_media(GetMediaArgs)` query:
  - Accept a media ID
  - Return the raw media blob and its content type
  - Allow any caller (public query, needed for federation)
- Implement `get_status_media(GetStatusMediaArgs)` query:
  - Accept a status ID
  - Return the list of `MediaAttachment` metadata for that status
- Update `receive_activity` handler for `Create(Note)`:
  - Parse attachment metadata from incoming notes and store references
    (remote media is referenced by URL, not downloaded)
- Define `GetMediaArgs`, `GetMediaResponse`, `GetStatusMediaArgs`,
  `GetStatusMediaResponse` in the `did` crate
- When a status is deleted (`delete_status`), cascade-delete its media rows

**Acceptance Criteria:**

- A status can be published with one or more media attachments
- Media blobs are stored in the `media` table linked to the status
- `get_media` returns the correct blob and content type
- `get_status_media` returns metadata for all attachments on a status
- The `Create(Note)` activity includes attachment objects
- Deleting a status also deletes its associated media
- Incoming notes with attachments store the remote attachment metadata
- A status without attachments works as before (no regression)

### WI-1.11: Integration tests for Milestone 1 flows

**Description:** Write end-to-end integration tests for all Milestone 1 user
stories.

**What should be done:**

- **Test UC3 (Update Profile):** Update profile fields, verify changes
- **Test UC4 (Delete Profile):** Delete account, verify canister removed
- **Test UC6 (Unfollow):** Follow then unfollow, verify lists
- **Test UC8 (Search):** Create users, search by prefix, verify results
- **Test UC10 (Like):** Like a status, verify like count and liked list;
  undo like
- **Test UC11 (Boost):** Boost a status, verify followers see it; undo boost
- **Test UC15 (Delete Status):** Publish and delete status, verify removal
- **Test UC16 (Moderation):** Add moderator, suspend user, verify lockout
- **Test Block:** Block user, verify follow removal and activity filtering
- **Test UC17 (Upgrade):** Deploy user canisters, trigger WASM upgrade,
  verify all run new version
- **Test UC18 (Sign-up Fee):** Attempt sign-up without cycles, verify
  rejection; sign up with cycles, verify success
- **Test UC19 (Rate Limit):** Perform 20 rapid actions, verify 21st is
  rejected
- **Test UC20 (Reply):** Alice publishes a status, Bob replies, verify
  reply has `in_reply_to` set, verify `get_thread` returns both statuses in
  order, verify Alice receives the reply in her inbox
- **Test UC21 (Media):** Alice publishes a status with a media attachment,
  verify `get_status_media` returns the attachment metadata, verify
  `get_media` returns the blob, delete the status and verify the media is
  also deleted

**Acceptance Criteria:**

- All user story flows pass as integration tests
- Tests run in CI via `just integration_test`
- Each test is independent and can run in isolation
- Tests cover both success and error paths
