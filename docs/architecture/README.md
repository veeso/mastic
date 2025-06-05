# Mastic Architecture Documentation

- [Mastic Architecture Documentation](#mastic-architecture-documentation)
  - [Scope](#scope)
  - [Architecture Overview](#architecture-overview)
  - [Flows](#flows)
    - [Create Profile](#create-profile)
    - [Create Status](#create-status)

## Scope

This document outlines the architecture of Mastic, with a focus on the core components and their interactions with the users and the Fediverse.

## Architecture Overview

```mermaid
block-beta
    columns 1
    Alice (("Alice"))

    space

    block:mastic
          uc["User Canister"]
          dir["Directory Canister"]
          fed["Federation Canister"]
          os["Orbit Station"]
    end

    space

    mastodon("Mastodon Web2")

    space

    Bob(("Bob"))

    Alice --> mastic
    mastic --> mastodon
    mastodon --> Bob

```

## Flows

### Create Profile

```mermaid
sequenceDiagram
    participant A as Alice
    participant UC as User Canister
    participant DIR as Directory Canister
    participant OS as Orbit Station

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

### Create Status

```mermaid
sequenceDiagram
    participant A as Alice
    participant UC as User Canister
    participant DIR as Directory Canister
    participant FED as Federation Canister
    participant M as Mastodon Web2
    participant B as Bob

    A->>DIR: Get Alice's User Canister (candid)
    DIR->>A: User Canister Principal
    A->>UC: Create Toot (candid)
    UC->>UC: Stores Toot to Alice Outbox
    UC->>FED: Forward Create Status Activity (ic)
    FED->>M: Forward Activity (ActivityPub)
    M->>B: Shows Status on Feed
```
