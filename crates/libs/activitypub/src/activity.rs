//! ActivityPub activity types.
//!
//! An [`Activity`] wraps a side-effect performed by an actor, such as creating
//! a post, following another actor, or liking an object. The [`ActivityType`]
//! enum covers all activity verbs used across ActivityPub and Mastodon.
//!
//! The `object` field uses [`ActivityObject`] to support both URI references
//! and recursively nested activities (e.g. `Accept(Follow)`).
// Rust guideline compliant 2026-03-31

use serde::{Deserialize, Serialize};

use crate::object::{BaseObject, Object, Reference};

/// The ActivityPub activity family of type discriminators.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum ActivityType {
    /// Wraps the creation of a new object.
    #[default]
    Create,
    /// Updates an existing object.
    Update,
    /// Deletes an existing object.
    Delete,
    /// Requests to follow an actor.
    Follow,
    /// Accepts a previous activity such as `Follow`.
    Accept,
    /// Rejects a previous activity such as `Follow`.
    Reject,
    /// Likes an object.
    Like,
    /// Re-shares an object.
    Announce,
    /// Reverses a previous activity.
    Undo,
    /// Blocks an actor.
    Block,
    /// Adds an object to a target collection.
    Add,
    /// Removes an object from a target collection.
    Remove,
    /// Reports or flags an object.
    Flag,
    /// Signals account migration.
    Move,
}

/// A nested value accepted by the `object` field of an activity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActivityObject {
    /// A URI reference to the target object or activity.
    Id(String),
    /// An embedded nested activity, for example `Accept(Follow)` or `Undo(Like)`.
    Activity(Box<Activity>),
    /// An embedded ActivityStreams object such as a `Note`.
    Object(Box<Object>),
}

/// A concrete ActivityPub activity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    /// Shared ActivityStreams object fields for the activity payload.
    #[serde(flatten)]
    pub base: BaseObject<ActivityType>,
    /// Actor performing the activity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    /// Primary object or nested activity acted upon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<ActivityObject>,
    /// Target collection or actor affected by the activity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Result object created or produced by the activity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Reference<Object>>,
    /// Origin collection or source context of the activity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    /// Instrument used to perform the activity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument: Option<Reference<Object>>,
}
