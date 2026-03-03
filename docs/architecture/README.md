# Mastic Architecture Documentation

- [Mastic Architecture Documentation](#mastic-architecture-documentation)
  - [Scope](#scope)
  - [Architecture Overview](#architecture-overview)
  - [Flows](#flows)
    - [Create Profile](#create-profile)
    - [Sign In](#sign-in)
    - [Update Profile](#update-profile)
    - [Delete Profile](#delete-profile)
    - [Follow User](#follow-user)
    - [Unfollow User](#unfollow-user)
    - [Block User](#block-user)
    - [Create Status](#create-status)
    - [Like Status](#like-status)
    - [Boost Status](#boost-status)
    - [Delete Status](#delete-status)
    - [Read Feed](#read-feed)
    - [Receive Updates from Fediverse](#receive-updates-from-fediverse)

## Scope

This document outlines the architecture of Mastic, with a focus on the core components and their interactions with the users and the Fediverse.

## Architecture Overview

```mermaid
block-beta
    columns 2
    Alice (("Alice")):2

    block:mastic
        columns 2
          fe["Frontend"]:2
          uc["User Canister"]
          dir["Directory Canister"]
          fed["Federation Canister"]:2
    end

    space

    mastodon("Mastodon Web2"):2

    Bob(("Bob")):2

    Alice --> mastic
    mastic --> mastodon
    mastodon --> Bob

```

## Flows

### Create Profile

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

### Sign In

```mermaid
sequenceDiagram
    actor A as Alice
    participant II as Internet Identity
    participant DIR as Directory Canister

    A->>II: Sign In with II
    A->>DIR: Get User Canister (candid)
    DIR->>A: Return Canister ID
```

### Update Profile

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

### Delete Profile

> **Note:** The Federation Canister must buffer the Delete activity data before the User Canister is destroyed,
> since the User Canister will no longer exist to serve actor profile requests after deletion.

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

### Follow User

#### Local follow (both users on Mastic)

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
    UC->>FED: Send Follow Activity
    FED->>DIR: Resolve Bob's User Canister
    DIR->>FED: Bob's User Canister Principal
    FED->>BUC: Deliver Follow activity
    BUC->>BUC: Record follower (Alice)
    BUC->>FED: Send Accept Activity
    FED->>DIR: Resolve Alice's User Canister
    FED->>UC: Deliver Accept activity
    UC->>UC: Record following (Bob)
```

#### Remote follow (target on external Fediverse instance)

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
    UC->>FED: Send Follow Activity
    FED->>M: Forward Follow Activity (ActivityPub / HTTP Signature)
    M->>M: Record follower (Alice)
    M->>FED: Send Accept Activity (ActivityPub)
    FED->>DIR: Resolve Alice's User Canister
    FED->>UC: Deliver Accept activity
    UC->>UC: Record following (remote user)
```

### Unfollow User

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

### Block User

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

### Create Status

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

### Like Status

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

### Boost Status

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
    A->>UC: Boost Status (candid)
    UC->>UC: Store Boost in Alice's Outbox
    UC->>FED: Forward Announce Activity (ic)
    alt Status author is local
        FED->>DIR: Resolve author's User Canister
        FED->>TUC: Deliver Announce activity
    else Status author is remote
        FED->>M: Forward Announce Activity (ActivityPub)
    end
    Note over FED: Also delivers to Alice's followers (local + remote)
```

### Delete Status

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

### Read Feed

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

### Receive Updates from Fediverse

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
