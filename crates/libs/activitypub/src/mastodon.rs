//! Mastodon-specific ActivityPub extensions.
//!
//! Mastodon extends the base ActivityPub vocabulary with profile metadata
//! fields ([`PropertyValue`]), image cropping hints ([`FocalPoint`]), and
//! a [`MediaReference`] alias used for icon and header images on actors.
// Rust guideline compliant 2026-03-31

use serde::{Deserialize, Serialize};

use crate::Object;
use crate::object::Reference;

/// The concrete type value used by Mastodon profile metadata attachments.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyValueType {
    /// A schema.org `PropertyValue`.
    PropertyValue,
}

/// A Mastodon profile metadata field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyValue {
    /// The concrete type discriminator.
    #[serde(rename = "type")]
    pub kind: PropertyValueType,
    /// Human-readable field label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// HTML or plain-text field value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// A Mastodon focal point encoded as a two-element JSON array.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FocalPoint(
    /// Horizontal focal point coordinate in the `[-1.0, 1.0]` range.
    pub f64,
    /// Vertical focal point coordinate in the `[-1.0, 1.0]` range.
    pub f64,
);

/// A helper alias for `Object` references embedded in attachments or media properties.
pub type MediaReference = Reference<Object>;
