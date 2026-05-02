# Flows

This page documents the sequence diagrams for all major Mastic flows.
Each flow maps to one or more [user stories](../project.md#user-stories)
defined in the project specification.

## Create Profile

```mermaid
sequenceDiagram
    actor A as Alice
    participant II as Internet Identity
    participant UC as Alice's User Canister
    participant DIR as Directory Canister
    participant IC as IC Management Canister

    A->>II: Sign In with II
    A->>DIR: Create Profile (candid)
    DIR->>DIR: Add map between user and handle
    DIR->>DIR: Start worker to create user canister
    DIR->>IC: create_canister
    IC-->>DIR: Canister ID
    DIR->>IC: install_code (User Canister WASM)
    IC-->>UC: Install Canister
    DIR->>DIR: Store User Canister Principal for Alice
    A->>DIR: Get user canister Principal
    DIR->>A: Principal of User Canister
```

## Sign In

```mermaid
sequenceDiagram
    actor A as Alice
    participant II as Internet Identity
    participant DIR as Directory Canister

    A->>II: Sign In with II
    A->>DIR: Get User Canister (candid)
    DIR->>A: Return Canister ID
```

## Update Profile

```mermaid
sequenceDiagram
    actor A as Alice
    participant UC as Alice's User Canister
    participant DIR as Directory Canister

    A->>DIR: Get Alice's User Canister (candid)
    DIR->>A: User Canister Principal
    A->>UC: Update Profile (candid)
    UC->>UC: Update Profile in User Canister
```

## Delete Profile

> **Note:** The Federation Canister must buffer the Delete activity data
> before the User Canister is destroyed, since the User Canister will no
> longer exist to serve actor profile requests after deletion.

```mermaid
sequenceDiagram
    actor A as Alice
    participant UC as Alice's User Canister
    participant DIR as Directory Canister
    participant FED as Federation Canister
    participant IC as IC Management Canister
    participant M as Mastodon Web2

    A->>DIR: Delete profile (candid)
    DIR->>A: Ok
    DIR->>DIR: Create tombstone for Alice
    DIR->>DIR: Start delete canister worker
    DIR->>UC: Notify Delete
    UC->>UC: Aggregate notification based on followers
    UC->>FED: Send Delete Activity
    FED->>FED: Buffer Delete activity data
    FED->>DIR: Route Delete to local followers
    DIR->>DIR: Resolve local follower User Canisters
    FED->>M: Forward Delete Activity to remote followers
    UC->>DIR: Activity Sent
    DIR->>IC: stop_canister + delete_canister
    IC-->>DIR: Canister Deleted
```

## Follow User

The follow lifecycle has three phases: **request**, **pending**, and
**accept/reject**. Alice sends a follow request; Bob's canister stores
it as a pending follow request; Bob reviews pending requests and
accepts or rejects each one.

### Send follow request (local)

```mermaid
sequenceDiagram
    actor A as Alice
    participant UC as Alice's User Canister
    participant DIR as Directory Canister
    participant FED as Federation Canister
    participant BUC as Bob's User Canister

    A->>DIR: Get Alice's User Canister (candid)
    DIR->>A: User Canister Principal
    A->>UC: follow_user (candid)
    UC->>UC: Store pending follow (status: Pending)
    UC->>FED: Send Follow Activity
    FED->>DIR: Resolve Bob's User Canister
    DIR->>FED: Bob's User Canister Principal
    FED->>BUC: receive_activity (Follow)
    BUC->>BUC: Store follow request in follow_requests table
```

### Send follow request (remote)

```mermaid
sequenceDiagram
    actor A as Alice
    participant UC as Alice's User Canister
    participant DIR as Directory Canister
    participant FED as Federation Canister
    participant M as Mastodon Web2

    A->>DIR: Get Alice's User Canister (candid)
    DIR->>A: User Canister Principal
    A->>UC: follow_user (candid)
    UC->>UC: Store pending follow (status: Pending)
    UC->>FED: Send Follow Activity
    FED->>M: Forward Follow Activity (ActivityPub / HTTP Signature)
```

### Accept follow request (local)

```mermaid
sequenceDiagram
    actor B as Bob
    participant BUC as Bob's User Canister
    participant FED as Federation Canister
    participant DIR as Directory Canister
    participant UC as Alice's User Canister

    B->>BUC: get_follow_requests (candid)
    BUC->>B: List of pending follow requests
    B->>BUC: accept_follow (candid, Alice's actor URI)
    BUC->>BUC: Add Alice to followers table
    BUC->>BUC: Remove request from follow_requests table
    BUC->>FED: Send Accept(Follow) Activity
    FED->>DIR: Resolve Alice's User Canister
    DIR->>FED: Alice's User Canister Principal
    FED->>UC: receive_activity (Accept(Follow))
    UC->>UC: Update following status: Accepted
```

### Accept follow request (remote target accepts)

```mermaid
sequenceDiagram
    participant M as Mastodon Web2
    participant FED as Federation Canister
    participant DIR as Directory Canister
    participant UC as Alice's User Canister

    M->>FED: Send Accept(Follow) Activity (ActivityPub)
    FED->>DIR: Resolve Alice's User Canister
    DIR->>FED: Alice's User Canister Principal
    FED->>UC: receive_activity (Accept(Follow))
    UC->>UC: Update following status: Accepted
```

### Reject follow request (local)

```mermaid
sequenceDiagram
    actor B as Bob
    participant BUC as Bob's User Canister
    participant FED as Federation Canister
    participant DIR as Directory Canister
    participant UC as Alice's User Canister

    B->>BUC: get_follow_requests (candid)
    BUC->>B: List of pending follow requests
    B->>BUC: reject_follow (candid, Alice's actor URI)
    BUC->>BUC: Remove request from follow_requests table
    BUC->>FED: Send Reject(Follow) Activity
    FED->>DIR: Resolve Alice's User Canister
    DIR->>FED: Alice's User Canister Principal
    FED->>UC: receive_activity (Reject(Follow))
    UC->>UC: Remove pending follow entry
```

### Reject follow request (remote target rejects)

```mermaid
sequenceDiagram
    participant M as Mastodon Web2
    participant FED as Federation Canister
    participant DIR as Directory Canister
    participant UC as Alice's User Canister

    M->>FED: Send Reject(Follow) Activity (ActivityPub)
    FED->>DIR: Resolve Alice's User Canister
    DIR->>FED: Alice's User Canister Principal
    FED->>UC: receive_activity (Reject(Follow))
    UC->>UC: Remove pending follow entry
```

## Unfollow User

```mermaid
sequenceDiagram
    actor A as Alice
    participant UC as Alice's User Canister
    participant DIR as Directory Canister
    participant FED as Federation Canister
    participant BUC as Bob's User Canister (local)
    participant M as Mastodon Web2

    A->>DIR: Get Alice's User Canister (candid)
    DIR->>A: User Canister Principal
    A->>UC: unfollow_user (candid)
    UC->>UC: Remove following (Bob)
    UC->>FED: Send Undo(Follow) Activity
    alt Bob is local
        FED->>DIR: Resolve Bob's User Canister
        FED->>BUC: Deliver Undo(Follow) activity
        BUC->>BUC: Remove follower (Alice)
    else Bob is remote
        FED->>M: Forward Undo(Follow) Activity (ActivityPub)
    end
```

## Block User

```mermaid
sequenceDiagram
    actor A as Alice
    participant UC as Alice's User Canister
    participant DIR as Directory Canister
    participant FED as Federation Canister
    participant BUC as Bob's User Canister (local)
    participant M as Mastodon Web2

    A->>DIR: Get Alice's User Canister (candid)
    DIR->>A: User Canister Principal
    A->>UC: block_user (candid)
    UC->>UC: Record block locally
    UC->>FED: Send Block Activity
    alt Bob is local
        FED->>DIR: Resolve Bob's User Canister
        FED->>BUC: Deliver Block activity
        BUC->>BUC: Hide Alice's profile from Bob
    else Bob is remote
        FED->>M: Forward Block Activity (ActivityPub)
    end
```

## Create Status

```mermaid
sequenceDiagram
    actor A as Alice
    participant UC as Alice's User Canister
    participant DIR as Directory Canister
    participant FED as Federation Canister
    participant BUC as Bob's User Canister (local)
    participant M as Mastodon Web2
    actor B as Bob (remote)

    A->>DIR: Get Alice's User Canister (candid)
    DIR->>A: User Canister Principal
    A->>UC: Create Status (candid)
    UC->>UC: Store Status in Alice's Outbox
    UC->>UC: Aggregate Create activity for each follower
    UC->>FED: Forward Create Status Activities (ic)
    FED->>DIR: Resolve local followers
    DIR->>FED: Local follower User Canister principals
    FED->>BUC: Deliver Create activity to local follower inboxes
    FED->>M: Forward Create activities to remote instances (ActivityPub)
    B->>M: Get Feed
    M->>B: Return Alice's Status
```

## Like Status

```mermaid
sequenceDiagram
    actor A as Alice
    participant UC as Alice's User Canister
    participant DIR as Directory Canister
    participant FED as Federation Canister
    participant TUC as Target User Canister (local)
    participant M as Mastodon Web2

    A->>DIR: Get Alice's User Canister (candid)
    DIR->>A: User Canister Principal
    A->>UC: Like Status (candid)
    UC->>UC: Store Like in Alice's Outbox
    UC->>FED: Forward Like Activity (ic)
    alt Status author is local
        FED->>DIR: Resolve author's User Canister
        FED->>TUC: Deliver Like activity
    else Status author is remote
        FED->>M: Forward Like Activity (ActivityPub)
    end
```

## Boost Status

```mermaid
sequenceDiagram
    actor A as Alice (booster)
    participant UC as Booster User Canister
    participant FED as Federation Canister
    participant DIR as Directory Canister
    participant TUC as Target User Canister (author)
    participant Fol as Follower User Canisters

    A->>UC: boost_status(status_url)
    UC->>FED: fetch_status(uri, requester=alice_actor_uri)
    FED->>DIR: lookup handle from URI
    DIR-->>FED: target canister id
    FED->>TUC: get_local_status(id, requester=alice_actor_uri)
    TUC-->>FED: Status (visibility-filtered)
    FED-->>UC: Status
    UC->>UC: tx { wrapper Status, Boost row, FeedEntry } (shared snowflake)
    UC->>FED: send_activity(Batch[Announce])
    FED->>TUC: receive_activity(Announce)  -- bumps boost_count
    FED->>Fol: receive_activity(Announce)  -- inbox row + feed entry
    UC-->>A: Ok
```

The booster's User Canister never trusts boost content from its caller:
the wrapper row's `content`, `spoiler_text`, and `sensitive` are
populated from the `Status` returned by `Federation.fetch_status`,
which in turn dereferences the local author through
`User.get_local_status` (Milestone 1; Milestone 3 will extend the
remote branch via HTTPS outcalls).

A single Snowflake is reused as `boosts.id`, the wrapper `statuses.id`,
the `feed.id` for the booster's outbox entry, and the `Announce`
activity `id` (`<own_actor_uri>/statuses/<snowflake>`).

`boost_status` is **idempotent**: a duplicate boost of the same
`status_url` returns `Ok` without inserting a second wrapper or
re-emitting the `Announce`. `undo_boost` reverses the flow — it deletes
the `boosts` row, the wrapper `statuses` row, and the `feed` outbox
entry, then dispatches an `Undo(Announce)` to followers and the
original author. `undo_boost` is also idempotent.

Remote author / follower delivery via HTTPS is Milestone 3.

## Delete Status

```mermaid
sequenceDiagram
    actor A as Alice
    participant UC as Alice's User Canister
    participant DIR as Directory Canister
    participant FED as Federation Canister
    participant M as Mastodon Web2

    A->>DIR: Get Alice's User Canister (candid)
    DIR->>A: User Canister Principal
    A->>UC: Delete Status (candid)
    UC->>UC: Remove Status from Alice's Outbox
    UC->>FED: Forward Delete Status Activity (ic)
    FED->>DIR: Resolve local followers
    FED->>FED: Deliver Delete to local follower inboxes
    FED->>M: Forward Delete Activity to remote instances (ActivityPub)
```

## Read Feed

```mermaid
sequenceDiagram
    actor A as Alice
    participant UC as Alice's User Canister
    participant DIR as Directory Canister

    A->>DIR: Get Alice's User Canister (candid)
    DIR->>A: User Canister Principal
    A->>UC: Read Feed page (candid)
    UC->>UC: Aggregate feed from Alice's Inbox and Outbox
    UC->>A: Return Feed
```

## Receive Updates from Fediverse

```mermaid
sequenceDiagram
    actor A as Alice
    participant UC as Alice's User Canister
    participant DIR as Directory Canister
    participant FED as Federation Canister
    participant M as Mastodon Web2
    actor B as Bob

    B->>M: Publish Status
    M->>M: Get who Follows Bob
    M->>FED: Dispatch create Status Activity for Alice
    FED->>DIR: Get User Canister for Alice
    DIR->>FED: User Canister ID
    FED->>UC: Put Status to Alice's Inbox
    A->>UC: Read feed
    UC->>A: Return Bob's Post
```
