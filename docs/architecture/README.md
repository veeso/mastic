# Mastic Architecture Documentation

- [Mastic Architecture Documentation](#mastic-architecture-documentation)
  - [Scope](#scope)
  - [Architecture Overview](#architecture-overview)
  - [Flows](#flows)
    - [Create Profile](#create-profile)
    - [Sign In](#sign-in)
    - [Update Profile](#update-profile)
    - [Delete Profile](#delete-profile)
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
          fed["Federation Canister"]
          os["Orbit Station"]
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
    participant OS as Orbit Station

    A->>II: Sign In with II
    A->>DIR: Create Profile (candid)
    DIR->>DIR: Add map between user and handle
    DIR->>DIR: Start worker to create user canister
    DIR->>OS: Create User Canister for Alice
    OS->>DIR: Request ID
    DIR->>OS: Get status for Request
    OS->>UC: Create Canister
    OS->>DIR: Created canister Principal
    DIR->>OS: Install User Canister
    OS->>DIR: Request ID
    DIR->>OS: Check install status
    OS->>UC: Install Canister
    OS->>DIR: Installation completed
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

```mermaid
sequenceDiagram
    actor A as Alice
    participant UC as Alice's User Canister
    participant DIR as Directory Canister
    participant FED as Federation Canister
    participant OS as Orbit Station
    participant M as Mastodon Web2

    A->>DIR: Delete profile (candid)
    DIR->>A: Ok
    DIR->>DIR: Create tombstone for Alice
    DIR->>DIR: Start delete canister worker
    DIR->>UC: Notify Delete
    UC->>UC: Aggregate notification based on followers
    UC->>FED: Send Delete Activity
    UC->>DIR: Activity Sent
    DIR->>OS: Delete Canister
    OS->>UC: Delete Canister
    OS->>DIR: Canister Deleted
    FED->>M: Forward Delete Activity
```

### Create Status

```mermaid
sequenceDiagram
    actor A as Alice
    participant UC as Alice's User Canister
    participant DIR as Directory Canister
    participant FED as Federation Canister
    participant M as Mastodon Web2
    actor B as Bob

    A->>DIR: Get Alice's User Canister (candid)
    DIR->>A: User Canister Principal
    A->>UC: Create Toot (candid)
    UC->>UC: Stores Toot to Alice Outbox
    UC->>UC: Aggregate outbox message for each one following Alice or hashtag
    UC->>FED: Forward Create Status Activities (ic)
    FED->>M: Forward Activities (ActivityPub)
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
    participant M as Mastodon Web2

    A->>DIR: Get Alice's User Canister (candid)
    DIR->>A: User Canister Principal
    A->>UC: Like Status (candid)
    UC->>UC: Stores Like to Alice Outbox
    UC->>FED: Forward Like Status Activity (ic)
    FED->>M: Forward Activity (ActivityPub)
```

### Boost Status

```mermaid
sequenceDiagram
    actor A as Alice
    participant UC as Alice's User Canister
    participant DIR as Directory Canister
    participant FED as Federation Canister
    participant M as Mastodon Web2

    A->>DIR: Get Alice's User Canister (candid)
    DIR->>A: User Canister Principal
    A->>UC: Boost Status (candid)
    UC->>UC: Stores Boost to Alice Outbox
    UC->>FED: Forward Boost Status Activity (ic)
    FED->>M: Forward Activity (ActivityPub)
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
    UC->>UC: Remove Status from Alice Outbox
    UC->>FED: Forward Delete Status Activity (ic)
    FED->>M: Forward Activity (ActivityPub)
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
