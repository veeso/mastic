//! Link primitives used by ActivityStreams and Mastodon tags.
//!
//! The base [`Link`] type models an ActivityStreams link object. [`Mention`]
//! and [`Hashtag`] are specialized subtypes that carry their own `type`
//! discriminator and appear inside `tag` arrays on posts.
// Rust guideline compliant 2026-03-31

use serde::{Deserialize, Serialize};

/// A generic ActivityStreams link.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    /// The target URI of the link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    /// Relationship types associated with the linked resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<Vec<String>>,
    /// MIME type of the target resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    /// Human-readable label for the link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Language code for the linked representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<String>,
    /// Height of the linked media resource in CSS pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u64>,
    /// Width of the linked media resource in CSS pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u64>,
}

/// The concrete ActivityStreams type value for a `Mention`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MentionType {
    /// A tag that references an actor or account.
    Mention,
}

/// A `Link` subtype representing an `@mention`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mention {
    /// The concrete ActivityStreams type discriminator.
    #[serde(rename = "type")]
    pub kind: MentionType,
    /// The target actor URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    /// Human-readable representation such as `@alice@example.com`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Relationship types associated with the target resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<Vec<String>>,
    /// MIME type of the target representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    /// Language code for the link label or representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<String>,
    /// Height of a linked media resource when relevant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u64>,
    /// Width of a linked media resource when relevant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u64>,
}

/// The concrete Mastodon/ActivityStreams type value for a `Hashtag`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashtagType {
    /// A tag that references a hashtag timeline.
    Hashtag,
}

/// A `Link` subtype representing a hashtag.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hashtag {
    /// The concrete type discriminator.
    #[serde(rename = "type")]
    pub kind: HashtagType,
    /// The target hashtag URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    /// Human-readable hashtag text such as `#cats`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Relationship types associated with the hashtag resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<Vec<String>>,
    /// MIME type of the target representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    /// Language code for the label or representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<String>,
    /// Height of a linked media resource when relevant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u64>,
    /// Width of a linked media resource when relevant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u64>,
}
