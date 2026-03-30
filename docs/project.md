---
title: "Project Specification"
layout: page
---

# Mastic

Unleashing the power of IC on the Fediverse

## Introduction

Mastic aims to bring the Fediverse — a decentralised network of interconnected social platforms — natively onto the
Internet Computer ecosystem.

The Fediverse operates through the ActivityPub protocol, enabling independent instances of the social platform on the
network to communicate with one another, thereby creating a distributed social network.

The **Mastic** project brings the possibility both to have a node dedicated to the Internet Computer community that can
finally have a dedicated fully decentralised social network, but also any people that wish to set up their own **Mastic**
node on the **Internet Computer**, empowering the decentralisation of the Fediverse, bringing web3 functionalities on
it, and with the ease the IC brings in spinning up network services.

## Mission

Our mission is to build a fully federated, ActivityPub-compatible social platform running entirely on the Internet
Computer protocol, enabling seamless participation in the Fediverse.

This project aims to bring a truly decentralised social experience to the IC ecosystem by integrating with an already
thriving network of over 1.4 million monthly active users across the Fediverse.

In particular, it will help engage the vibrant community of Rust developers, many of whom are active on Mastodon, by
bridging their preferred social environment with the technical and philosophical foundations of the Internet Computer.

## Impact

Mastic has the potential to reshape the future of federated social platforms by demonstrating how a modern, scalable,
and fully on-chain implementation of ActivityPub can be achieved using Rust and the Internet Computer ecosystem.

While Mastodon has played a central role in popularising the Fediverse, its current architecture — based on Ruby on
Rails and multiple supporting services (PostgreSQL, Redis, Sidekiq, Nginx, etc.) — presents significant barriers to
contribution and scalability.

The complexity of deploying and maintaining Mastodon discourages smaller communities and individual developers, and its
ageing technology stack makes it less attractive to modern developers, particularly those in the Web3 and Rust
ecosystems.

By reimagining an ActivityPub-compatible server as a set of autonomous, composable canisters written in Rust, this
project removes those barriers: no external infrastructure, no ops burden, and a far more approachable codebase for
developers already aligned with the values of decentralisation.

Mastic can not only attract a new wave of Rust and Web3 contributors to the Internet Computer but also act as a
reference architecture for building scalable, open social infrastructure in the post-cloud era.

Additionally, this would finally bring authentication to the Internet Identity on the Fediverse.

## Governance & Sustainability Model

Mastic is not just about a technical implementation of a decentralised social platform - it is a long-term vision for
how open social platforms can be sustainably governed and evolved without relying on centralised control or ad revenue
models.

To ensure sustainability and a decentralised governance model, Mastic will be deployed under a **Service Nervous System
** (SNS), making Mastic a fully on-chain DAO governed by its community.

### DAO-Based Governance

We propose giving the DAO community governance for upgrades, feature proposals, federation decisions and moderation
policies.

Token holders will be able to vote on proposals and allocate funds to the treasury.

Participants in the DAO will be able to apply for moderation and will be elected by the community.

## Market Analysis

### Total Addressable Market

Currently, Mastodon has 1.4 million active users per month.

Most of these are developers, journalists, artists and niche communities. In particular, Mastodon has experienced
significant growth following Elon Musk's acquisition of Twitter.

Of course, some users have left Twitter but have not joined Mastodon, either because they are using BlueSky or other
platforms, such as Reddit or Discord groups.

### Serviceable Available Market

Above 10-15% of the TAM

Our first target is:

- Rust Developers: 2.3 million developers in 2024
- Web3 Developers: about 25k developers in 2024
- IC developers: \~3k

### Serviceable Obtainable Market

A realistic target within the first year could be around 20,000 users, if we primarily focus on developers and web3
developers.

### Differentiation from Traditional Mastodon Instances

Mastic introduces a fundamentally new paradigm for federated social platforms by reimagining the Mastodon experience as
a fully on-chain, modular architecture built natively on the Internet Computer. While Mastodon has pioneered the
Fediverse movement, its traditional deployment model — based on a monolithic Ruby on Rails stack supported by
PostgreSQL, Redis, Sidekiq, and other infrastructure — imposes significant operational overhead and limits scalability.

By contrast, Mastic offers the following distinct advantages:

- Fully on-chain architecture: Every component of Mastic runs within canisters on the Internet Computer, eliminating the
  need for traditional DevOps, hosting, and external databases.
- Modularity and scalability: Each user operates within their User Canister, ensuring composability, privacy, and
  scalability by design.
- Internet Identity integration: Users can sign in with Internet Identity, bringing native IC authentication to the
  Fediverse.
- Decentralised governance: Instead of relying on a central instance admin, Mastic is governed by a DAO through the
  Service Nervous System (SNS), enabling transparent, community-driven decisions around moderation and feature
  development.
- Developer-friendly technology stack: Mastic is built entirely in Rust, offering a modern, secure, and performant
  alternative that resonates with a growing community of Rust and Web3 developers.

These innovations make Mastic not only more accessible for developers and small communities but also more aligned with
the philosophical foundations of the decentralised web.

### User Acquisition Strategy

Mastic’s adoption strategy is designed to activate key communities aligned with its technical foundation and
decentralised vision, and to establish a sustainable user base through targeted outreach and integrations within the
Web3 ecosystem.

Our multi-phase growth approach includes:

- [ ] Developer-first onboarding: We will bootstrap the platform by targeting Rust and Internet Computer developers,
  communities that already value decentralisation, composability, and performance. Early access, bounties, and
  contributor incentives will be designed to attract this group.
- [ ] Fediverse-native advocacy: As a fully compatible ActivityPub implementation, Mastic will be promoted within the
  Fediverse itself. We aim to attract privacy-conscious users and instance operators looking for a lower-maintenance,
  modern alternative to traditional Mastodon deployments.

## Architecture Overview

The **architecture of Mastic** consists of 3 components, each of which will be a standalone canister.

### Frontend

The frontend of Mastic provides an interface where each user can sign in to Mastic and interact with the
Fediverse by publishing Statuses and interacting with other users, including those from the Mastodon Fediverse on the
web2. The User will be able to sign in with Internet Identity on the frontend and interact with the backend canisters
using Candid.

### User Canister

Each user has one User Canister. The User canister is created by the Directory Canister via the IC management canister every time a user signs up on Mastic, and is deleted whenever they delete their account or migrate to another Fediverse instance. The User Canister provides
the interface for users to interact with the Fediverse through the **Inbox** and **Outbox,** implementing the core
functionality of the **ActivityPub** protocol.

The User canister will use [wasm-dbms](https://github.com/veeso/wasm-dbms) to store data inside of a relational
database.

### Directory Canister

The Directory Canister provides an index for all existing users on **Mastic** by creating a
Map between a user’s identity and their handle (e.g., @veeso@mastic.social) and their User Canister instance.

### Federation Canister

The Federation canister implements the HTTP Web server to handle both incoming and outgoing
requests of the **Federation Protocol**[[1](https://www.w3.org/TR/activitypub/#server-to-server-interactions)], which
is used to communicate with the other instances on the **Fediverse**. The Federation canister MUST also implement the
**WebFinger**[[2](https://docs.joinmastodon.org/spec/webfinger/)] protocol to search for users.

## Authorization Model

Mastic uses principal-based authorization, where each canister checks the caller’s principal against an expected set of
principals configured at install time via init args.

- **User → User Canister**: The caller’s principal must match the owner principal that was set when the User Canister was
  installed. This ensures only the canister owner can call methods like *publish\_status*, *update\_profile*, *like\_status*,
  etc.
- **Federation Canister → User Canister**: The User Canister stores the Federation Canister’s principal at install time
  (passed via *UserInstallArgs*). Only the Federation Canister can call *receive\_activity* to deliver incoming activities
  from the Fediverse.
- **User Canister → Federation Canister**: The Federation Canister stores the Directory Canister’s principal at install
  time. The Directory Canister registers each new User Canister principal with the Federation Canister, so the Federation
  Canister maintains a list of all authorised User Canister principals that can call *send\_activity*.
- **Directory Canister**: Moderator actions (*add\_moderator*, *remove\_moderator*, *suspend*) require the caller’s
  principal to be present in the moderator list. The initial moderator is set at install time via *DirectoryInstallArgs*.

## Interface

Here’s a description of the Candid interface of the **Directory**, **Federation** and **User** canisters. Types are not
yet defined, but only calls are provided to provide an overview of the flows that will need to be implemented. For more
information on the flows, see the **User Stories**.

### Directory Interface

service : (*DirectoryInstallArgs*) -> {

add\_moderator : (*AddModeratorArgs*) -> (*AddModeratorResponse*);

delete\_profile : () -> (*DeleteProfileResponse*);

get\_user : (*GetUserArgs*) -> (*GetUserResponse*) query;

remove\_moderator : (*RemoveModeratorArgs*) -> (*RemoveModeratorResponse*);

search\_profiles : (*SearchProfilesArgs*) -> (*SearchProfilesResponse*) query;

sign\_up : (*text*) -> (*SignUpResponse*);

suspend : (*SuspendArgs*) -> (*SuspendResponse*);

user\_canister : (opt *Principal*) -> (*UserCanisterResponse*) query;

whoami : () -> (*WhoAmIResponse*) query

}

### Federation Interface

service : (*FederationInstallArgs*) -> {

http\_request : (*HttpRequest*) -> (*HttpResponse*) query;

http\_request\_update : (*HttpRequest*) -> (*HttpResponse*);

send\_activity : (*SendActivityArgs*) -> (*SendActivityResponse*)

}

### User Interface

service : (*UserInstallArgs*) -> {

accept\_follow : (*AcceptFollowArgs*) -> (*AcceptFollowResponse*);

block\_user : (*BlockUserArgs*) -> (*BlockUserResponse*);

boost\_status : (*BoostStatusArgs*) -> (*BoostStatusResponse*);

delete\_profile : () -> (*DeleteProfileResponse*);

delete\_status : (*DeleteStatusArgs*) -> (*DeleteStatusResponse*);

follow\_user : (*FollowUserArgs*) -> (*FollowUserResponse*);

get\_followers : (*GetFollowersArgs*) -> (*GetFollowersResponse*) query;

get\_following : (*GetFollowingArgs*) -> (*GetFollowingResponse*) query;

get\_liked : (*GetLikedArgs*) -> (*GetLikedResponse*) query;

get\_profile : () -> (*GetProfileResponse*) query;

like\_status : (*LikeStatusArgs*) -> (*LikeStatusResponse*);

publish\_status : (*PublishStatusArgs*) -> (*PublishStatusResponse*);

read\_feed : (*ReadFeedArgs*) -> (*ReadFeedResponse*) query;

receive\_activity : (*ReceiveActivityArgs*) -> (*ReceiveActivityResponse*);

reject\_follow : (*RejectFollowArgs*) -> (*RejectFollowResponse*);

undo\_boost : (*UndoBoostArgs*) -> (*UndoBoostResponse*);

undo\_like : (*UndoLikeArgs*) -> (*UndoLikeResponse*);

unfollow\_user : (*UnfollowUserArgs*) -> (*UnfollowUserResponse*);

update\_profile : (*UpdateProfileArgs*) -> (*UpdateProfileResponse*)

}

# User Stories

### UC1: As a User, I should be able to create a Profile

- **Alice** lands on Mastic **Frontend**
- **Alice** signs in with her **Internet Identity**
- **Alice** sends a *sign\_up* call
- The **Directory Canister** establishes a connection between Alice’s identity and her handle
- The **Directory Canister** starts a worker to create her **User Canister**
- The **Directory Canister** creates **Alice’s User Canister** via the **IC management canister**
- The **Directory Canister** installs **Alice’s User Canister** via the **IC management canister**
- The **Directory Canister** stores **Alice’s User Canister** ID for **Alice**’s Identity
- **Alice** queries her **User Canister** ID with *user\_canister*
- The **Directory Canister** returns the ID to **Alice**

### UC2: As a User, I should be able to Sign In

- **Alice** lands on Mastic **Frontend**
- **Alice** signs in with her **Internet Identity**
- **Alice** queries the **Directory Canister** to get her user\_canister with *whoami*
- The **Directory Canister** returns her **User Canister**’s principal

### UC3: As a User, I should be able to update my Profile

- **Alice** signs in with her **Internet Identity**
- **Alice** queries the **Directory Canister** with *whoami* to obtain her **User Canister** principal
- **Alice** calls *update\_profile* on her **User Canister** with the updated fields (display name, bio, avatar, etc.)
- The **User Canister** persists the changes
- The **User Canister** sends an *Update* activity to the **Federation Canister** so that remote followers receive the updated profile

### UC4: As a User, I should be able to delete my Profile

- **Alice** signs in with her **Internet Identity**
- **Alice** calls *delete\_profile* on the **Directory Canister**
- The **Directory Canister** creates a tombstone for Alice and starts a delete worker
- The **Directory Canister** notifies **Alice’s User Canister**, which aggregates a *Delete* activity for all of Alice’s followers
- The **User Canister** sends the *Delete* activity to the **Federation Canister**
- The **Federation Canister** buffers the activity data, then forwards it to all remote followers
- The **Directory Canister** deletes **Alice’s User Canister** via the **IC management canister**

### UC5: As a User, I should be able to follow another Profile

- **Alice** signs in and obtains her **User Canister** principal via the **Directory Canister**
- **Alice** calls *follow\_user* on her **User Canister** with the target user’s handle
- If the target is **local**: the **User Canister** resolves the handle via the **Directory Canister**, then sends a *Follow* activity to the target **User Canister** through the **Federation Canister**
- If the target is **remote**: the **User Canister** sends a *Follow* activity to the **Federation Canister**, which forwards it to the remote instance via HTTP
- When the target accepts, an *Accept* activity is delivered back, and Alice’s **User Canister** records the follow relationship

### UC6: As a User, I should be able to remove a Following

- **Alice** signs in and obtains her **User Canister** principal via the **Directory Canister**
- **Alice** calls *unfollow\_user* on her **User Canister** with the target user’s handle
- The **User Canister** removes the follow relationship locally
- The **User Canister** sends an *Undo(Follow)* activity through the **Federation Canister** to notify the target (local or remote) that Alice has unfollowed them

### UC7: As a User, I should be able to see a user’s profile

- **Alice** signs in and obtains her **User Canister** principal via the **Directory Canister**
- **Alice** calls *get\_user* on the **Directory Canister** with the target handle
- The **Directory Canister** returns the target user’s **User Canister** principal
- **Alice** calls *get\_profile* on the target **User Canister** to retrieve their public profile information

### UC8: As a User, I should be able to search for other users

- **Alice** signs in and obtains her **User Canister** principal via the **Directory Canister**
- **Alice** calls *search\_profiles* on the **Directory Canister** with a search query
- The **Directory Canister** returns a list of matching user handles and their **User Canister** principals

### UC9: As a User, I should be able to create a Status

- **Alice** signs in and obtains her **User Canister** principal via the **Directory Canister**
- **Alice** calls *publish\_status* on her **User Canister** with the status content
- The **User Canister** stores the status in Alice’s outbox
- The **User Canister** aggregates a *Create(Note)* activity for each follower of Alice
- The **User Canister** sends the activities to the **Federation Canister**
- The **Federation Canister** routes local activities through the **Directory Canister** to each local follower’s **User Canister** inbox
- The **Federation Canister** forwards remote activities via HTTP to external Fediverse instances

### UC10: As a User, I should be able to like a Status

- **Alice** signs in and obtains her **User Canister** principal via the **Directory Canister**
- **Alice** calls *like\_status* on her **User Canister** with the status ID
- The **User Canister** records the like in Alice’s outbox
- The **User Canister** sends a *Like* activity to the **Federation Canister**
- The **Federation Canister** routes the activity to the status author’s **User Canister** (local) or forwards it via HTTP (remote)

### UC11: As a User, I should be able to boost a Status

- **Alice** signs in and obtains her **User Canister** principal via the **Directory Canister**
- **Alice** calls *boost\_status* on her **User Canister** with the status ID
- The **User Canister** records the boost in Alice’s outbox
- The **User Canister** sends an *Announce* activity to the **Federation Canister**
- The **Federation Canister** routes the activity to the status author and Alice’s followers, handling both local delivery (through the **Directory Canister**) and remote forwarding via HTTP

### UC12: As a User, I should be able to read my Feed

- **Alice** signs in and obtains her **User Canister** principal via the **Directory Canister**
- **Alice** calls *read\_feed* on her **User Canister** with pagination parameters
- The **User Canister** aggregates the feed from Alice’s inbox (statuses from followed users) and outbox (Alice’s own statuses)
- The **User Canister** returns the paginated feed to **Alice**

### UC13: As a User, I should be able to receive updates from users I follow on other Fediverse instances

- **Bob** publishes a status on a remote Mastodon instance
- The remote instance dispatches a *Create(Note)* activity to the **Federation Canister** via HTTP
- The **Federation Canister** verifies the HTTP Signature and resolves the target user via the **Directory Canister**
- The **Federation Canister** calls *receive\_activity* on **Alice’s User Canister** to deliver the status to her inbox
- **Alice** reads her feed and sees Bob’s status

### UC14: As a User on a Web2 Mastodon Instance, I should be able to receive updates and interact with users on Mastic

- **Alice** publishes a status on Mastic
- The **Federation Canister** forwards the *Create(Note)* activity via HTTP to **Bob’s** remote Mastodon instance
- **Bob** sees Alice’s status in his feed
- **Bob** likes or replies to Alice’s status
- The remote instance sends the corresponding activity (*Like*, *Create(Note)* reply) to the **Federation Canister** via HTTP
- The **Federation Canister** routes the activity to **Alice’s User Canister**

### UC15: As a Moderator, I should be able to remove a Status violating the policy

- A **Moderator** signs in with their **Internet Identity**
- The **Moderator** identifies a status that violates the instance policy
- The **Moderator** calls *delete\_status* on the offending **User Canister** (authorised by the moderator principal stored in the **Directory Canister**)
- The **User Canister** removes the status from the user’s outbox
- The **User Canister** sends a *Delete* activity through the **Federation Canister** to notify followers

### UC16: As a Moderator, I should be able to suspend a Profile violating the policy

- A **Moderator** signs in with their **Internet Identity**
- The **Moderator** calls *suspend* on the **Directory Canister** with the offending user’s handle
- The **Directory Canister** marks the user as suspended, preventing further API calls
- The **Directory Canister** notifies the **User Canister**, which sends a *Delete* activity through the **Federation Canister** to notify remote followers that the account is no longer active

## Milestones

These are the milestones we plan to achieve during the first year of Mastic's development cycle.

### Milestone 0 - Proof of concept

Duration: 1.5 months

First implementation to demo the social platform.

Only basic functionalities are implemented, such as signing up, posting statuses, and reading the feed.

User stories:

- UC1
- UC2
- UC9
- UC5
- UC12
- UC7

### Milestone 1 - Standalone Mastic Node

Duration: 3 months

Mastic is set up; we build all the data structures required by the user to sign up, operate on statuses (Create, Delete,
like, and Boost), and find and follow other users on the Mastic node.

This Milestone won’t include the integration with the Fediverse yet, but it will provide an already usable Social
Network, ready to be integrated onto the Fediverse soon.

These stories must be implemented during this phase:

- UC3
- UC4
- UC6
- UC8
- UC10
- UC11
- UC15
- UC16

### Milestone 2 - Integrating the Fediverse

Duration: 2 months

This step requires implementing the **Federation Canister** using the **Federation Protocol**.

This will allow Mastic to be fully integrated with the Fediverse ecosystem.

These user stories must be implemented during this phase:

- UC13
- UC14

### Milestone 3 - SNS Launch

Duration: 1 month

During this phase, we plan to launch Mastic on the SNS to add a fully decentralised governance to Mastic.

We need to implement a comprehensive integration for voting, specifically for adding and removing moderators and updating
policies.

## Reference

1. ActivityPub: [https://www.w3.org/TR/activitypub/](https://www.w3.org/TR/activitypub/)
2. ActivityStreams: [https://www.w3.org/TR/activitystreams-core/](https://www.w3.org/TR/activitystreams-core/)
3. Mastodon ActivityPub
   Spec: [https://docs.joinmastodon.org/spec/activitypub/](https://docs.joinmastodon.org/spec/activitypub/)
4. ActivityPub Federation framework implemented with
   Rust: [https://docs.rs/activitypub\_federation/0.6.5/activitypub\_federation/](https://docs.rs/activitypub_federation/0.6.5/activitypub_federation/)
5. Webfinger: [https://docs.joinmastodon.org/spec/webfinger/](https://docs.joinmastodon.org/spec/webfinger/)
