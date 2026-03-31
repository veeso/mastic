//! ActivityStreams collection types.
//!
//! Collections are used throughout ActivityPub to expose followers, following,
//! outbox, inbox, liked, and featured items. This module provides both full
//! collections ([`Collection`], [`OrderedCollection`]) and their paginated
//! counterparts ([`CollectionPage`], [`OrderedCollectionPage`]).
// Rust guideline compliant 2026-03-31

use serde::{Deserialize, Serialize};

use crate::context::Context;

/// The ActivityStreams collection family of type discriminators.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollectionType {
    /// An unordered collection.
    Collection,
    /// An ordered collection.
    OrderedCollection,
    /// A page of an unordered collection.
    CollectionPage,
    /// A page of an ordered collection.
    OrderedCollectionPage,
}

/// A generic unordered ActivityStreams collection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    /// Optional JSON-LD context for the collection payload.
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
    /// Stable URI identifying the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The concrete collection type, usually `Collection`.
    #[serde(rename = "type")]
    pub kind: CollectionType,
    /// Number of items contained by the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_items: Option<u64>,
    /// URI of the first page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<String>,
    /// URI of the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
    /// URI of the current page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
    /// Collection members.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<serde_json::Value>>,
}

/// An ordered collection whose items are presented in a meaningful order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderedCollection {
    /// Optional JSON-LD context for the collection payload.
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
    /// Stable URI identifying the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The concrete collection type, usually `OrderedCollection`.
    #[serde(rename = "type")]
    pub kind: CollectionType,
    /// Number of items contained by the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_items: Option<u64>,
    /// URI of the first page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<String>,
    /// URI of the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
    /// URI of the current page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
    /// Ordered collection members.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ordered_items: Option<Vec<serde_json::Value>>,
}

/// A page inside an unordered collection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionPage {
    /// Optional JSON-LD context for the page payload.
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
    /// Stable URI identifying the page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The concrete page type, usually `CollectionPage`.
    #[serde(rename = "type")]
    pub kind: CollectionType,
    /// Number of items exposed by the page or whole collection when provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_items: Option<u64>,
    /// Collection this page belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_of: Option<String>,
    /// URI of the next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    /// URI of the previous page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    /// Page members.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<serde_json::Value>>,
}

/// A page inside an ordered collection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderedCollectionPage {
    /// Optional JSON-LD context for the page payload.
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
    /// Stable URI identifying the page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The concrete page type, usually `OrderedCollectionPage`.
    #[serde(rename = "type")]
    pub kind: CollectionType,
    /// Number of items exposed by the page or whole collection when provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_items: Option<u64>,
    /// Collection this page belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_of: Option<String>,
    /// URI of the next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    /// URI of the previous page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    /// Ordered page members.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ordered_items: Option<Vec<serde_json::Value>>,
}
