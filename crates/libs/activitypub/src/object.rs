//! ActivityStreams object types and shared object fields.
//!
//! [`BaseObject`] is the core building block: a generic struct parameterized by
//! its `type` discriminator that carries all shared ActivityStreams fields
//! (content, addressing, attachments, polls, etc.). The [`Object`] type alias
//! pins the discriminator to [`ObjectType`].
//!
//! Helper enums [`OneOrMany`], [`Reference`], and [`Attachment`] handle the
//! polymorphic shapes that appear throughout ActivityPub JSON payloads.
// Rust guideline compliant 2026-03-31

use serde::{Deserialize, Serialize};

use crate::collection::Collection;
use crate::context::Context;
use crate::link::Link;
use crate::mastodon::{FocalPoint, PropertyValue};
use crate::tag::Tag;

/// A value that can be serialized either as a single item or as an array of items.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    /// A single value.
    One(T),
    /// Multiple values.
    Many(Vec<T>),
}

/// A value that can be represented either as a stable URI string or as an embedded object.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Reference<T> {
    /// A dereferenceable URI.
    Id(String),
    /// An embedded representation.
    Object(Box<T>),
}

/// The original source body from which a rendered object was derived.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// Raw source content, for example Markdown.
    pub content: String,
    /// MIME type of the source content.
    pub media_type: String,
}

/// The ActivityStreams object family of type discriminators.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    /// A short textual post.
    Note,
    /// A poll or question object.
    Question,
    /// An image object.
    Image,
    /// A tombstone replacing a deleted object.
    Tombstone,
    /// A generic document object.
    Document,
    /// An article or long-form entry.
    Article,
    /// An audio object.
    Audio,
    /// A video object.
    Video,
    /// An event object.
    Event,
    /// A place object.
    Place,
    /// A profile object.
    Profile,
    /// A page object.
    Page,
    /// Any other object type not explicitly modeled above.
    #[serde(other)]
    Other,
}

/// Shared ActivityStreams object fields parameterized by the concrete `type` discriminator.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseObject<T> {
    /// Optional JSON-LD context for the payload.
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
    /// Stable URI uniquely identifying the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The ActivityStreams `type` discriminator.
    #[serde(rename = "type")]
    pub kind: T,
    /// Natural-language or HTML body content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Human-readable name or title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Summary or content warning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Publication timestamp in RFC 3339 format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    /// Last update timestamp in RFC 3339 format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    /// Canonical or alternate URLs for this object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<OneOrMany<Reference<Link>>>,
    /// Primary recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<OneOrMany<String>>,
    /// Carbon-copy recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<OneOrMany<String>>,
    /// Blind primary recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bto: Option<OneOrMany<String>>,
    /// Blind carbon-copy recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<OneOrMany<String>>,
    /// Audience collections or actor URIs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<OneOrMany<String>>,
    /// Actors credited with authoring the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributed_to: Option<OneOrMany<String>>,
    /// Parent object or conversation object URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<OneOrMany<String>>,
    /// Original source body from which `content` was rendered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// Tags associated with the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<OneOrMany<Tag>>,
    /// Attached media or profile metadata records.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<OneOrMany<Attachment>>,
    /// Replies collection or reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replies: Option<Reference<Collection>>,
    /// Likes collection or reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub likes: Option<Reference<Collection>>,
    /// Shares collection or reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shares: Option<Reference<Collection>>,
    /// Whether the object or its media should be treated as sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitive: Option<bool>,
    /// MIME type of the object representation, especially for media objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    /// Tombstone deletion timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<String>,
    /// Poll closing timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    /// Timestamp at which the poll became closed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<String>,
    /// Number of distinct voters recorded for a Mastodon poll.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voters_count: Option<u64>,
    /// Single-choice poll options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<Object>>,
    /// Multiple-choice poll options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<Object>>,
    /// Mastodon attachment blurhash preview.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blurhash: Option<String>,
    /// Mastodon focal point for image media.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focal_point: Option<FocalPoint>,
    /// Experimental quote-post URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_uri: Option<String>,
    /// Experimental quote-post URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_url: Option<String>,
    /// Compatibility field used by Misskey quote posts.
    #[serde(rename = "_misskey_quote", skip_serializing_if = "Option::is_none")]
    pub misskey_quote: Option<String>,
}

/// The canonical ActivityStreams object representation used by the crate.
pub type Object = BaseObject<ObjectType>;

/// Any value accepted by the `attachment` field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Attachment {
    /// A Mastodon profile metadata field.
    PropertyValue(PropertyValue),
    /// A generic link attachment.
    Link(Link),
    /// An embedded ActivityStreams object such as an `Image`.
    Object(Box<Object>),
}
