---
title: "ActivityPub"
layout: page
---

# ActivityPub on Mastic

- [ActivityPub on Mastic](#activitypub-on-mastic)
  - [Mapping to Mastic Architecture](#mapping-to-mastic-architecture)
    - [Federation Canister HTTP Endpoints](#federation-canister-http-endpoints)
  - [Objects](#objects)
    - [Retrieving objects](#retrieving-objects)
    - [Source](#source)
  - [Actors](#actors)
    - [Inbox and Outbox](#inbox-and-outbox)
  - [Activity Streams](#activity-streams)
    - [Public collections](#public-collections)
  - [Protocol](#protocol)
  - [Social API](#social-api)
    - [Client Addressing](#client-addressing)
    - [Create Activity](#create-activity)
    - [Update Activity](#update-activity)
    - [Delete Activity](#delete-activity)
    - [Follow Activity](#follow-activity)
    - [Add Activity](#add-activity)
    - [Remove Activity](#remove-activity)
    - [Like Activity](#like-activity)
    - [Block Activity](#block-activity)
    - [Undo Activity](#undo-activity)
    - [Delivery](#delivery)
  - [Federation Protocol](#federation-protocol)
    - [Server Side Activities](#server-side-activities)
  - [Mastodon](#mastodon)
    - [Statuses Federation](#statuses-federation)
      - [Payloads](#payloads)
      - [HTML Sanitization](#html-sanitization)
      - [Status Properties](#status-properties)
      - [Poll specific properties](#poll-specific-properties)
    - [Profiles Federation](#profiles-federation)
      - [Profile Properties](#profile-properties)
    - [Reports Extension](#reports-extension)
    - [Sensitive Extension](#sensitive-extension)
    - [Hashtag](#hashtag)
    - [Custom Emoji](#custom-emoji)
    - [Focal Points](#focal-points)
    - [Quote Posts](#quote-posts)
    - [Discoverability Flag](#discoverability-flag)
    - [Indexable Flag](#indexable-flag)
    - [Suspended Flag](#suspended-flag)
    - [Memorial Flag](#memorial-flag)
    - [Polls](#polls)
    - [Mentions](#mentions)
    - [Public Key](#public-key)
    - [Blurhash](#blurhash)
    - [Featured Collection](#featured-collection)
    - [Featured Tags](#featured-tags)
    - [Profile Metadata](#profile-metadata)
    - [Account Migration](#account-migration)
    - [Remote Blocking](#remote-blocking)
  - [HTTP Signatures](#http-signatures)
    - [Signing POST requests](#signing-post-requests)
    - [Verifying Signatures](#verifying-signatures)
  - [WebFinger](#webfinger)
    - [WebFinger Simple Flow](#webfinger-simple-flow)

This module provides a technical overview with simple diagrams of the ActivityPub protocol, which is used for federated social networking.

The diagrams illustrate the flow of activities between actors in a federated network, showing how they interact with each other through various endpoints, and so what it has to be implemented in order to support **ActivityPub on Mastic**.

## Mapping to Mastic Architecture

The following table shows how core ActivityPub concepts map to Mastic's canister-based architecture:

| ActivityPub Concept | Mastic Component | Notes |
|---|---|---|
| **Actor** | **User Canister** | Each Mastic user is represented by a dedicated User Canister that acts as their ActivityPub Actor. |
| **Inbox / Outbox** | **User Canister** | The actor's inbox and outbox collections are stored in the User Canister. They are exposed to the Fediverse via HTTP endpoints served by the Federation Canister. |
| **Social API (C2S)** | **Candid calls to User Canister** | Instead of HTTP-based Client-to-Server interactions, Mastic users interact with their User Canister through authenticated Candid calls, using Internet Identity for authentication. |
| **Federation Protocol (S2S)** | **Federation Canister** | All Server-to-Server HTTP traffic is handled by the Federation Canister, which receives incoming activities and forwards outgoing activities to remote instances. |
| **HTTP Signatures** | **User Canister (key storage) + Federation Canister (signing/verification)** | Each User Canister generates and stores an RSA key pair at creation time. The Federation Canister uses the private key to sign outgoing requests and serves the public key when the actor profile is requested. |
| **WebFinger** | **Federation Canister** | WebFinger lookups (`/.well-known/webfinger`) are handled by the Federation Canister's `http_request` query method, which resolves account handles to actor URIs via the Directory Canister. |

### Federation Canister HTTP Endpoints

The Federation Canister serves the following HTTP routes to enable ActivityPub federation and discovery:

| Method | Route | Description |
|--------|-------|-------------|
| `GET` | `/.well-known/webfinger` | WebFinger lookup — resolves `acct:` URIs to actor profiles via the Directory Canister |
| `GET` | `/users/{handle}` | Actor profile — returns the JSON-LD representation of the actor |
| `GET` | `/users/{handle}/inbox` | Actor inbox — returns the inbox as an `OrderedCollection` |
| `POST` | `/users/{handle}/inbox` | Receive activities from remote instances (S2S) — validates HTTP Signatures and delivers to the User Canister |
| `GET` | `/users/{handle}/outbox` | Actor outbox — returns the outbox as an `OrderedCollection` |
| `GET` | `/users/{handle}/followers` | Followers collection — returns the actor's followers as an `OrderedCollection` |
| `GET` | `/users/{handle}/following` | Following collection — returns the actors followed by this actor as an `OrderedCollection` |
| `GET` | `/users/{handle}/liked` | Liked collection — returns the activities liked by this actor as an `OrderedCollection` |

All `GET` endpoints are served by the Federation Canister's `http_request` query method. The `POST /users/{handle}/inbox` endpoint is handled by `http_request_update` since it requires state changes (delivering activities to User Canisters).

## Objects

All objects in ActivityPub are represented as JSON-LD documents. The objects can be of various types, such as `Person`, `Note`, `Create`, `Like`, etc.

Each object MUST have:

- `id`: The object's unique global identifier (unless the object is transient, in which case the id MAY be omitted).
- `type`: The type of the object.

and can have various properties such as `to`, `actor`, and `content`.

### Retrieving objects

Servers MUST present the ActivityStreams object representation in response to `application/ld+json; profile="https://www.w3.org/ns/activitystreams"`, and SHOULD also present the ActivityStreams representation in response to `application/activity+json` as well.

The client MUST specify an Accept header with the `application/ld+json; profile="https://www.w3.org/ns/activitystreams"` media type in order to retrieve the activity.

### Source

The Object also contains the source attribute, which has been originally used to derive the `content`:

```json
{
  "content": "<p>I <em>really</em> like strawberries!</p>",
  "source": {
    "content": "I *really* like strawberries!",
    "mediaType": "text/markdown"
    }
}
```

## Actors

Actors are the entities that perform actions in the ActivityPub protocol. They can be users, applications, or services. Each actor has a unique identifier and can have various properties such as name, icon, and preferred language.

Actors are represented as JSON-LD documents with the `type` set to `Person`, `Application`, or other types defined in the ActivityStreams vocabulary.

Each actor MUST, in addition to the properties for the [Objects](#objects), have the following properties:

- `inbox`: (`OrderedCollection`) The URL of the actor's inbox, where it receives activities.
- `outbox`: (`OrderedCollection`) The URL of the actor's outbox, where it sends activities.
- `following`: (`OrderedCollection`) An Url to an [ActivityStreams](#activity-streams) collection that contains the actors that this actor is following.
- `followers`: (`OrderedCollection`) An Url to an [ActivityStreams](#activity-streams) collection that contains the actors that are following this actor.
- `liked`: (`OrderedCollection`) An Url to an [ActivityStreams](#activity-streams) collection that contains the activities that this actor has liked.

```json
{
  "@context": ["https://www.w3.org/ns/activitystreams", { "@language": "ja" }],
  "type": "Person",
  "id": "https://kenzoishii.example.com/",
  "following": "https://kenzoishii.example.com/following.json",
  "followers": "https://kenzoishii.example.com/followers.json",
  "liked": "https://kenzoishii.example.com/liked.json",
  "inbox": "https://kenzoishii.example.com/inbox.json",
  "outbox": "https://kenzoishii.example.com/feed.json",
  "preferredUsername": "kenzoishii",
  "name": "石井健蔵",
  "summary": "この方はただの例です",
  "icon": ["https://kenzoishii.example.com/image/165987aklre4"]
}
```

### Inbox and Outbox

Every actor has both an inbox and an outbox. The inbox is where the actor receives activities from other actors, while the outbox is where the actor sends activities to other actors.

From an implementation perspective, both the Inbox and the Outbox, are `OrderedCollection` objects.

```mermaid
block-beta
    columns 7

    U (["User"])
    space:2

    block:queues
    columns 1
    IN ("Inbox")
    OUT ("Outbox")
    end

    space:2

    RO ("Rest of the world")

    U -- "POST messages to" --> OUT
    IN -- "GET messages from" --> U
    RO -- "POST messages to" --> IN
    OUT -- "GET messages from" --> RO
```

Actor data:

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Person",
  "id": "https://social.example/alice/",
  "name": "alice P. Hacker",
  "preferredUsername": "alice",
  "summary": "Lisp enthusiast hailing from MIT",
  "inbox": "https://social.example/alice/inbox/",
  "outbox": "https://social.example/alice/outbox/",
  "followers": "https://social.example/alice/followers/",
  "following": "https://social.example/alice/following/",
  "liked": "https://social.example/alice/liked/"
}
```

Now let's say Alice wants to send a message to Bob. The following diagram illustrates the flow of this activity:

```mermaid
block-beta
    columns 13

    A (["Alice"])
    space

    block:aliq
        columns 1
        AIN ("Inbox")
        AOUT ("Outbox")
    end

    space

    AS ("Alice's Server")

    space:2

    BS ("Bob's Server")

    space

    block:bobq
        columns 1
        BIN ("Inbox")
        BOUT ("Outbox")
    end

    space:2

    B ("Bob")

    A -- "POST message" --> AOUT
    AOUT --> AS
    AS -- "POST message to" --> BS
    BS --> BIN
    BIN -- "GET message" --> B
```

First Alice sends a message to her outbox:

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Note",
  "to": ["https://chatty.example/ben/"],
  "attributedTo": "https://social.example/alice/",
  "content": "Say, did you finish reading that book I lent you?"
}
```

Then Alice's server creates the post and forwards the message to Bob's server:

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Create",
  "id": "https://social.example/alice/posts/a29a6843-9feb-4c74-a7f7-081b9c9201d3",
  "to": ["https://chatty.example/ben/"],
  "actor": "https://social.example/alice/",
  "object": {
    "type": "Note",
    "id": "https://social.example/alice/posts/49e2d03d-b53a-4c4c-a95c-94a6abf45a19",
    "attributedTo": "https://social.example/alice/",
    "to": ["https://chatty.example/ben/"],
    "content": "Say, did you finish reading that book I lent you?"
  }
}
```

Later after Bob has answered, Alice can fetch her inbox with a GET and see the answer to that message:

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Create",
  "id": "https://chatty.example/ben/p/51086",
  "to": ["https://social.example/alice/"],
  "actor": "https://chatty.example/ben/",
  "object": {
    "type": "Note",
    "id": "https://chatty.example/ben/p/51085",
    "attributedTo": "https://chatty.example/ben/",
    "to": ["https://social.example/alice/"],
    "inReplyTo": "https://social.example/alice/posts/49e2d03d-b53a-4c4c-a95c-94a6abf45a19",
    "content": "<p>Argh, yeah, sorry, I'll get it back to you tomorrow.</p><p>I was reviewing the section on register machines,since it's been a while since I wrote one.</p>"
  }
}
```

Further interactions can be made, such as liking the reply:

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Like",
  "id": "https://social.example/alice/posts/5312e10e-5110-42e5-a09b-934882b3ecec",
  "to": ["https://chatty.example/ben/"],
  "actor": "https://social.example/alice/",
  "object": "https://chatty.example/ben/p/51086"
}
```

And this will follow the same flow as before.

## Activity Streams

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "id": "https://www.w3.org/ns/activitystreams",
  "type": "Collection"
}
```

ActivityStreams defines the collection concept; ActivityPub defines several collections with special behavior. Note that ActivityPub makes use of ActivityStreams paging to traverse large sets of objects.

> Note that some of these collections are specified to be of type `OrderedCollection` specifically, while others are permitted to be either a `Collection` or an `OrderedCollection`. An `OrderedCollection` MUST be presented consistently in reverse chronological order.

### Public collections

Some collections are marked as `Public`

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "id": "https://www.w3.org/ns/activitystreams#Public",
  "type": "Collection"
}
```

And MUST be accessible to anyone, regardless of whether they are authenticated or not.

## Protocol

The protocol is based on HTTP and uses JSON-LD for data representation.

Two APIs are defined:

- **Social API**: It's a client-to-server API that allows clients to interact with the server, such as creating posts, following users, and liking content.
- **Federation Protocol**: It's a server-to-server API that allows servers to exchange activities with each other, such as sending posts, following users, and liking content.

Mastic is implemented as a **ActivityPub conformant Federated Server**, with a significant variation.

While the Federation Protocol is implemented with HTTP, the Social API is implemented using the Internet Computer's native capabilities, such as `update` calls and `query` calls, and calls are so authenticated using the [Internet Computer's Internet Identity](https://internetcomputer.org/internet-identity).

## Social API

In the standard ActivityPub specification, client-to-server (C2S) interaction takes place through clients posting `Activities` to an actor's outbox via HTTP POST requests. Mastic replaces this HTTP-based C2S layer with **typed Candid methods** on the User Canister.

Instead of discovering an outbox URL and POSTing JSON-LD payloads to it, Mastic users call specific Candid methods on their User Canister, such as `publish_status`, `like_status`, `follow_user`, `boost_status`, `block_user`, etc. Each method accepts a typed Candid argument struct and returns a typed response. The User Canister internally manages the actor's outbox, appending the corresponding ActivityPub activity for each operation.

Requests MUST be authenticated using the Internet Computer's Internet Identity — the caller's principal must match the owner principal configured at canister install time.

The User Canister handles the same side effects that the ActivityPub spec requires for outbox operations:

- The server (User Canister) MUST generate a new `id` for each Activity.
- The server MUST remove `bto` and/or `bcc` properties before delivery, but MUST utilize the addressing originally stored on these properties for determining recipients.
- The server MUST add the new Activity to the outbox collection.
- Depending on the type of Activity, the server may carry out further side effects as described per individual Activity below.

### Client Addressing

Clients are responsible for addressing new Activities appropriately. To some extent, this is dependent upon the particular client implementation, but clients must be aware that the server will only forward new Activities to addressees in the `to`, `bto`, `cc`, `bcc`, and `audience` fields.

### Create Activity

The `Create` activity is used when posting a new object. This has the side effect that the object embedded within the Activity (in the object property) is created.

When a `Create` activity is posted, the actor of the activity SHOULD be copied onto the object's `attributedTo` field.

For client to server posting, it is possible to submit an object for creation **without a surrounding activity**. The server MUST accept a valid [ActivityStreams](#activity-streams) object that isn't a subtype of Activity in the POST request to the outbox. The server then MUST attach this object as the object of a Create Activity. For non-transient objects, the server MUST attach an id to both the wrapping Create and its wrapped Object.

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Note",
  "content": "This is a note",
  "published": "2015-02-10T15:04:55Z",
  "to": ["https://example.org/~john/"],
  "cc": ["https://example.com/~erik/followers",
         "https://www.w3.org/ns/activitystreams#Public"]
}
```

Is equivalent to this and both MUST be accepted by the server:

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Create",
  "id": "https://example.net/~mallory/87374",
  "actor": "https://example.net/~mallory",
  "object": {
    "id": "https://example.com/~mallory/note/72",
    "type": "Note",
    "attributedTo": "https://example.net/~mallory",
    "content": "This is a note",
    "published": "2015-02-10T15:04:55Z",
    "to": ["https://example.org/~john/"],
    "cc": ["https://example.com/~erik/followers",
           "https://www.w3.org/ns/activitystreams#Public"]
  },
  "published": "2015-02-10T15:04:55Z",
  "to": ["https://example.org/~john/"],
  "cc": ["https://example.com/~erik/followers",
         "https://www.w3.org/ns/activitystreams#Public"]
}
```

### Update Activity

The Update activity is used when updating an already existing object. The side effect of this is that the object MUST be modified to reflect the new structure as defined in the update activity, assuming the actor has permission to update this object.

Usually updates are partial.

### Delete Activity

The Delete activity is used to delete an already existing object

The side effect of this is that the server MAY replace the object with a Tombstone of the object that will be displayed in activities which reference the deleted object. If the deleted object is requested the server SHOULD respond with either the `Gone` status code if a Tombstone object is presented as the response body, otherwise respond with a `NotFound`.

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "id": "https://example.com/~alice/note/72",
  "type": "Tombstone",
  "published": "2015-02-10T15:04:55Z",
  "updated": "2015-02-10T15:04:55Z",
  "deleted": "2015-02-10T15:04:55Z"
}
```

### Follow Activity

The Follow activity is used to subscribe to the activities of another actor.

### Add Activity

Upon receipt of an `Add` activity into the outbox, the server SHOULD add the object to the collection specified in the target property, unless:

- the `target` is not owned by the receiving server, and thus they are not authorized to update it.
- the `object` is not allowed to be added to the `target` collection for some other reason, at the receiving server's discretion.

### Remove Activity

Upon receipt of a Remove activity into the outbox, the server SHOULD remove the object from the collection specified in the target property, unless:

- the `target` is not owned by the receiving server, and thus they are not authorized to update it.
- the `object` is not allowed to be removed from the `target` collection for some other reason, at the receiving server's discretion.

### Like Activity

The Like activity indicates the actor likes the object.

The side effect of receiving this in an outbox is that the server SHOULD add the object to the actor's liked Collection.

### Block Activity

The Block activity is used to indicate that the posting actor does not want another actor (defined in the object property) to be able to interact with objects posted by the actor posting the Block activity. The server SHOULD prevent the blocked user from interacting with any object posted by the actor.

Servers SHOULD NOT deliver Block Activities to their object.

### Undo Activity

The Undo activity is used to undo a previous activity. See the Activity Vocabulary documentation on Inverse Activities and "Undo". For example, Undo may be used to undo a previous Like, Follow, or Block. The undo activity and the activity being undone MUST both have the same actor. Side effects should be undone, to the extent possible. For example, if undoing a Like, any counter that had been incremented previously should be decremented appropriately.

There are some exceptions where there is an existing and explicit "inverse activity" which should be used instead. Create based activities should instead use Delete, and Add activities should use Remove.

### Delivery

Federated servers MUST perform delivery on all Activities posted to the outbox according to outbox delivery.

## Federation Protocol

Servers communicate with other servers and propagate information across the social graph by posting activities to actors' inbox endpoints. An **Activity** sent over the network SHOULD have an `id`, unless it is intended to be transient (in which case it MAY omit the id).

POST requests (eg. to the inbox) MUST be made with a `Content-Type` of `application/ld+json; profile="https://www.w3.org/ns/activitystreams"` and GET requests (see also 3.2 Retrieving objects) with an `Accept header` of `application/ld+json; profile="https://www.w3.org/ns/activitystreams"`.

Servers SHOULD interpret a `Content-Type` or `Accept header` of `application/activity+json as equivalent to application/ld+json; profile="https://www.w3.org/ns/activitystreams`" for server-to-server interactions.

In order to propagate updates throughout the social graph, Activities are sent to the appropriate recipients. First, these recipients are determined through following the appropriate links between objects until you reach an actor, and then the Activity is inserted into the actor's inbox (delivery). This allows recipient servers to:

1. conduct any side effects related to the Activity (for example, notification that an actor has liked an object is used to update the object's like count);
2. deliver the Activity to recipients of the original object, to ensure updates are propagated to the whole social graph (see inbox delivery).

Delivery is usually triggered by, for example:

- an Activity being created in an actor's outbox with their Followers Collection as the recipient.
- an Activity being created in an actor's outbox with directly addressed recipients.
- an Activity being created in an actors's outbox with user-curated collections as recipients.
- an Activity being created in an actor's outbox or inbox which references another object.

Servers performing delivery to the `inbox` or `sharedInbox` properties of actors on other servers MUST provide the object property in the activity: `Create`, `Update`, `Delete`, `Follow`, `Add`, `Remove`, `Like`, `Block`, `Undo`. Additionally, servers performing server to server delivery of the following activities MUST also provide the `target` property: `Add`, `Remove`.

An activity is delivered to its targets (which are actors) by first looking up the targets' inboxes and then posting the activity to those inboxes. Targets for delivery are determined by checking the ActivityStreams audience targeting; namely, the `to`, `bto`, `cc`, `bcc`, and `audience` fields of the activity.

### Server Side Activities

Just follow 1:1 the document described here:

<https://www.w3.org/TR/activitypub/#create-activity-inbox>

## Mastodon

> **Note:** Not all Mastodon extensions described in this section will be implemented in the initial milestones. Features such as pinned posts (Featured Collection), Flag/reporting, and Move/account migration are documented here for completeness and future reference. See the milestone plan in [`docs/project.md`](../project.md#milestones) for the implementation timeline.

### Statuses Federation

In Mastodon statuses are posts, aka _toots_, of the type of `Notes` of the ActivityPub protocol.

Mastodon supports the following activities for `Statuses`:

- `Create`: Transformed into a status and saved into database
- `Delete`: Delete a Status from the database
- `Like`: Favourited a Status
- `Announce`: Boost a status (like rt on Twitter)
- `Undo`: Undo a Like or a Boost
- `Flag`: Transformed into a report to the moderation team. See the [Reports extension](#reports-extension) for more information
- `QuoteRequest`: Request approval for a quote post. See the [Quote Posts extension](#quote-posts)

#### Payloads

The first-class Object types supported by Mastodon are `Note` and `Question`.

- `Notes` are transformed into regular statuses.
- `Questions` are transformed into a poll status. See the [Polls](#polls) extension for more information.

#### HTML Sanitization

<https://docs.joinmastodon.org/spec/activitypub/#sanitization>.

#### Status Properties

These are the properties used:

- `content`: status text content
- `name`: Used as status text, if content is not provided on a transformed Object type
- `summary`: Used as CW (Content warning) text
- `sensitive`: Used to determine whether status media or text should be hidden by default. See the [Sensitive content extension](#sensitive-extension) section for more information about as:sensitive
- `inReplyTo`: Used for threading a status as a reply to another status
- `published`: status published date
- `url`: status permalink
- `attributedTo`: Used to determine the profile which authored the status
- `to/cc`: Used to determine audience and visibility of a status, in combination with mentions. See [Mentions](#mentions) for adddressing and notifications.
- `tag`: Used to mark up mentions and hashtags.
  - `type`: Either Mention, Hashtag, or Emoji is currently supported. See the [Hashtag](#hashtag) and [Custom emoji extension](#custom-emoji) sections for more information.
  - `name`: The plain-text Webfinger address of a profile Mention (`@user` or `@user@domain`), or the plain-text Hashtag (#tag), or the custom Emoji shortcode (`:thounking:`)
  - `href`: The URL of the actor or tag
- `attachment`: Used to include attached images, videos, or audio
  - `url`: Used to fetch the media attachment
  - `summary`: Used as media description `alt`
  - `blurhash`: Used to generate a blurred preview image corresponding to the colors used within the image. See [Blurhash](#blurhash) for more details
- `replies`: A Collection of `statuses` that are in reply to the current status. Up to 5 replies from the same server will be fetched upon discovery of a remote status, in order to resolve threads more fully. On Mastodon’s side, the first page contains self-replies, and additional pages contain replies from other people.
- `likes`: A Collection used to represent Like activities received for this status. The actual activities are not exposed by Mastodon at this time.
  - `totalItems`: The number of likes this status has received
- `shares`: A Collection used to represent Announce activities received for this status. The actual activities are not exposed by Mastodon at this time.
  - `totalItems`: The number of Announce activities received for this status.

#### Poll specific properties

- `endTime`: The timestamp for when voting will close on the poll
- `closed`: The timestamp for when voting closed on the poll. The timestamp will likely match the endTime timestamp. If this property is present, the poll is assumed to be closed.
- `votersCount`: How many people have voted in the poll. Distinct from how many votes have been cast (in the case of multiple-choice polls)
- `oneOf`: Single-choice poll options
  - `name`: The poll option’s text
  - `replies`:
    - `totalItems`: The poll option’s vote count
- `anyOf`: Multiple-choice poll options
  - `name`: The poll option’s text
  - `replies`:
    - `totalItems`: The poll option’s vote count

### Profiles Federation

Profiles are represented as `Person` objects in ActivityPub, and they are used to represent users on the platform. Mastodon supports the following activities for profiles:

- `Follow`: Indicate interest in receiving status updates from a profile.
- `Accept/Reject`: Used to approve or deny Follow activities. Unlocked accounts will automatically reply with an Accept, while locked accounts can manually choose whether to approve or deny a follow request.
- `Add/Remove`: Manage pinned posts and featured collections.
- `Update`: Refresh account details
- `Delete`: Remove an account from the database, as well as all of their statuses.
- `Undo`: Undo a previous Follow, Accept Follow, or Block.
- `Block`: Signal to a remote server that they should hide your profile from that user. Not guaranteed.
- `Flag`: Report a user to their moderation team. See the [Reports extension](#reports-extension) for more information
- `Move`: Migrate followers from one account to another. Requires `alsoKnownAs` to be set on the new account pointing to the old account

#### Profile Properties

- `preferredUsername`: Used for Webfinger lookup. Must be unique on the domain, and must correspond to a Webfinger `acct:` URI.
- `name`: Used as profile display name.
- `summary`: Used as profile bio.
- `type`: Assumed to be `Person`. If type is `Application` or `Service`, it will be interpreted as a bot flag.
- `url`: Used as profile link.
- `icon`: Used as profile avatar.
- `image`: Used as profile header.
- `manuallyApprovesFollowers`: Will be shown as a locked account.
- `discoverable`: Will be shown in the profile directory.
- `indexable`: Posts by this account can be indexed for full-text search
- `publicKey`: Required for signatures. See [Public Key](#public-key) for more information.
- `featured`: Pinned posts. See [Featured collection](#featured-collection)
- `attachment`: Used for profile fields. See [Profile metadata](#profile-metadata).
- `alsoKnownAs`: Required for `Move` activity
- `published`: When the profile was created.
- `memorial`: Whether the account is a memorial account. See [Memorial Flag](#memorial-flag) for more information.
- `suspended`: Whether the account is currently suspended. See [Suspended Flag](#suspended-flag) for more information.
- `attributionDomains`: Domains allowed to use `fediverse:creator` for this actor in published articles.

### Reports Extension

To report profiles and/or posts on remote servers, Mastodon will send a `Flag` activity **from the instance actor**. The **object** of this activity contains the **user being reported**, as well as any posts attached to the report. If a comment is attached to the report, it will be used as the content of the activity.

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "id": "https://mastodon.example/ccb4f39a-506a-490e-9a8c-71831c7713a4",
  "type": "Flag",
  "actor": "https://mastodon.example/actor",
  "content": "Please take a look at this user and their posts",
  "object": [
    "https://example.com/users/1",
    "https://example.com/posts/380590",
    "https://example.com/posts/380591"
  ],
  "to": "https://example.com/users/1"
}
```

### Sensitive Extension

Mastodon uses the `as:sensitive` extension property to mark certain posts as sensitive. When a post is marked as sensitive, any media attached to it will be hidden by default, and if a summary is present, the status content will be collapsed behind this summary. In Mastodon, this is known as a content warning.

### Hashtag

Similar to the `Mention` subtype of `Link` already defined in `ActivityStreams`, Mastodon will use `Hashtag` as a subtype of Link in order to surface posts referencing some common topic identified by a string key. The Hashtag has a name containing the `#hashtag` microsyntax – a `#` followed by a string sequence representing a topic. This is similar to the `@mention` microsyntax, where an `@` is followed by some string sequence representing a resource (where in Mastodon’s case, this resource is expected to be an account). **Mastodon will also normalize hashtags to be case-insensitive** lowercase strings, performing ASCII folding and removing invalid characters.

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "Hashtag": "https://www.w3.org/ns/activitystreams#Hashtag"
    }
  ],
  "id": "https://example.com/some-post",
  "type": "Note",
  "attributedTo": "https://example.com",
  "content": "I love #cats",
  "tag": [
    {
      "type": "Hashtag",
      "name": "#cats",
      "href": "https://example.com/tagged/cats"
    }
  ]
}
```

### Custom Emoji

Mastodon supports arbitrary emojis by including a tag of the Emoji type. Handling of custom emojis is similar to handling of mentions and hashtags, where the name of the tagged entity is found as a substring of the natural language properties (name, summary, content) and then linked to the local representation of some resource or topic. In the case of emoji shortcodes, the name is replaced by the HTML for an inline image represented by the icon property (where icon.url links to the image resource).

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "Emoji": "http://joinmastodon.org/ns#Emoji",
    }
  ],

  "id": "https://example.com/@alice/hello-world",
  "type": "Note",
  "content": "Hello world :kappa:",
  "tag": [
    {
      "id": "https://example.com/emoji/123",
      "type": "Emoji",
      "name": ":kappa:",
      "icon": {
        "type": "Image",
        "mediaType": "image/png",
        "url": "https://example.com/files/kappa.png"
      }
    }
  ]
}
```

### Focal Points

Mastodon supports setting a focal point on uploaded images, so that wherever that image is displayed, the focal point stays in view. This is implemented using an extra property focalPoint on Image objects. The property is an array of two floating points between `-1.0` and `1.0`, with `0,0` being the center of the image, the first value being `x` (`-1.0` is the left edge, `+1.0` is the right edge) and the second value being `y` (`-1.0` is the bottom edge, `+1.0` is the top edge).

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "focalPoint": {
        "@container": "@list",
        "@id": "http://joinmastodon.org/ns#focalPoint"
      }
    }
  ],

  "id": "https://example.com/@alice/hello-world",
  "type": "Note",
  "content": "A picture attached!",
  "attachment": [
    {
      "type": "Image",
      "mediaType": "image/png",
      "url": "https://example.com/files/cats.png",
      "focalPoint": [
        -0.55,
        0.43
      ]
    }
  ]
}
```

### Quote Posts

Mastodon implements experimental support for handling remote quote posts according to `FEP-044f`. Additionally, it understands `quoteUri`, `quoteUrl` and `_misskey_quote` for compatibility.

Should a post contain multiple quotes, Mastodon only accepts the first one.

Furthermore, Mastodon does not handle the full range of interaction policies, but instead converts the authorized followers to a combination of “public”, “followers” and “unknown”, defaulting to “nobody”.

At this time, Mastodon does not offer authoring quotes, nor does it expose a quote policy, or produce stamps for incoming quote requests.

### Discoverability Flag

Mastodon allows users to opt-in or opt-out of discoverability features like the profile directory. This flag may also be used as an indicator of the user’s preferences toward being included in external discovery services. If you are implementing such a tool, it is recommended that you respect this property if it is present. This is implemented using an extra property discoverable on objects mapping to profiles.

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "discoverable": "http://joinmastodon.org/ns#discoverable"
    }
  ],
  "id": "https://mastodon.social/users/Gargron",
  "type": "Person",
  "discoverable": true
}
```

### Indexable Flag

Mastodon allows users to opt-in or opt-out of indexing features like full-text search of public statuses. If you are implementing such a tool, it is recommended that you respect this property if it is present. This is implemented using an extra property indexable on objects mapping to profiles.

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "indexable": "http://joinmastodon.org/ns#indexable"
    }
  ],
  "id": "https://mastodon.social/users/Gargron",
  "type": "Person",
  "indexable": true
}
```

### Suspended Flag

Mastodon reports whether a user was locally suspended, for better handling of these accounts. **Suspended accounts in Mastodon return empty data**. If a remote account is marked as suspended, it cannot be unsuspended locally. Suspended accounts can be targeted by activities such as `Update`, `Undo`, `Reject`, and `Delete`. This functionality is implemented using an extra property `suspended` on objects.

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "suspended": "http://joinmastodon.org/ns#suspended"
    }
  ],
  "id": "https://example.com/@eve",
  "type": "Person",
  "suspended": true
}
```

### Memorial Flag

Mastodon reports whether a user’s profile was memorialized, for better handling of these accounts. **Memorial accounts in Mastodon return normal data**, but are rendered with a header indicating that the account is a memorial account. This functionality is implemented using an extra property `memorial` on objects.

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "memorial": "http://joinmastodon.org/ns#memorial"
    }
  ],
  "id": "https://example.com/@alice",
  "type": "Person",
  "memorial": true
}
```

### Polls

The ActivityStreams Vocabulary specification describes loosely (non-normatively) how a question might be represented. Mastodon’s implementation of polls is somewhat inspired by this section. The following implementation details can be observed:

Question is used as an `Object` type instead of as an `IntransitiveActivity`; rather than being sent directly, it is wrapped in a `Create` just like any other status.

`Poll` options are serialized using `oneOf` or `anyOf` as an array.

Each item in this array has no id, has a type of `Note`, and has a name representing the text of the poll option.

Each item in this array also has a replies property, representing the responses to this particular poll option. This node has no id, has a type of Collection, and has a totalItems property representing the total number of votes received for this option.

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "votersCount": "http://joinmastodon.org/ns#votersCount"
    }
  ],
  "id": "https://mastodon.example/users/alice/statuses/1009947848598745",
  "type": "Question",
  "content": "What should I eat for breakfast today?",
  "published": "2023-03-05T07:40:13Z",
  "endTime": "2023-03-06T07:40:13Z",
  "votersCount": 7,
  "anyOf": [
    {
      "type": "Note",
      "name": "apple",
      "replies": {
        "type": "Collection",
        "totalItems": 3
      }
    },
    {
      "type": "Note",
      "name": "orange",
      "replies": {
        "type": "Collection",
        "totalItems": 7
      }
    },
    {
      "type": "Note",
      "name": "banana",
      "replies": {
        "type": "Collection",
        "totalItems": 6
      }
    }
  ]
}
```

Poll votes are serialized as `Create` activities, where the object is a Note with a name that exactly matches the name of the poll option. The Note.inReplyTo points to the URI of the Question object.

For multiple-choice polls, multiple activities may be sent. Votes will be counted if you have not previously voted for that option.

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "id": "https://mastodon.example/users/bob#votes/827163/activity",
  "to": "https://mastodon.example/users/alice",
  "actor": "https://mastodon.example/users/bob",
  "type": "Create",
  "object": {
    "id": "https://mastodon.example/users/bob#votes/827163",
    "type": "Note",
    "name": "orange",
    "attributedTo": "https://mastodon.example/users/bob",
    "to": "https://mastodon.example/users/alice",
    "inReplyTo": "https://mastodon.example/users/alice/statuses/1009947848598745"
  }
}
```

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "id": "https://mastodon.example/users/bob#votes/827164/activity",
  "to": "https://mastodon.example/users/alice",
  "actor": "https://mastodon.example/users/bob",
  "type": "Create",
  "object": {
    "id": "https://mastodon.example/users/bob#votes/827164",
    "type": "Note",
    "name": "banana",
    "attributedTo": "https://mastodon.example/users/bob",
    "to": "https://mastodon.example/users/alice",
    "inReplyTo": "https://mastodon.example/users/alice/statuses/1009947848598745"
  }
}
```

### Mentions

In the `ActivityStreams` Vocabulary, `Mention` is a subtype of `Link` that is intended to represent the microsyntax of `@mentions`. The tag property is intended to add references to other `Objects` or `Links`. For Link tags, the name of the Link should be a substring of the natural language properties (`name`, `summary`, `content`) on that object. Wherever such a substring is found, it can be transformed into a hyperlink reference to the `href`.

However, Mastodon also uses Mention tags for addressing in some cases. Based on the presence or exclusion of Mention tags, and compared to the explicitly declared audiences in to and cc, Mastodon will calculate a visibility level for the post. Additionally, Mastodon requires Mention tags in order to generate a notification. (The mentioned actor must still be included within to or cc explicitly in order to receive the post.)

- `public`: Public statuses have the as:Public magic collection in `to`
- `unlisted`: Unlisted statuses have the as:Public magic collection in `cc`
- `private`: Followers-only statuses have an actor’s follower collection in `to` or `cc`, but do not include the `as:Public` magic collection
- `limited`: Limited-audience statuses have actors in `to` or `cc`, at least one of which is not Mentioned in tag
- `direct`: Mentions-only statuses have actors in `to` or `cc`, all of which are Mentioned in tag

### Public Key

Public keys are used for HTTP Signatures and Linked Data Signatures. This is implemented using an extra property `publicKey` on [actor](#actors) objects. See [HTTP Signatures](#http-signatures) for more information.

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://w3id.org/security/v1"
  ],
  "id": "https://mastodon.social/users/Gargron",
  "type": "Person",
  "publicKey": {
    "id": "https://mastodon.social/users/Gargron#main-key",
    "owner": "https://mastodon.social/users/Gargron",
    "publicKeyPem": "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAvXc4vkECU2/CeuSo1wtn\nFoim94Ne1jBMYxTZ9wm2YTdJq1oiZKif06I2fOqDzY/4q/S9uccrE9Bkajv1dnkO\nVm31QjWlhVpSKynVxEWjVBO5Ienue8gND0xvHIuXf87o61poqjEoepvsQFElA5ym\novljWGSA/jpj7ozygUZhCXtaS2W5AD5tnBQUpcO0lhItYPYTjnmzcc4y2NbJV8hz\n2s2G8qKv8fyimE23gY1XrPJg+cRF+g4PqFXujjlJ7MihD9oqtLGxbu7o1cifTn3x\nBfIdPythWu5b4cujNsB3m3awJjVmx+MHQ9SugkSIYXV0Ina77cTNS0M2PYiH1PFR\nTwIDAQAB\n-----END PUBLIC KEY-----\n"
  }
}
```

### Blurhash

Mastodon generates colorful preview thumbnails for attachments. This is implemented using an extra property blurhash on Image objects. The property is a string generated by the BlurHash algorithm.

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "blurhash": "http://joinmastodon.org/ns#blurhash"
    }
  ],

  "id": "https://example.com/@alice/hello-world",
  "type": "Note",
  "content": "A picture attached!",
  "attachment": [
    {
      "type": "Image",
      "mediaType": "image/png",
      "url": "https://example.com/files/cats.png",
      "blurhash": "UBL_:rOpGG-oBUNG,qRj2so|=eE1w^n4S5NH"
    }
  ]
}
```

### Featured Collection

What is known in Mastodon as “pinned statuses”, or statuses that are always featured at the top of people’s profiles, is implemented using an extra property featured on the actor object that points to a Collection of objects.

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "featured": {
        "@id": "http://joinmastodon.org/ns#featured",
        "@type": "@id"
      }
    }
  ],

  "id": "https://example.com/@alice",
  "type": "Person",
  "featured": "https://example.com/@alice/collections/featured"
}
```

### Featured Tags

Mastodon allows users to feature specific hashtags on their profile for easy browsing, as a discoverability mechanism. This is implemented using an extra property featuredTags on the actor object that points to a Collection of Hashtag objects specifically.

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "featuredTags": {
        "@id": "http://joinmastodon.org/ns#featuredTags",
        "@type": "@id"
      }
    }
  ],

  "id": "https://example.com/@alice",
  "type": "Person",
  "featuredTags": "https://example.com/@alice/collections/tags"
}
```

### Profile Metadata

Mastodon supports arbitrary profile fields containing name-value pairs. This is implemented using the attachment property on actor objects, with objects in the array having a type of PropertyValue and a value property, both from the schema.org namespace.

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    {
      "schema": "http://schema.org#",
      "PropertyValue": "schema:PropertyValue",
      "value": "schema:value"
    }
  ],
  "id": "https://mastodon.social/users/Gargron",
  "type": "Person",
  "attachment": [
    {
      "type": "PropertyValue",
      "name": "Patreon",
      "value": "<a href=\"https://www.patreon.com/mastodon\" rel=\"me nofollow noopener noreferrer\" target=\"_blank\"><span class=\"invisible\">https://www.</span><span class=\"\">patreon.com/mastodon</span><span class=\"invisible\"></span}"
    },
    {
      "type": "PropertyValue",
      "name": "Homepage",
      "value": "<a href=\"https://zeonfederated.com\" rel=\"me nofollow noopener noreferrer\" target=\"_blank\"><span class=\"invisible\">https://</span><span class=\"\">zeonfederated.com</span><span class=\"invisible\"></span}"
    }
  ]
}
```

### Account Migration

Mastodon uses the Move activity to signal that an account has migrated to a different account. For the migration to be considered valid, Mastodon checks that the new account has defined an alias pointing to the old account (via the `alsoKnownAs` property).

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "id": "https://mastodon.example/users/alice#moves/1",
  "actor": "https://mastodon.example/users/alice",
  "type": "Move",
  "object": "https://mastodon.example/users/alice",
  "target": "https://alice.com/users/109835986274379",
  "to": "https://mastodon.example/users/alice/followers"
}
```

### Remote Blocking

ActivityPub defines the Block activity for client-to-server (C2S) use-cases, but not for server-to-server (S2S) – it recommends that servers SHOULD NOT deliver Block activities to their object. However, Mastodon will send this activity when a local user blocks a remote user. When Mastodon receives a Block activity where the object is an actor on the local domain, it will interpret this as a signal to hide the actor’s profile and posts from the local user, as well as disallowing mentions of that actor by that local user.

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "id": "https://mastodon.example/bd06bb61-01e0-447a-9dc8-95915db9aec8",
  "type": "Block",
  "actor": "https://mastodon.example/users/alice",
  "object": "https://example.com/~mallory",
  "to": "https://example.com/~mallory"
}
```

---

## HTTP Signatures

HTTP Signatures are used to authenticate requests between servers (S2S) part of the **Federation Protocol**.

In particular, in the Public data for each user there is a `publicKey` property that contains the public key of the user. This public key is used to verify the signature of the request.

When a user is created on the server, the server generates and stores securely the private key of the user.

When the server sends an activity to the inbox of another server, the request MUST be signed with the private key of the user.

The server receiving the request MUST verify the signature using the public key of the user.

> **Mastic implementation:** In Mastic, each User Canister generates an RSA key pair at creation time. The private key
> is stored securely within the User Canister and is used by the Federation Canister to sign outgoing HTTP requests on
> behalf of the user. The public key is served by the Federation Canister when the actor profile is requested by a
> remote instance (e.g. to verify a signature).

For any HTTP request incoming to Mastodon for the Federation Protocol, the `Signature` header MUST be present and contain the signature of the request:

```txt
Signature: keyId="https://my.example.com/username#main-key",headers="(request-target) host date",signature="Y2FiYW...IxNGRiZDk4ZA=="
```

The three parts of the Signature: header can be broken down like so:

```txt
Signature:
  keyId="https://my.example.com/username#main-key",
  headers="(request-target) host date",
  signature="Y2FiYW...IxNGRiZDk4ZA=="
```

The `keyId` should correspond to the actor and the key being used to generate the signature, whose value is equal to all parameters in headers concatenated together and signed by the key, then Base64-encoded. See [Public key](#public-key) for more information on actor keys.

An example key looks like this:

```json
{
  "publicKey": {
    "id": "https://my.example.com/username#main-key",
    "owner": "https://my.example.com/username",
    "publicKeyPem": "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAvXc4vkECU2/CeuSo1wtn\nFoim94Ne1jBMYxTZ9wm2YTdJq1oiZKif06I2fOqDzY/4q/S9uccrE9Bkajv1dnkO\nVm31QjWlhVpSKynVxEWjVBO5Ienue8gND0xvHIuXf87o61poqjEoepvsQFElA5ym\novljWGSA/jpj7ozygUZhCXtaS2W5AD5tnBQUpcO0lhItYPYTjnmzcc4y2NbJV8hz\n2s2G8qKv8fyimE23gY1XrPJg+cRF+g4PqFXujjlJ7MihD9oqtLGxbu7o1cifTn3x\nBfIdPythWu5b4cujNsB3m3awJjVmx+MHQ9SugkSIYXV0Ina77cTNS0M2PYiH1PFR\nTwIDAQAB\n-----END PUBLIC KEY-----\n"
 }
}
```

### Signing POST requests

When making a POST request to Mastodon, you must calculate the `RSA-SHA256` digest hash of your request’s body and include this hash (in `base64` encoding) within the `Digest:` header. The `Digest:` header must also be included within the headers parameter of the `Signature:` header. For example:

```http
POST /users/username/inbox HTTP/1.1
HOST: mastodon.example
Date: 18 Dec 2019 10:08:46 GMT
Digest: sha-256=hcK0GZB1BM4R0eenYrj9clYBuyXs/lemt5iWRYmIX0A=
Signature: keyId="https://my.example.com/actor#main-key",headers="(request-target) host date digest",signature="Y2FiYW...IxNGRiZDk4ZA=="
Content-Type: application/ld+json; profile="https://www.w3.org/ns/activitystreams"

{
  "@context": "https://www.w3.org/ns/activitystreams",
  "actor": "https://my.example.com/actor",
  "type": "Create",
  "object": {
    "type": "Note",
    "content": "Hello!"
  },
  "to": "https://mastodon.example/users/username"
}
```

### Verifying Signatures

Mastodon verifies the signature using the following algorithm:

1. **Split Signature**: into its separate parameters.
2. **Construct the signature** string from the value of headers.
3. **Fetch the `keyId`** and resolve to an actor’s `publicKey`.
4. **RSA-SHA256 hash** the signature string and compare to the Base64-decoded signature as decrypted by `publicKey[publicKeyPem]`.
5. Use the `Date:` header to check that the signed request was made within the past `12 hours`.

---

## WebFinger

For fully-featured Mastodon support, Mastic also implements the WebFinger protocol, which is used to discover information about users and their profiles on the Fediverse. WebFinger is a protocol that allows clients to discover information about a user based on their account name or email address.

### WebFinger Simple Flow

Suppose we want to lookup the user `@Gargron` hosted on the mastodon.social website.

Just make a request to that domain’s `/.well-known/webfinger` endpoint, with the `resource` query parameter set to an `acct:` URI (e.g. `acct:veeso_dev@hachyderm.io`).

For instance: `https://hachyderm.io/.well-known/webfinger?resource=acct%3Aveeso_dev%40hachyderm.io`

```json
{
  "subject": "acct:veeso_dev@hachyderm.io",
  "aliases": [
    "https://hachyderm.io/@veeso_dev",
    "https://hachyderm.io/users/veeso_dev"
  ],
  "links": [
    {
      "rel": "http://webfinger.net/rel/profile-page",
      "type": "text/html",
      "href": "https://hachyderm.io/@veeso_dev"
    },
    {
      "rel": "self",
      "type": "application/activity+json",
      "href": "https://hachyderm.io/users/veeso_dev"
    },
    {
      "rel": "http://ostatus.org/schema/1.0/subscribe",
      "template": "https://hachyderm.io/authorize_interaction?uri={uri}"
    },
    {
      "rel": "http://webfinger.net/rel/avatar",
      "type": "image/png",
      "href": "https://media.hachyderm.io/accounts/avatars/114/410/957/328/747/476/original/1cc6bed1aa3ad81e.png"
    }
  ]
}
```
