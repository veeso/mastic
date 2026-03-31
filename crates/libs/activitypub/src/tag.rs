//! Tag types used by ActivityPub and Mastodon.
//!
//! Tags appear inside the `tag` array of posts and are deserialized
//! polymorphically via the [`Tag`] enum. Supported variants include
//! [`Mention`](crate::link::Mention), [`Hashtag`](crate::link::Hashtag), and
//! the Mastodon-specific [`Emoji`] custom emoji type.
// Rust guideline compliant 2026-03-31

use serde::{Deserialize, Serialize};

use crate::link::{Hashtag, Mention};
use crate::object::Object;

/// The concrete Mastodon type value for custom emoji tags.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmojiType {
    /// A custom emoji tag.
    Emoji,
}

/// A Mastodon custom emoji tag.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Emoji {
    /// Stable URI identifying the emoji resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The concrete type discriminator.
    #[serde(rename = "type")]
    pub kind: EmojiType,
    /// Emoji shortcode such as `:kappa:`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Image object describing the emoji asset.
    pub icon: Box<Object>,
}

/// A tag that may be serialized as a `Mention`, `Hashtag`, or `Emoji`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Tag {
    /// An `@mention` tag.
    Mention(Mention),
    /// A hashtag tag.
    Hashtag(Hashtag),
    /// A custom emoji tag.
    Emoji(Emoji),
}
