
# Milestone 0 - Proof of Concept

**Duration:** 1.5 months

**Goal:** First implementation to demo the social platform with basic
functionalities: signing up, posting statuses, following users, and reading
the feed.

**User Stories:** UC1, UC2, UC5, UC7, UC9, UC12

## Work Items

### WI-0.1: Implement ActivityPub types in the `activitypub` crate

**Description:** Implement all ActivityPub and ActivityStreams protocol types in
the `activitypub` crate (`crates/libs/activitypub`). This crate provides the
canonical Rust representation of the ActivityPub protocol used by the Federation
Canister for S2S communication and JSON-LD serialization/deserialization.
Types must round-trip correctly through `serde_json` and match the JSON-LD
payloads documented in `docs/src/activitypub.md`.

**What should be done:**

- **Core types (`activitypub::object`):**
  - `Object` — base type with `id`, `type`, `content`, `name`, `summary`,
    `published`, `updated`, `url`, `to`, `cc`, `bto`, `bcc`, `audience`,
    `attributed_to`, `in_reply_to`, `source`, `tag`, `attachment`, `replies`,
    `likes`, `shares`, `sensitive`
  - `Source` — `content` + `media_type`
  - `Tombstone` — `id`, `type`, `published`, `updated`, `deleted`
  - `ObjectType` enum — `Note`, `Question`, `Image`, `Tombstone`, etc.
- **Actor types (`activitypub::actor`):**
  - `Actor` — extends Object with `inbox`, `outbox`, `following`, `followers`,
    `liked`, `preferred_username`, `public_key`, `endpoints`,
    `manually_approves_followers`, `discoverable`, `indexable`, `suspended`,
    `memorial`, `featured`, `featured_tags`, `also_known_as`,
    `attribution_domains`, `icon`, `image`
  - `ActorType` enum — `Person`, `Application`, `Service`, `Group`,
    `Organization`
  - `PublicKey` — `id`, `owner`, `public_key_pem`
  - `Endpoints` — `shared_inbox`
- **Activity types (`activitypub::activity`):**
  - `Activity` — extends Object with `actor`, `object`, `target`, `result`,
    `origin`, `instrument`
  - `ActivityType` enum — `Create`, `Update`, `Delete`, `Follow`, `Accept`,
    `Reject`, `Like`, `Announce`, `Undo`, `Block`, `Add`, `Remove`, `Flag`,
    `Move`
- **Collection types (`activitypub::collection`):**
  - `Collection` — `id`, `type`, `total_items`, `first`, `last`, `current`,
    `items`
  - `OrderedCollection` — same as Collection with `ordered_items`
  - `CollectionPage` / `OrderedCollectionPage` — `part_of`, `next`, `prev`,
    `items`/`ordered_items`
- **Link types (`activitypub::link`):**
  - `Link` — `href`, `rel`, `media_type`, `name`, `hreflang`, `height`,
    `width`
  - `Mention` — subtype of Link
  - `Hashtag` — subtype of Link
- **Tag types (`activitypub::tag`):**
  - `Tag` enum — `Mention`, `Hashtag`, `Emoji`
  - `Emoji` — `id`, `name`, `icon` (Image with `url` and `media_type`)
- **Mastodon extensions (`activitypub::mastodon`):**
  - `PropertyValue` — `name`, `value` (for profile metadata fields)
  - Poll support on `Question` objects: `end_time`, `closed`,
    `voters_count`, `one_of`/`any_of` with `name` + `replies.total_items`
  - Attachment properties: `blurhash`, `focal_point`
- **WebFinger types (`activitypub::webfinger`):**
  - `WebFingerResponse` — `subject`, `aliases`, `links`
  - `WebFingerLink` — `rel`, `type`, `href`, `template`
- **JSON-LD context (`activitypub::context`):**
  - Constants for standard context URIs
    (`https://www.w3.org/ns/activitystreams`,
    `https://w3id.org/security/v1`,
    Mastodon namespace `http://joinmastodon.org/ns#`)
  - `Context` type for `@context` serialization (single URI, array, or map)
- All types derive `serde::Serialize`, `serde::Deserialize`, `Clone`,
  `Debug`, `PartialEq`
- Use `#[serde(rename_all = "camelCase")]` to match JSON-LD field naming
- Use `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields
- Use `#[serde(rename = "@context")]` for the context field

**Acceptance Criteria:**

- All types compile and are exported from the `activitypub` crate
- `serde_json` round-trip tests for every top-level type
- Deserialization tests using real-world Mastodon JSON-LD payloads from
  `docs/src/activitypub.md` examples
- `cargo clippy` passes with zero warnings
- Unit tests cover: Object, Actor, each ActivityType, Collection,
  OrderedCollection, CollectionPage, WebFinger, Mention, Hashtag, Emoji

### WI-0.2: Define shared Candid types in the `did` crate

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
- Ensure all types derive `CandidType`, `Deserialize`, `Serialize`, `Clone`, `Debug`

**Acceptance Criteria:**

- All types compile and are exported from the `did` crate
- Types match the `.did` interface files in `docs/interface/`
- `cargo clippy` passes with zero warnings
- Unit tests verify serialization/deserialization round-trips

### WI-0.3: Design and implement database schema for Milestone 0

**Description:** Design the relational database schema using `wasm-dbms` for
all entities required by Milestone 0 in the Directory and User canisters.
Since `wasm-dbms` manages its own stable memory, `ic-stable-structures` cannot
be used alongside it in these canisters. Canister init arguments and runtime
configuration are persisted in a `settings` key-value table instead.
The Federation Canister does not use `wasm-dbms` and uses
`ic-stable-structures` directly (see WI-0.10).

**What should be done:**

- Add `ic-dbms-canister` as a workspace dependency
- **Create `crates/libs/db-utils` crate:**
  - Define `SettingKey` as a `u32` newtype with named constants per canister
    (e.g., `FEDERATION_PRINCIPAL`, `OWNER_PRINCIPAL`, `DOMAIN_NAME`)
  - Define `SettingValue` enum wrapping `ic-dbms-canister` `Value` variants
    (Text, Integer, Blob) with typed accessor methods
    (`as_text()`, `as_principal()`, etc.)
  - Provide helper functions for reading/writing settings rows
  - Add the crate to the workspace in root `Cargo.toml`
- **Shared `settings` table (both canisters):**
  - `settings` table: `key` (INTEGER PK), `value` (depends on key — TEXT,
    INTEGER, or BLOB)
  - Uses `SettingKey` constants from `db-utils` to identify entries
- **Directory Canister schema:**
  - `settings` table — stores `federation_principal` from
    `DirectoryInstallArgs`
  - The initial moderator from `DirectoryInstallArgs` is inserted as the
    first row in the `moderators` table during `init`
  - `users` table: `principal` (PRINCIPAL PK), `handle` (TEXT UNIQUE NOT
    NULL), `user_canister_id` (PRINCIPAL NOT NULL), `status` (TEXT NOT NULL
    DEFAULT 'active'), `created_at` (INTEGER NOT NULL)
  - `moderators` table: `principal` (PRINCIPAL PK), `added_at` (INTEGER
    NOT NULL)
  - Index on `users.handle` for fast lookups
- **User Canister schema:**
  - `settings` table — stores `owner_principal`, `federation_principal`
    from `UserInstallArgs`
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
  - Ed25519 public key is derived at runtime via the IC threshold
    Schnorr API (`schnorr_public_key`) and cached in memory
  - Indexes on `statuses.created_at`, `inbox.created_at` for feed ordering
- Initialize the schema in each canister's `init` function and persist init
  args into the `settings` table
- Data survives canister upgrades via `wasm-dbms` stable memory management

**Acceptance Criteria:**

- All tables are created on canister initialization
- Init args are persisted in the `settings` table and retrievable after upgrade
- `db-utils` crate compiles and is usable from both canisters
- Schema supports all queries needed by Milestone 0 work items
- Data persists across canister upgrades
- Unit tests verify table creation and basic CRUD operations

### WI-0.4: Implement Directory Canister - sign-up flow

**Description:** Implement the `sign_up` method on the Directory Canister,
which creates a new User Canister for the caller and maps their principal to a
handle and canister ID.

**What should be done:**

- Use the database schema from WI-0.3 (`users`, `moderators`, `settings`
  tables)
- Implement `init` to accept `DirectoryInstallArgs`, create the schema,
  and persist init args into the `settings` table
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
**Acceptance Criteria:**

- Calling `sign_up` with a valid handle creates a User Canister and returns
  its principal
- Duplicate handles are rejected
- Duplicate sign-ups from the same principal are rejected
- Invalid handles are rejected with a descriptive error
- The user record is persisted across canister upgrades
- Integration test: sign up, then verify the canister exists and is callable

### WI-0.5: Implement Directory Canister - query methods

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

### WI-0.6: Implement User Canister - profile and state management

**Description:** Implement the User Canister's internal state, initialization,
Ed25519 signing via IC threshold Schnorr, and profile query method.

**What should be done:**

- Use the database schema from WI-0.3 (`settings`, `profile`, `statuses`,
  `inbox`, `followers`, `following` tables)
- Implement `init` to accept `UserInstallArgs`, create the schema, and
  persist init args into the `settings` table
- Implement Ed25519 key retrieval and signing via the IC management
  canister's threshold Schnorr API (`schnorr_public_key`,
  `sign_with_schnorr` with `SchnorrAlgorithm::Ed25519`):
  - The public key is fetched once and cached in a thread-local
  - Signing is performed on demand for HTTP Signatures
  - An adapter trait (`SchnorrCanister`) abstracts the management canister
    calls for testability
- Implement `get_profile()` query: return the user's profile (handle,
  display name, bio, avatar, created at)
- Implement authorization guard: reject calls from non-owner principals for
  owner-only methods

**Acceptance Criteria:**

- The User Canister initializes correctly with the provided args
- The Ed25519 public key is retrievable via the Schnorr adapter
- Signing produces a valid Ed25519 signature via the Schnorr adapter
- `get_profile` returns the profile for any caller (public data)
- Owner-only methods reject unauthorized callers
- State survives canister upgrades

### WI-0.7: Implement User Canister - publish status

**Description:** Implement the `publish_status` method on the User Canister,
which stores a status in the user's outbox (`statuses` table) and sends Create
activities to followers via the Federation Canister. Also expose `send_activity`
on the Federation Canister as a no-op stub (actual routing logic comes in
WI-0.10).

**What should be done:**

- **Federation Canister:** expose
  `send_activity(SendActivityArgs) -> SendActivityResponse` as a no-op — accept
  the call, return success, do nothing (actual routing is WI-0.10)
- **User Canister:** implement `publish_status(PublishStatusArgs)`:
  - Authorize the caller (owner only)
  - Generate a Snowflake ID for the status
  - Create and insert a `Status` record (id, content, visibility, created\_at)
    into the `statuses` table
  - Query the `followers` table
  - For each follower, build a `Create(Note)` activity and call
    `send_activity` on the Federation Canister
  - Return `PublishStatusResponse` with the new status ID

**Acceptance Criteria:**

- Only the owner can publish a status
- The status is stored in the `statuses` table with a unique Snowflake ID
- A `Create(Note)` activity is sent for each follower via `send_activity`
- The status ID is returned to the caller
- Statuses persist across upgrades
- `send_activity` is exposed on the Federation Canister (no-op for now)

### WI-0.8: Implement User Canister - follow user

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

### WI-0.9: Implement User Canister - read feed

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

### WI-0.10: Implement Federation Canister - activity routing

**Description:** Implement the Federation Canister's `send_activity` method,
which routes activities between local User Canisters via the Directory
Canister. Remote HTTP delivery is out of scope for Milestone 0.

**What should be done:**

- Define canister state using `ic-stable-structures`: directory canister
  principal, domain name, set of authorized User Canister principals
  (the Federation Canister does not use `wasm-dbms`)
- Implement `init` to accept `FederationInstallArgs` and persist state in
  stable memory
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

### WI-0.11: Integration tests for Milestone 0 flows

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
