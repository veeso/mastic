---
title: "Milestone 0 - Proof of Concept"
layout: page
---

# Milestone 0 - Proof of Concept

**Duration:** 1.5 months

**Goal:** First implementation to demo the social platform with basic
functionalities: signing up, posting statuses, following users, and reading
the feed.

**User Stories:** UC1, UC2, UC5, UC7, UC9, UC12

## Work Items

### WI-0.1: Define shared Candid types in the `did` crate

**Description:** Define all shared Candid types required by Milestone 0 in the
`did` crate. These types are used across the Directory, Federation, and User
canisters.

**What should be done:**

- Define `DirectoryInstallArgs` (initial moderator principal, federation
  canister principal)
- Define `UserInstallArgs` (owner principal, federation canister principal)
- Define `FederationInstallArgs` (directory canister principal, domain name)
- Define sign-up types: `SignUpResponse`
- Define whoami types: `WhoAmIResponse`
- Define user canister query types: `UserCanisterResponse`
- Define get-user types: `GetUserArgs`, `GetUserResponse`
- Define profile types: `GetProfileResponse`, `UserProfile` (handle,
  display name, bio, avatar URL, created at)
- Define follow types: `FollowUserArgs`, `FollowUserResponse`,
  `AcceptFollowArgs`, `AcceptFollowResponse`, `RejectFollowArgs`,
  `RejectFollowResponse`
- Define status types: `PublishStatusArgs`, `PublishStatusResponse`, `Status`
  (id, content, author, created at, visibility)
- Define feed types: `ReadFeedArgs`, `ReadFeedResponse`, `FeedItem`
- Define get-followers/following types: `GetFollowersArgs`,
  `GetFollowersResponse`, `GetFollowingArgs`, `GetFollowingResponse`
- Define `SendActivityArgs`, `SendActivityResponse` for the Federation
  canister
- Define `ReceiveActivityArgs`, `ReceiveActivityResponse` for the User
  canister
- Define ActivityPub base types in `did::federation::activitypub`: `Activity`,
  `ActivityType` (Create, Follow, Accept, Reject), `Object`, `Actor`
- Ensure all types derive `CandidType`, `Deserialize`, `Clone`, `Debug`

**Acceptance Criteria:**

- All types compile and are exported from the `did` crate
- Types match the `.did` interface files in `docs/interface/`
- `cargo clippy` passes with zero warnings
- Unit tests verify serialization/deserialization round-trips

### WI-0.2: Design and implement database schema for Milestone 0

**Description:** Design the relational database schema using `wasm-dbms` for
all entities required by Milestone 0 across the Directory and User canisters.

**What should be done:**

- Add `wasm-dbms` as a workspace dependency
- **Directory Canister schema:**
  - `users` table: `principal` (TEXT PK), `handle` (TEXT UNIQUE NOT NULL),
    `user_canister_id` (TEXT NOT NULL), `status` (TEXT NOT NULL DEFAULT
    'active'), `created_at` (INTEGER NOT NULL)
  - `moderators` table: `principal` (TEXT PK), `added_at` (INTEGER NOT NULL)
  - Index on `users.handle` for fast lookups
- **User Canister schema:**
  - `profile` table (single-row): `handle` (TEXT NOT NULL),
    `display_name` (TEXT), `bio` (TEXT), `avatar_url` (TEXT),
    `header_url` (TEXT), `created_at` (INTEGER NOT NULL),
    `updated_at` (INTEGER NOT NULL)
  - `statuses` table: `id` (TEXT PK), `content` (TEXT NOT NULL),
    `visibility` (TEXT NOT NULL DEFAULT 'public'),
    `created_at` (INTEGER NOT NULL)
  - `inbox` table: `id` (TEXT PK), `activity_type` (TEXT NOT NULL),
    `actor_uri` (TEXT NOT NULL), `object_json` (TEXT NOT NULL),
    `created_at` (INTEGER NOT NULL)
  - `followers` table: `actor_uri` (TEXT PK),
    `created_at` (INTEGER NOT NULL)
  - `following` table: `actor_uri` (TEXT PK), `status` (TEXT NOT NULL
    DEFAULT 'pending'), `created_at` (INTEGER NOT NULL)
  - `keypair` table (single-row): `public_key_pem` (TEXT NOT NULL),
    `private_key_pem` (TEXT NOT NULL)
  - Indexes on `statuses.created_at`, `inbox.created_at` for feed ordering
- Initialize the schema in each canister's `init` function
- Ensure schema survives canister upgrades (`pre_upgrade`/`post_upgrade`)

**Acceptance Criteria:**

- All tables are created on canister initialization
- Schema supports all queries needed by Milestone 0 work items
- Data persists across canister upgrades
- Unit tests verify table creation and basic CRUD operations

### WI-0.3: Implement Directory Canister - sign-up flow

**Description:** Implement the `sign_up` method on the Directory Canister,
which creates a new User Canister for the caller and maps their principal to a
handle and canister ID.

**What should be done:**

- Define canister state: a `BTreeMap` mapping principal to `UserRecord`
  (handle, user canister ID, status)
- Define a secondary index: handle to principal (for lookups)
- Store state in stable memory using `ic-stable-structures`
- Implement `init` to accept `DirectoryInstallArgs` and store the initial
  moderator and federation canister principal
- Implement `sign_up(handle)`:
  - Validate handle format (alphanumeric, lowercase, 1-30 chars)
  - Check handle uniqueness
  - Create a new User Canister via the IC management canister
    (`ic_cdk::api::management_canister::main::create_canister`)
  - Install the User Canister WASM via
    `ic_cdk::api::management_canister::main::install_code`
  - Store the mapping (principal -> handle, canister ID)
  - Register the new User Canister with the Federation Canister
  - Return `SignUpResponse` with the canister ID
- Implement `pre_upgrade` / `post_upgrade` for stable memory persistence

**Acceptance Criteria:**

- Calling `sign_up` with a valid handle creates a User Canister and returns
  its principal
- Duplicate handles are rejected
- Duplicate sign-ups from the same principal are rejected
- Invalid handles are rejected with a descriptive error
- The user record is persisted across canister upgrades
- Integration test: sign up, then verify the canister exists and is callable

### WI-0.4: Implement Directory Canister - query methods

**Description:** Implement the read-only query methods on the Directory
Canister that allow users to discover their canister and look up other users.

**What should be done:**

- Implement `whoami()` query: return the caller's `UserRecord` (handle +
  canister ID) or an error if not registered
- Implement `user_canister(opt principal)` query: return the User Canister ID
  for the given principal (or the caller if `None`)
- Implement `get_user(GetUserArgs)` query: look up a user by handle, return
  their public info (handle, canister ID)

**Acceptance Criteria:**

- `whoami` returns the correct record for a registered user
- `whoami` returns an error for an unregistered caller
- `user_canister(None)` returns the caller's canister
- `user_canister(Some(p))` returns the canister for principal `p`
- `get_user` returns the correct user for a valid handle
- `get_user` returns an error for a non-existent handle

### WI-0.5: Implement User Canister - profile and state management

**Description:** Implement the User Canister's internal state, initialization,
and profile query method.

**What should be done:**

- Define canister state: owner principal, federation canister principal,
  profile data, inbox, outbox, followers list, following list
- Store state in stable memory using `ic-stable-structures`
- Implement `init` to accept `UserInstallArgs` and store owner + federation
  principals
- Generate an RSA keypair for HTTP Signatures (store in stable memory)
- Implement `get_profile()` query: return the user's profile (handle,
  display name, bio, avatar, created at)
- Implement authorization guard: reject calls from non-owner principals for
  owner-only methods
- Implement `pre_upgrade` / `post_upgrade`

**Acceptance Criteria:**

- The User Canister initializes correctly with the provided args
- `get_profile` returns the profile for any caller (public data)
- Owner-only methods reject unauthorized callers
- State survives canister upgrades

### WI-0.6: Implement User Canister - publish status

**Description:** Implement the `publish_status` method, which stores a status
in the user's outbox and sends Create activities to followers via the
Federation Canister.

**What should be done:**

- Define status storage: a collection of `Status` records in stable memory,
  keyed by a unique status ID (e.g., ULID or timestamp-based)
- Implement `publish_status(PublishStatusArgs)`:
  - Authorize the caller (owner only)
  - Create a `Status` record with unique ID, content, timestamp, visibility
  - Store the status in the outbox
  - For each follower, build a `Create(Note)` activity
  - Send activities to the Federation Canister via `send_activity`
- Return `PublishStatusResponse` with the new status ID

**Acceptance Criteria:**

- Only the owner can publish a status
- The status is stored in the outbox with a unique ID
- A `Create(Note)` activity is sent for each follower
- The status ID is returned to the caller
- Statuses persist across upgrades

### WI-0.7: Implement User Canister - follow user

**Description:** Implement the `follow_user`, `accept_follow`, and
`reject_follow` methods for managing follow relationships.

**What should be done:**

- Implement `follow_user(FollowUserArgs)`:
  - Authorize the caller (owner only)
  - Build a `Follow` activity targeting the given handle/actor URI
  - Send the activity to the Federation Canister via `send_activity`
  - Store a pending follow request locally
- Implement `accept_follow(AcceptFollowArgs)`:
  - Called by the Federation Canister when the target accepts
  - Add the requester to the followers list
  - Send an `Accept(Follow)` activity back via the Federation Canister
- Implement `reject_follow(RejectFollowArgs)`:
  - Called by the Federation Canister when the target rejects
  - Remove the pending follow request
  - Send a `Reject(Follow)` activity back via the Federation Canister
- Implement `receive_activity(ReceiveActivityArgs)`:
  - Authorize the caller (federation canister only)
  - Handle incoming `Follow` activities: auto-accept (for M0) and add to
    followers
  - Handle incoming `Accept(Follow)`: add to following list
  - Handle incoming `Create(Note)`: store in inbox

**Acceptance Criteria:**

- `follow_user` sends a Follow activity and records a pending request
- When an Accept is received, the target is added to the following list
- When a Follow is received, the requester is added to the followers list
- `get_followers` returns the correct follower list
- `get_following` returns the correct following list
- Only the Federation Canister can call `receive_activity`

### WI-0.8: Implement User Canister - read feed

**Description:** Implement the `read_feed` method, which aggregates the user's
inbox and outbox into a chronological, paginated feed.

**What should be done:**

- Implement `read_feed(ReadFeedArgs)`:
  - Authorize the caller (owner only)
  - Merge inbox items (statuses from followed users) and outbox items (own
    statuses)
  - Sort by timestamp descending
  - Apply pagination (cursor-based or offset-based as defined in
    `ReadFeedArgs`)
  - Return `ReadFeedResponse` with the page of `FeedItem` records

**Acceptance Criteria:**

- Feed contains both inbox and outbox items
- Items are sorted by timestamp (newest first)
- Pagination works correctly (returns the requested page size, provides a
  cursor/offset for the next page)
- An empty feed returns an empty list (no error)
- Only the owner can read their own feed

### WI-0.9: Implement Federation Canister - activity routing

**Description:** Implement the Federation Canister's `send_activity` method,
which routes activities between local User Canisters via the Directory
Canister. Remote HTTP delivery is out of scope for Milestone 0.

**What should be done:**

- Define canister state: directory canister principal, domain name, set of
  authorized User Canister principals
- Implement `init` to accept `FederationInstallArgs`
- Implement a method to register User Canister principals (called by the
  Directory Canister during sign-up)
- Implement `send_activity(SendActivityArgs)`:
  - Authorize the caller (must be a registered User Canister)
  - Parse the activity to determine the target actor(s)
  - For local targets: resolve the target User Canister via the Directory
    Canister, then call `receive_activity` on it
  - For remote targets: log/skip (federation is Milestone 2)
- Return `SendActivityResponse`

**Acceptance Criteria:**

- Only registered User Canisters can call `send_activity`
- Local activities are correctly routed to the target User Canister
- The Federation Canister resolves local handles via the Directory Canister
- Remote targets are gracefully skipped (no crash)
- Integration test: Alice follows Bob (both local), Bob sees Alice in
  followers

### WI-0.10: Integration tests for Milestone 0 flows

**Description:** Write end-to-end integration tests using pocket-ic that
exercise the complete Milestone 0 user flows.

**What should be done:**

- **Test UC1 (Create Profile):** Deploy Directory + Federation canisters,
  call `sign_up`, verify the User Canister is created and callable
- **Test UC2 (Sign In):** After sign-up, call `whoami` and verify the
  correct canister ID is returned
- **Test UC7 (View Profile):** After sign-up, call `get_user` on the
  Directory, then `get_profile` on the User Canister
- **Test UC5 (Follow User):** Two users sign up, Alice follows Bob, verify
  follower/following lists
- **Test UC9 (Create Status):** Publish a status, verify it appears in the
  author's outbox and is delivered to followers' inboxes
- **Test UC12 (Read Feed):** Publish multiple statuses from different users,
  verify the feed is correctly aggregated and paginated

**Acceptance Criteria:**

- All six user story flows pass as integration tests
- Tests run in CI via `just integration_test`
- Tests use pocket-ic with realistic canister deployment
- Each test is independent and can run in isolation
