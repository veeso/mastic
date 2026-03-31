//! JSON-LD `@context` helpers for ActivityPub payloads.
//!
//! Every ActivityPub document carries a `@context` field that maps short
//! property names to full IRIs. This module provides the [`Context`] and
//! [`ContextEntry`] enums for polymorphic deserialization, along with the
//! well-known URI constants ([`ACTIVITY_STREAMS_CONTEXT`],
//! [`SECURITY_CONTEXT_V1`], [`MASTODON_CONTEXT`]).
// Rust guideline compliant 2026-03-31

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The canonical ActivityStreams JSON-LD context URI.
pub const ACTIVITY_STREAMS_CONTEXT: &str = "https://www.w3.org/ns/activitystreams";

/// The canonical Linked Data security context URI used for ActivityPub public keys.
pub const SECURITY_CONTEXT_V1: &str = "https://w3id.org/security/v1";

/// The Mastodon extension namespace URI.
pub const MASTODON_CONTEXT: &str = "http://joinmastodon.org/ns#";

/// A single entry inside a JSON-LD `@context` array.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContextEntry {
    /// A plain context URI such as `https://www.w3.org/ns/activitystreams`.
    Uri(String),
    /// A local JSON-LD term definition map.
    Definition(BTreeMap<String, serde_json::Value>),
}

/// A JSON-LD `@context` value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Context {
    /// A single context URI.
    Uri(String),
    /// An array containing context URIs and/or term definition maps.
    Array(Vec<ContextEntry>),
    /// A standalone JSON-LD term definition map.
    Definition(BTreeMap<String, serde_json::Value>),
}
