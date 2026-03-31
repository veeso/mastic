//! WebFinger response types for Fediverse account discovery.
//!
//! [WebFinger (RFC 7033)](https://tools.ietf.org/html/rfc7033) is the
//! discovery protocol used by Mastodon and other Fediverse software to
//! resolve `acct:` URIs to ActivityPub actor endpoints. This module
//! provides [`WebFingerResponse`] and [`WebFingerLink`].
// Rust guideline compliant 2026-03-31

use serde::{Deserialize, Serialize};

/// A link entry inside a WebFinger response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WebFingerLink {
    /// Registered or extension relation type.
    pub rel: String,
    /// MIME type of the target representation.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    /// Target URI of the link relation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    /// URI template used by interactive relations such as subscribe.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

/// A WebFinger discovery document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WebFingerResponse {
    /// Account or resource identifier being described.
    pub subject: String,
    /// Alternate URIs for the same resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
    /// Relation links advertised for the resource.
    pub links: Vec<WebFingerLink>,
}
