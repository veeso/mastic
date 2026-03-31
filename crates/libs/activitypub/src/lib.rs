//! Canonical ActivityPub and ActivityStreams types used across Mastic.
//!
//! This crate provides Rust representations of the core
//! [ActivityStreams 2.0](https://www.w3.org/TR/activitystreams-core/) and
//! [ActivityPub](https://www.w3.org/TR/activitypub/) data model, together with
//! Mastodon-specific extensions and WebFinger discovery types.
//!
//! Every type derives [`serde::Serialize`] and [`serde::Deserialize`] so that
//! payloads can be round-tripped between JSON and Rust with zero boilerplate.
// Rust guideline compliant 2026-03-31

#![forbid(unsafe_code)]

/// Activity representations such as [`Activity`], [`ActivityType`], and
/// [`ActivityObject`].
///
/// Covers the full set of ActivityPub side-effects: `Create`, `Update`,
/// `Delete`, `Follow`, `Accept`, `Reject`, `Like`, `Announce`, `Undo`,
/// `Block`, `Add`, `Remove`, `Flag`, and `Move`.
pub mod activity;

/// Actor representations such as [`Actor`], [`ActorType`], [`PublicKey`], and
/// [`Endpoints`].
///
/// Models the five standard ActivityPub actor types (`Person`, `Application`,
/// `Service`, `Group`, `Organization`) plus Mastodon-specific profile
/// extensions like discoverability and featured collections.
pub mod actor;

/// Collection and collection page representations.
///
/// Provides [`Collection`], [`OrderedCollection`], [`CollectionPage`], and
/// [`OrderedCollectionPage`] — the four collection shapes defined by
/// ActivityStreams 2.0.
pub mod collection;

/// JSON-LD `@context` helpers and well-known context constants.
///
/// Contains the [`Context`] and [`ContextEntry`] enums used in every
/// ActivityPub payload, plus the [`ACTIVITY_STREAMS_CONTEXT`],
/// [`SECURITY_CONTEXT_V1`], and [`MASTODON_CONTEXT`] URI constants.
pub mod context;

/// Link and link-like representations.
///
/// Defines the base [`Link`] type as well as [`Mention`] and [`Hashtag`]
/// subtypes used in tag arrays.
pub mod link;

/// Mastodon-specific extensions layered on top of ActivityPub.
///
/// Provides [`PropertyValue`] for profile metadata fields, [`FocalPoint`]
/// for image cropping hints, and the [`MediaReference`](mastodon::MediaReference)
/// type alias.
pub mod mastodon;

/// ActivityStreams object representations.
///
/// The core of the data model: [`BaseObject`] carries all shared fields
/// (content, addressing, attachments, polls, etc.), while [`Object`] is the
/// concrete alias parameterized by [`ObjectType`]. Also defines [`OneOrMany`],
/// [`Reference`], [`Source`], and [`Attachment`].
pub mod object;

/// Tag representations such as [`Mention`](link::Mention),
/// [`Hashtag`](link::Hashtag), and [`Emoji`].
///
/// The [`Tag`] enum deserializes polymorphically via `#[serde(untagged)]`.
pub mod tag;

/// WebFinger payload representations.
///
/// Provides [`WebFingerResponse`] and [`WebFingerLink`] for
/// [RFC 7033](https://tools.ietf.org/html/rfc7033) account discovery used
/// across the Fediverse.
pub mod webfinger;

#[doc(inline)]
pub use activity::{Activity, ActivityObject, ActivityType};
#[doc(inline)]
pub use actor::{Actor, ActorType, Endpoints, PublicKey};
#[doc(inline)]
pub use collection::{
    Collection, CollectionPage, CollectionType, OrderedCollection, OrderedCollectionPage,
};
#[doc(inline)]
pub use context::{
    ACTIVITY_STREAMS_CONTEXT, Context, ContextEntry, MASTODON_CONTEXT, SECURITY_CONTEXT_V1,
};
#[doc(inline)]
pub use link::{Hashtag, HashtagType, Link, Mention, MentionType};
#[doc(inline)]
pub use mastodon::{FocalPoint, PropertyValue, PropertyValueType};
#[doc(inline)]
pub use object::{Attachment, BaseObject, Object, ObjectType, Reference, Source};
#[doc(inline)]
pub use tag::{Emoji, EmojiType, Tag};
#[doc(inline)]
pub use webfinger::{WebFingerLink, WebFingerResponse};

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip<T>(json: &str)
    where
        T: for<'de> serde::Deserialize<'de> + serde::Serialize + PartialEq + core::fmt::Debug,
    {
        let value: T = serde_json::from_str(json).expect("JSON must deserialize");
        let serialized = serde_json::to_string(&value).expect("value must serialize");
        let reparsed: T =
            serde_json::from_str(&serialized).expect("serialized JSON must deserialize");
        assert_eq!(value, reparsed);
    }

    #[test]
    fn object_round_trip_from_document_example() {
        let json = r##"{
          "@context": "https://www.w3.org/ns/activitystreams",
          "type": "Note",
          "content": "This is a note",
          "published": "2015-02-10T15:04:55Z",
          "to": ["https://example.org/~john/"],
          "cc": [
            "https://example.com/~erik/followers",
            "https://www.w3.org/ns/activitystreams#Public"
          ]
        }"##;

        round_trip::<Object>(json);
    }

    #[test]
    fn actor_round_trip_from_public_key_example() {
        let json = r##"{
          "@context": [
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1"
          ],
          "id": "https://mastodon.social/users/Gargron",
          "type": "Person",
          "inbox": "https://mastodon.social/users/Gargron/inbox",
          "outbox": "https://mastodon.social/users/Gargron/outbox",
          "following": "https://mastodon.social/users/Gargron/following",
          "followers": "https://mastodon.social/users/Gargron/followers",
          "liked": "https://mastodon.social/users/Gargron/liked",
          "publicKey": {
            "id": "https://mastodon.social/users/Gargron#main-key",
            "owner": "https://mastodon.social/users/Gargron",
            "publicKeyPem": "-----BEGIN PUBLIC KEY-----\nABC\n-----END PUBLIC KEY-----\n"
          }
        }"##;

        round_trip::<Actor>(json);
    }

    #[test]
    fn collection_round_trip() {
        let json = r##"{
          "@context": "https://www.w3.org/ns/activitystreams",
          "id": "https://www.w3.org/ns/activitystreams",
          "type": "Collection",
          "totalItems": 1,
          "items": ["https://example.com/items/1"]
        }"##;

        round_trip::<Collection>(json);
    }

    #[test]
    fn ordered_collection_round_trip() {
        let json = r##"{
          "@context": "https://www.w3.org/ns/activitystreams",
          "id": "https://social.example/alice/posts/1/likes",
          "type": "OrderedCollection",
          "totalItems": 5,
          "orderedItems": [
            "https://social.example/bob/likes/1",
            "https://social.example/carol/likes/3"
          ]
        }"##;

        round_trip::<OrderedCollection>(json);
    }

    #[test]
    fn collection_page_round_trip() {
        let json = r##"{
          "@context": "https://www.w3.org/ns/activitystreams",
          "id": "https://example.com/users/alice/followers?page=1",
          "type": "CollectionPage",
          "partOf": "https://example.com/users/alice/followers",
          "next": "https://example.com/users/alice/followers?page=2",
          "items": ["https://example.com/users/bob"]
        }"##;

        round_trip::<CollectionPage>(json);
    }

    #[test]
    fn ordered_collection_page_round_trip() {
        let json = r##"{
          "@context": "https://www.w3.org/ns/activitystreams",
          "id": "https://example.com/users/alice/outbox?page=1",
          "type": "OrderedCollectionPage",
          "partOf": "https://example.com/users/alice/outbox",
          "prev": "https://example.com/users/alice/outbox?page=0",
          "orderedItems": ["https://example.com/users/alice/statuses/1"]
        }"##;

        round_trip::<OrderedCollectionPage>(json);
    }

    #[test]
    fn mention_round_trip() {
        let json = r##"{
          "type": "Mention",
          "href": "https://example.com/@alice",
          "name": "@alice@example.com"
        }"##;

        round_trip::<Mention>(json);
    }

    #[test]
    fn hashtag_round_trip() {
        let json = r##"{
          "type": "Hashtag",
          "name": "#cats",
          "href": "https://example.com/tagged/cats"
        }"##;

        round_trip::<Hashtag>(json);
    }

    #[test]
    fn emoji_round_trip() {
        let json = r##"{
          "id": "https://example.com/emoji/123",
          "type": "Emoji",
          "name": ":kappa:",
          "icon": {
            "type": "Image",
            "mediaType": "image/png",
            "url": "https://example.com/files/kappa.png"
          }
        }"##;

        round_trip::<Emoji>(json);
    }

    #[test]
    fn webfinger_round_trip() {
        let json = r##"{
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
            }
          ]
        }"##;

        round_trip::<WebFingerResponse>(json);
    }

    #[test]
    fn every_activity_type_deserializes() {
        let cases = [
            (
                ActivityType::Create,
                r##"{"@context":"https://www.w3.org/ns/activitystreams","type":"Create","actor":"https://example.com/users/alice","object":{"type":"Note","content":"hello"}}"##,
            ),
            (
                ActivityType::Update,
                r##"{"@context":"https://www.w3.org/ns/activitystreams","type":"Update","actor":"https://example.com/users/alice","object":{"type":"Note","id":"https://example.com/statuses/1","content":"edited"}}"##,
            ),
            (
                ActivityType::Delete,
                r##"{"@context":"https://www.w3.org/ns/activitystreams","type":"Delete","actor":"https://example.com/users/alice","object":{"type":"Tombstone","id":"https://example.com/statuses/1","deleted":"2025-01-01T00:00:00Z"}}"##,
            ),
            (
                ActivityType::Follow,
                r##"{"@context":"https://www.w3.org/ns/activitystreams","type":"Follow","actor":"https://example.com/users/bob","object":"https://example.com/users/alice"}"##,
            ),
            (
                ActivityType::Accept,
                r##"{"@context":"https://www.w3.org/ns/activitystreams","type":"Accept","actor":"https://example.com/users/alice","object":{"type":"Follow","actor":"https://example.com/users/bob","object":"https://example.com/users/alice"}}"##,
            ),
            (
                ActivityType::Reject,
                r##"{"@context":"https://www.w3.org/ns/activitystreams","type":"Reject","actor":"https://example.com/users/alice","object":{"type":"Follow","actor":"https://example.com/users/bob","object":"https://example.com/users/alice"}}"##,
            ),
            (
                ActivityType::Like,
                r##"{"@context":"https://www.w3.org/ns/activitystreams","type":"Like","actor":"https://example.com/users/alice","object":"https://example.com/statuses/1"}"##,
            ),
            (
                ActivityType::Announce,
                r##"{"@context":"https://www.w3.org/ns/activitystreams","type":"Announce","actor":"https://example.com/users/alice","object":"https://example.com/statuses/1"}"##,
            ),
            (
                ActivityType::Undo,
                r##"{"@context":"https://www.w3.org/ns/activitystreams","type":"Undo","actor":"https://example.com/users/alice","object":{"type":"Like","actor":"https://example.com/users/alice","object":"https://example.com/statuses/1"}}"##,
            ),
            (
                ActivityType::Block,
                r##"{"@context":"https://www.w3.org/ns/activitystreams","type":"Block","actor":"https://example.com/users/alice","object":"https://example.com/users/eve"}"##,
            ),
            (
                ActivityType::Add,
                r##"{"@context":"https://www.w3.org/ns/activitystreams","type":"Add","actor":"https://example.com/users/alice","object":"https://example.com/statuses/1","target":"https://example.com/collections/featured"}"##,
            ),
            (
                ActivityType::Remove,
                r##"{"@context":"https://www.w3.org/ns/activitystreams","type":"Remove","actor":"https://example.com/users/alice","object":"https://example.com/statuses/1","target":"https://example.com/collections/featured"}"##,
            ),
            (
                ActivityType::Flag,
                r##"{"@context":"https://www.w3.org/ns/activitystreams","type":"Flag","actor":"https://example.com/users/alice","object":"https://example.com/statuses/1"}"##,
            ),
            (
                ActivityType::Move,
                r##"{"@context":"https://www.w3.org/ns/activitystreams","type":"Move","actor":"https://example.com/users/alice","object":"https://example.com/users/alice","target":"https://example.net/users/alice"}"##,
            ),
        ];

        for (expected, json) in cases {
            let activity: Activity = serde_json::from_str(json).expect("activity must deserialize");
            assert_eq!(activity.base.kind, expected);
            let encoded = serde_json::to_string(&activity).expect("activity must serialize");
            let reparsed: Activity =
                serde_json::from_str(&encoded).expect("activity must round trip");
            assert_eq!(activity, reparsed);
        }
    }

    #[test]
    fn mastodon_poll_object_deserializes() {
        let json = r##"{
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
            }
          ]
        }"##;

        round_trip::<Object>(json);
    }
}
