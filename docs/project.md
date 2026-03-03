Mastic

Unleashing the power of IC on the Fediverse

# Introduction

Mastic aims to bring the Fediverse — a decentralised network of interconnected social platforms — natively onto the Internet Computer ecosystem.

The Fediverse operates through the ActivityPub protocol, enabling independent instances of the social platform on the network to communicate with one another, thereby creating a distributed social network.

The **Mastic** project brings the possibility both to have a node dedicated to the Internet Computer community that can finally have a dedicated fully decentralised social network, but also any people that will to setup their own **Mastic** node on the **Internet Computer**, empowering the decentralisation of the Fediverse, bringing web3 functionalities on it, and with the ease the IC brings in spinning up network services.

# Mission

Our mission is to build a fully federated, ActivityPub-compatible social platform running entirely on the Internet Computer protocol, enabling seamless participation in the Fediverse.

This project aims to bring a truly decentralised social experience to the IC ecosystem by integrating with an already thriving network of over 1.4 million monthly active users across the Fediverse.

In particular, it will help engage the vibrant community of Rust developers, many of whom are active on Mastodon, by bridging their preferred social environment with the technical and philosophical foundations of the Internet Computer.

Our final commitment is to unleash the power of the Internet Computer to build the **Infiniverse**, where, along with full compatibility with the Fediverse, we implement even more capabilities, such as integrating the Internet Identity, wallets for tips and internal transactions, decentralised governance and premium features.

# Impact

Mastic has the potential to reshape the future of federated social platforms by demonstrating how a modern, scalable, and fully on-chain implementation of ActivityPub can be achieved using Rust and the Internet Computer ecosystem.

While Mastodon has played a central role in popularising the Fediverse, its current architecture — based on Ruby on Rails and multiple supporting services (PostgreSQL, Redis, Sidekiq, Nginx, etc.) — presents significant barriers to contribution and scalability.

The complexity of deploying and maintaining Mastodon discourages smaller communities and individual developers, and its ageing technology stack makes it less attractive to modern developers, particularly those in the Web3 and Rust ecosystems.

By reimagining an ActivityPub-compatible server as a set of autonomous, composable canisters written in Rust, this project removes those barriers: no external infrastructure, no ops burden, and a far more approachable codebase for developers already aligned with the values of decentralisation.

Mastic can not only attract a new wave of Rust and Web3 contributors to the Internet Computer but also act as a reference architecture for building scalable, open social infrastructure in the post-cloud era.

Additionally, this would finally bring authentication to the Internet Identity on the Fediverse.

# Governance & Sustainability Model

Mastic is not just about a technical implementation of a decentralised social platform - it is a long-term vision for how open social platforms can be sustainably governed and evolved without relying on centralised control or ad revenue models.

To ensure sustainability and a decentralised governance model, Mastic will be deployed under a **Service Nervous System** (SNS), making Mastic a fully on-chain DAO governed by its community.

## DAO-Based Governance

We propose giving the DAO community governance for upgrades, feature proposals, federation decisions and moderation policies.

Token holders will be able to vote on proposals and allocate funds to the treasury.

Participants in the DAO will be able to apply for moderation and will be elected by the community.

## A Sustainable Funding Model

Premium social features, such as Pinned Posts, Profile customisations, verification, and NFT integrations, will be unlocked via staking or one-time contributions.

# Future Integrations

We plan to have future integrations with Mastic to achieve the **Infiniverse**, such as integrating **OpenChat** for direct messaging between users on an existing platform within the IC.

Additionally, we plan to integrate **Wallets** for sending tips and sponsoring creators on the platform, as well as to enable creators to publish *premium**&#32;***content.

In our vision for creating the **Infiniverse**, we aim to create spaces on Mastic where communities can interact with one another.

Finally, once multiple Mastic instances are launched onto the network, we can implement the **Infiniverse Protocol** to enable cases to communicate directly over the Internet Computer Protocol, rather than relying on an HTTP server.

# Market Analysis

## Total Addressable Market

Currently, Mastodon has 1.4 million active users per month.

Most of these are developers, journalists, artists and niche communities. In particular, Mastodon has experienced significant growth following Elon Musk's acquisition of Twitter.

Additionally, we have approximately 4 million crypto enthusiasts on [X.com](http://x.com), who may be interested in joining a web3 social platform and reducing their reliance on web2 services.

It would be ideal for the Internet Computer project to engage these users within its ecosystem by creating a Mastodon instance on the IC.

Of course, some users have left Twitter but have not joined Mastodon, either because they are using BlueSky or other platforms, such as Reddit or Discord groups. A captivating platform like Mastic, with the additional functionalities described in this document, could be a good hook for even more users.

## Serviceable Available Market

Above 10-15% of the TAM

Our first target is:

- Rust Developers: 2.3 million developers in 2024
- Web3 Developers: about 25k developers in 2024
- IC developers: \~3k
- Crypto Enthusiasts: 4 million users on X.com

## Serviceable Obtainable Market

A realistic target within the first year could be around 20,000 users, if we primarily focus on developers, crypto enthusiasts, and web3 developers.

## Differentiation from Traditional Mastodon Instances

Mastic introduces a fundamentally new paradigm for federated social platforms by reimagining the Mastodon experience as a fully on-chain, modular architecture built natively on the Internet Computer. While Mastodon has pioneered the Fediverse movement, its traditional deployment model — based on a monolithic Ruby on Rails stack supported by PostgreSQL, Redis, Sidekiq, and other infrastructure — imposes significant operational overhead and limits scalability.

By contrast, Mastic offers the following distinct advantages:

- Fully on-chain architecture: Every component of Mastic runs within canisters on the Internet Computer, eliminating the need for traditional DevOps, hosting, and external databases.
- Modularity and scalability: Each user operates within their User Canister, ensuring composability, privacy, and scalability by design.
- Web3-native features, including integrated Internet Identity login, on-chain wallets, tipping, premium content, and future NFT support, natively bridge the Fediverse with the Web3 ecosystem.
- Decentralised governance: Instead of relying on a central instance admin, Mastic is governed by a DAO through the Service Nervous System (SNS), enabling transparent, community-driven decisions around moderation, feature development, and treasury allocation.
- Developer-friendly technology stack: Mastic is built entirely in Rust, offering a modern, secure, and performant alternative that resonates with a growing community of Rust and Web3 developers.

These innovations make Mastic not only more accessible for developers and small communities but also more aligned with the philosophical foundations of the decentralised web.

## User Acquisition Strategy

Mastic’s adoption strategy is designed to activate key communities aligned with its technical foundation and decentralised vision, and to establish a sustainable user base through targeted outreach and integrations within the Web3 ecosystem.

Our multi-phase growth approach includes:

- [ ] Developer-first onboarding: We will bootstrap the platform by targeting Rust and Internet Computer developers, communities that already value decentralisation, composability, and performance. Early access, bounties, and contributor incentives will be designed to attract this group.
- [ ] Web3 community engagement: Mastic will engage the broader Web3 ecosystem, including creators, NFT artists, and DAO participants, who seek alternative social platforms for self-expression, monetisation, and identity ownership. This includes strategic outreach to the active Web3 community on X.com (formerly Twitter), which comprises several million users.
- [ ] Fediverse-native advocacy: As a fully compatible ActivityPub implementation, Mastic will be promoted within the Fediverse itself. We aim to attract privacy-conscious users and instance operators looking for a lower-maintenance, modern alternative to traditional Mastodon deployments.
- [ ] Incentivised adoption: Early users and contributors may receive governance tokens through an SNS airdrop, and creators will benefit from tools for tipping, sponsorships, and gated content. This will foster long-term alignment and participation in the platform’s growth.
- [ ] Partnerships and integrations: Collaborations with existing Internet Computer projects, such as OpenChat, will enable cross-platform interaction and shared user flows, further extending Mastic’s reach within the ecosystem.

Through this targeted strategy, Mastic aims to establish itself as the leading Web3-native gateway to the Fediverse, cultivating a resilient, decentralised, and self-sustaining social platform.

# Architecture Overview



The **architecture of Mastic** consists of 5 components, each of which will be a standalone canister.

- **Frontend**: The frontend of Mastic provides an interface where each user can sign in to Mastic and interact with the Fediverse by publishing Statuses and interacting with other users, including those from the Mastodon Fediverse on the web2. The User will be able to sign in with Internet Identity on the frontend and interact with the backend canisters using Candid.
- **User Canister**: Each user has one User Canister. The User canister is deployed every time a user signs up on Mastic and is deleted whenever they delete their account or migrate to another Fediverse instance. The User Canister provides the interface for users to interact with the Fediverse through the **Inbox** and **Outbox,** implementing the core functionality of the **ActivityPub** protocol.
- **Directory Canister**: The Directory Canister provides an index for all existing users on **Mastic** by creating a Map between a user’s identity and their handle (e.g., @veeso@mastic.social) and their User Canister instance.
- **Federation Canister**: The Federation canister implements the HTTP Web server to handle both incoming and outgoing requests of the **Federation Protocol**[[1](https://www.w3.org/TR/activitypub/#server-to-server-interactions)], which is used to communicate with the other instances on the **Fediverse**. The Federation canister MUST also implement the **WebFinger**[[2](https://docs.joinmastodon.org/spec/webfinger/)] protocol to search for users.
- **Orbit Station**: Mastic utilises the **Orbit Station**[[3](https://orbit.global/)] to deploy User Canisters and manage their permissions, upgrades, and cycles.

# Interface

Here’s a description of the Candid interface of the **Directory**, **Federation** and **User** canisters. Types are not yet defined, but only calls are provided to provide an overview of the flows that will need to be implemented. For more information on the flows, see the **User Stories**.

## Directory Interface

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

## Federation Interface

service : (*FederationInstallArgs*) -> {

 http\_request : (*HttpRequest*) -> (*HttpResponse*);

 send\_activity : (*SendActivityArgs*) -> (*SendActivityResponse*)

}

## User Interface

service : (*UserInstallArgs*) -> {

 boost\_status : (*BoostStatusArgs*) -> (*BoostStatusResponse*);

 delete\_profile : () -> (*DeleteProfileResponse*);

 delete\_status : (*DeleteStatusArgs*) -> (*DeleteStatusResponse*);

 like\_status : (*LikeStatusArgs*) -> (*LikeStatusResponse*);

 publish\_status : (*PublishStatusArgs*) -> (*PublishStatusResponse*);

 read\_feed : (*ReadFeedArgs*) -> (*ReadFeedResponse*) query;

 update\_profile : (*UpdateProfileArgs*) -> (*UpdateProfileResponse*)

}

# User Stories

## UC1: As a User, I should be able to create a Profile

- **Alice** lands on Mastic **Frontend**
- **Alice** signs in with her **Internet Identity**
- **Alice** sends a *sign\_up* call
- The Directory Canister establishes a connection between Alice’s identity and her handle.
- The **Directory Canister** starts a worker to create her **User Canister**
- The **Directory Canister** creates **Alice’s User Canister** on the **Orbit Station**
- The **Directory Canister** installs **Alice’s User Canister** onto the **Orbit Station**
- The **Directory Canister** stores **Alice’s User Canister&#32;**ID for **Alice**’s Identity
- **Alice** queries her **User Canister** ID with *user\_canister*
- The **Directory Canister** returns the ID to **Alice**

## UC2: As a User, I should be able to Sign In

- **Alice** lands on Mastic **Frontend**
- **Alice** signs in with her **Internet Identity**
- **Alice** queries the **Directory Canister** to get her user\_canister with *whoami*
- The **Directory Canister** returns her **User Canister**’s principal

## UC3: As a User, I should be able to update my Profile

## UC4: As a User, I should be able to delete my Profile

## UC5: As a User, I should be able to follow another Profile

## UC6: As a User, I should be able to remove a Following

## UC7: As a User, I should be able to see a user’s profile

## UC8: As a User, I should be able to search for other users

## UC9: As a User, I should be able to create a Status

## UC10: As a User, I should be able to like a Status

## UC11: As a User, I should be able to boost a Status

## UC12: As a User, I should be able to read my Feed

## UC13: As a User, I should be able to receive updates from users I follow on other Fediverse instances

## UC14: As a User on a Web2 Mastodon Instance, I should be able to receive updates and interact with users on Mastic

## UC15: As a Moderator, I should be able to remove a Status violating the policy

## UC16: As a Moderator, I should be able to suspend a Profile violating the policy

# Milestones

These are the milestones we plan to achieve during the first year of Mastic's development cycle.

## Milestone 0 - Proof of concept

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

## Milestone 1 - Standalone Mastic Node

Duration: 3 months

Mastic is set up; we build all the data structures required by the user to sign up, operate on statuses (Create, Delete, like, and Boost), and find and follow other users on the Mastic node.

This Milestone won’t include the integration with the Fediverse yet, but it will provide an already usable Social Network, ready to be integrated onto the Fediverse soon.
 
These stories must be implemented during this phase:

- UC3
- UC4
- UC6
- UC8
- UC10
- UC11
- UC12
- UC15
- UC16

## Milestone 2 - Integrating the Fediverse

Duration: 2 months

This step requires implementing the **Fediverse Canister** using the **Fediverse Protocol**.

This will allow Mastic to be fully integrated with the Fediverse ecosystem.

These user stories must be implemented during this phase:

- UC13
- UC14

## Milestone 3 - SNS Launch

Duration: 1 month

During this phase, we plan to launch Mastic on the SNS to add a fully decentralised governance to Mastic.

We need to implement a comprehensive integration for voting, specifically for adding and removing moderators, updating policies, and integrating premium features.

## Milestone 4 - Bringing in the IC Ecosystem

Duration: 4 months

In this final phase of the project, we plan to integrate other applications of the **Internet Computer** ecosystem into **Mastic**. The details will be planned later on, but some nice features to have would be:

- Integrating **Open Chat**[[4](https://oc.app/)] for direct communication with other users on the **Infiniverse**
- Integrating **Wallets** for sponsoring content creators

# Reference

1. ActivityPub: [https://www.w3.org/TR/activitypub/](https://www.w3.org/TR/activitypub/) 
2. ActivityStreams: [https://www.w3.org/TR/activitystreams-core/](https://www.w3.org/TR/activitystreams-core/) 
3. Mastodon ActivityPub Spec: [https://docs.joinmastodon.org/spec/activitypub/](https://docs.joinmastodon.org/spec/activitypub/)
4. ActivityPub Federation framework implemented with Rust: [https://docs.rs/activitypub\_federation/0.6.5/activitypub\_federation/](https://docs.rs/activitypub_federation/0.6.5/activitypub_federation/)
5. Webfinger: [https://docs.joinmastodon.org/spec/webfinger/](https://docs.joinmastodon.org/spec/webfinger/) 
6. Orbit: [https://orbit.global/](https://orbit.global/) 
