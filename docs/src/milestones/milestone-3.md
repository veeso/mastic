
# Milestone 3 - Integrating the Fediverse

**Duration:** 2 months

**Goal:** Implement the Federation Protocol to make Mastic fully compatible
with the Fediverse. Remote Mastodon instances can discover Mastic users via
WebFinger, fetch actor profiles, and exchange activities over HTTP with
ActivityPub and HTTP Signatures.

**User Stories:** UC13, UC14

**Prerequisites:** Milestone 2 completed.

## Work Items

### WI-3.1: Extend database schema for Milestone 3

**Description:** Extend the `wasm-dbms` schema to support federation-specific
data: remote actor cache, delivery queue, and HTTP signature key references.

**What should be done:**

- **Federation Canister schema:**
  - `remote_actors` table: `actor_uri` (TEXT PK), `inbox_url` (TEXT NOT
    NULL), `shared_inbox_url` (TEXT), `public_key_pem` (TEXT NOT NULL),
    `display_name` (TEXT), `summary` (TEXT), `icon_url` (TEXT),
    `fetched_at` (INTEGER NOT NULL), `expires_at` (INTEGER NOT NULL)
  - `delivery_queue` table: `id` (TEXT PK), `activity_json` (TEXT NOT NULL),
    `target_inbox_url` (TEXT NOT NULL), `sender_canister_id` (TEXT NOT
    NULL), `attempts` (INTEGER DEFAULT 0), `last_attempt_at` (INTEGER),
    `status` (TEXT NOT NULL DEFAULT 'pending'),
    `created_at` (INTEGER NOT NULL)
  - `authorized_canisters` table: `canister_id` (TEXT PK),
    `registered_at` (INTEGER NOT NULL)
  - Index on `delivery_queue.status` for pending delivery lookup
  - Index on `remote_actors.expires_at` for cache eviction
- **User Canister schema additions:**
  - Add `actor_uri` (TEXT) column to `followers` and `following` tables to
    distinguish local vs remote actors
- Run schema migrations on canister upgrade

**Acceptance Criteria:**

- New tables and columns are created on upgrade from M2 schema
- Existing data is preserved during migration
- Cache eviction queries work on the `remote_actors` table
- Delivery queue supports retry queries (find pending with attempts < max)

### WI-3.2: Implement WebFinger endpoint

**Description:** Serve WebFinger responses so remote instances can discover
Mastic users by their `acct:` URI.

**What should be done:**

- In the Federation Canister, handle `GET /.well-known/webfinger` in
  `http_request` (query)
- Parse the `resource` query parameter (e.g.,
  `acct:alice@mastic.social`)
- Extract the handle, resolve it via the Directory Canister
- Return a JSON Resource Descriptor (JRD) with:
  - `subject`: the `acct:` URI
  - `links`: a `self` link pointing to the actor's ActivityPub profile URL
    with `type: application/activity+json`
- Return 404 for unknown handles
- Return 400 for malformed requests

**Acceptance Criteria:**

- `GET /.well-known/webfinger?resource=acct:alice@mastic.social` returns a
  valid JRD with the correct actor URL
- Unknown handles return 404
- Malformed `resource` parameters return 400
- Response has `Content-Type: application/jrd+json`
- Integration test: create user, query WebFinger, verify JRD

### WI-3.3: Serve ActivityPub actor profiles

**Description:** Serve actor profile JSON for remote instances that look up
Mastic users.

**What should be done:**

- In the Federation Canister, handle `GET /users/{handle}` in
  `http_request` (query) when `Accept` header includes
  `application/activity+json`
- Resolve the handle via the Directory Canister
- Fetch the user's profile from their User Canister
- Fetch the user's RSA public key from their User Canister
- Build an ActivityPub `Person` object with:
  - `id`, `url`, `preferredUsername`, `name`, `summary`
  - `inbox`, `outbox`, `followers`, `following` collection URLs
  - `publicKey` block (key ID, owner, PEM-encoded RSA public key)
  - `icon` and `image` if avatar/header are set
- Return the JSON-LD response

**Acceptance Criteria:**

- `GET /users/alice` with the correct Accept header returns a valid
  ActivityPub Person object
- The `publicKey` block contains the correct RSA public key
- Collection URLs are well-formed
- Unknown handles return 404
- Integration test: create user, fetch actor profile, verify all fields

### WI-3.4: Serve ActivityPub collections

**Description:** Serve the `outbox`, `followers`, and `following`
OrderedCollection endpoints for remote instances.

**What should be done:**

- Handle `GET /users/{handle}/outbox` in `http_request`:
  - Return an `OrderedCollection` with `totalItems` and paginated
    `OrderedCollectionPage` items
  - Fetch outbox items from the User Canister
- Handle `GET /users/{handle}/followers` in `http_request`:
  - Return an `OrderedCollection` of follower actor URIs
- Handle `GET /users/{handle}/following` in `http_request`:
  - Return an `OrderedCollection` of following actor URIs
- Support pagination via `page` query parameter

**Acceptance Criteria:**

- Each collection endpoint returns valid ActivityPub OrderedCollection JSON
- Pagination works correctly
- Empty collections return `totalItems: 0`
- Unknown handles return 404
- Integration test: create user with statuses and follows, verify collections

### WI-3.5: Implement HTTP Signatures for outgoing requests

**Description:** Sign all outgoing HTTP requests from the Federation Canister
using the sender's RSA private key, per the HTTP Signatures spec used by
Mastodon.

**What should be done:**

- Implement HTTP Signature generation:
  - Sign headers: `(request-target)`, `host`, `date`, `digest`,
    `content-type`
  - Use RSA-SHA256 algorithm
  - Fetch the sender's private key from their User Canister
  - Build the `Signature` header string
- Add the `Signature` and `Digest` headers to all outgoing ActivityPub
  requests
- Implement a helper to compute SHA-256 digest of the request body

**Acceptance Criteria:**

- All outgoing ActivityPub requests include a valid `Signature` header
- The `Digest` header matches the SHA-256 hash of the body
- The signature can be verified using the sender's public key
- Unit test: sign a request, verify the signature with the public key

### WI-3.6: Implement HTTP Signature verification for incoming requests

**Description:** Verify HTTP Signatures on incoming ActivityPub requests to
ensure authenticity.

**What should be done:**

- In the Federation Canister `http_request_update` handler, before
  processing any incoming activity:
  - Parse the `Signature` header to extract `keyId`, `headers`, `signature`
  - Fetch the remote actor's profile from the `keyId` URL (via
    `ic_cdk::api::management_canister::http_request`)
  - Extract the remote actor's RSA public key
  - Reconstruct the signing string from the specified headers
  - Verify the signature using the remote public key
- Cache remote actor public keys to avoid repeated fetches (with TTL)
- Reject requests with invalid or missing signatures

**Acceptance Criteria:**

- Incoming requests with valid signatures are accepted
- Incoming requests with invalid signatures are rejected with 401
- Incoming requests with missing signatures are rejected with 401
- Remote public keys are cached with a reasonable TTL
- Unit test: construct a signed request, verify it passes validation

### WI-3.7: Implement incoming activity processing (inbox)

**Description:** Process incoming ActivityPub activities received via HTTP POST
to the shared inbox.

**What should be done:**

- In the Federation Canister, handle `POST /inbox` in
  `http_request_update`:
  - Verify HTTP Signature (WI-3.5)
  - Parse the activity JSON
  - Determine the activity type and target
  - Route to the appropriate User Canister(s) via `receive_activity`
- Handle the following incoming activity types:
  - `Create(Note)`: deliver to the target user's inbox
  - `Follow`: deliver to the target user for acceptance
  - `Accept(Follow)`: deliver to the original requester
  - `Reject(Follow)`: deliver to the original requester
  - `Undo(Follow)`: deliver to the target user
  - `Like`: deliver to the status author
  - `Undo(Like)`: deliver to the status author
  - `Announce`: deliver to the target user
  - `Undo(Announce)`: deliver to the target user
  - `Delete`: deliver to affected users
  - `Update(Person)`: update cached remote actor info
  - `Block`: deliver to the blocked user

**Acceptance Criteria:**

- All listed activity types are correctly parsed and routed
- Invalid JSON returns 400
- Unknown activity types are gracefully ignored (return 202)
- Activities targeting non-existent local users return 404
- Integration test: simulate an incoming Create(Note) from a remote instance

### WI-3.8: Implement outgoing activity delivery (HTTP POST)

**Description:** Deliver activities to remote Fediverse instances via signed
HTTP POST requests.

**What should be done:**

- In the Federation Canister `send_activity` handler, when the target is a
  remote actor:
  - Resolve the remote actor's inbox URL (fetch actor profile if not cached)
  - Serialize the activity as JSON-LD
  - Set ActivityPub `to`/`cc` addressing based on visibility:
    - `Public`: `to: [as:Public]`, `cc: [followers collection]`
    - `Unlisted`: `to: [followers collection]`, `cc: [as:Public]`
    - `FollowersOnly`: `to: [followers collection]`, no `as:Public`
    - `Direct`: `to: [mentioned actors only]`, no `cc`
  - Sign the request using the sender's RSA key (WI-3.5)
  - Send the HTTP POST via `ic_cdk::api::management_canister::http_request`
  - Handle retries for transient failures (e.g., 5xx responses)
- Implement delivery to shared inboxes when multiple recipients share the
  same instance
- Handle delivery failures gracefully (log, do not block the caller)

**Acceptance Criteria:**

- Activities are delivered to remote inboxes via signed HTTP POST
- Shared inbox optimization works (one request per remote instance)
- Transient failures are retried (up to a configurable limit)
- Permanent failures (4xx) are not retried
- The caller is not blocked by slow remote deliveries
- `to`/`cc` fields correctly reflect the status visibility level

### WI-3.9: Implement remote actor resolution and caching

**Description:** Fetch and cache remote actor profiles for use in activity
routing and display.

**What should be done:**

- Implement a remote actor resolver in the Federation Canister:
  - Given a remote actor URI, perform WebFinger lookup to find the actor URL
  - Fetch the actor profile via HTTP GET with
    `Accept: application/activity+json`
  - Parse the actor profile to extract: display name, summary, public key,
    inbox URL, followers/following URLs, icon
  - Cache the actor profile in stable memory with a TTL (e.g., 24 hours)
- Provide a method for User Canisters to request remote actor info (for
  display in feeds)

**Acceptance Criteria:**

- Remote actor profiles are fetched and cached
- Cached entries expire after the TTL
- Invalid actor URIs return a descriptive error
- The resolver handles redirects and content negotiation
- Unit test: mock a remote actor endpoint, verify parsing

### WI-3.10: Implement NodeInfo endpoint

**Description:** Serve the NodeInfo endpoint so remote instances and monitoring
tools can discover Mastic's software and protocol information.

**What should be done:**

- Handle `GET /.well-known/nodeinfo` in `http_request`:
  - Return a JSON document with a link to the NodeInfo 2.0 schema URL
- Handle `GET /nodeinfo/2.0` in `http_request`:
  - Return NodeInfo 2.0 JSON with: software name ("mastic"), version,
    protocols (["activitypub"]), open registrations status, usage statistics
    (total users, active users, local posts)
  - Fetch statistics from the Directory Canister

**Acceptance Criteria:**

- `GET /.well-known/nodeinfo` returns a valid link to the NodeInfo endpoint
- `GET /nodeinfo/2.0` returns valid NodeInfo 2.0 JSON
- Statistics reflect actual counts from the Directory Canister
- Integration test: deploy canisters, query NodeInfo, verify response

### WI-3.11: Integration tests for federation flows

**Description:** Write integration tests that exercise the full federation
flows, verifying interoperability with the ActivityPub protocol.

**What should be done:**

- **Test UC13 (Receive Updates from Fediverse):** Simulate a remote instance
  sending a `Create(Note)` activity, verify it appears in the local user's
  feed
- **Test UC14 (Interact with Mastic from Web2):** Simulate a local user
  publishing a status, verify the Federation Canister produces a correctly
  signed HTTP request with the right ActivityPub payload
- **Test WebFinger:** Query WebFinger for a local user, verify the JRD
- **Test Actor Profile:** Fetch a local user's actor profile, verify the
  Person object
- **Test Collections:** Fetch outbox/followers/following collections, verify
  pagination
- **Test HTTP Signature round-trip:** Sign a request, verify it passes
  validation
- **Test incoming Follow from remote:** Simulate a remote Follow, verify the
  local user gets a new follower

**Acceptance Criteria:**

- All federation flows pass as integration tests
- Tests run in CI via `just integration_test`
- Tests simulate remote instances by crafting raw HTTP requests with valid
  signatures
- Each test is independent and can run in isolation
