//! Type definitions for the Federation canister

#[cfg(test)]
mod tests;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

/// Install arguments for the Federation canister.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum FederationInstallArgs {
    /// Initial installation argument, provided on `init`
    Init {
        /// Principal of the Directory canister
        directory_canister: candid::Principal,
        /// The URL of this server's public endpoint (e.g. `https://example.com`)
        public_url: String,
    },
    /// Upgrade argument, provided on `upgrade`
    Upgrade {},
}

/// ActivityPub activity type discriminator for canister calls.
///
/// Mirrors the activity verbs defined by the ActivityPub specification. Unlike
/// [`activitypub::ActivityType`], this enum does **not** use `#[serde(other)]`
/// or `#[serde(untagged)]`, so it round-trips cleanly through Candid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, CandidType, Serialize, Deserialize)]
pub enum ActivityType {
    /// Wraps the creation of a new object.
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

/// A Candid-compatible representation of an ActivityPub activity.
///
/// Designed to cross canister call boundaries. Complex nested fields
/// (object, result, instrument) are carried as serialized JSON strings
/// because the polymorphic shapes used by ActivityPub (`#[serde(untagged)]`)
/// are incompatible with Candid's variant encoding.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct Activity {
    /// Stable URI identifying the activity.
    pub id: Option<String>,
    /// The activity verb.
    pub activity_type: ActivityType,
    /// Actor URI performing the activity.
    pub actor: Option<String>,
    /// The object of the activity, serialized as JSON.
    ///
    /// May be a URI string, a nested activity, or an embedded object.
    pub object_json: Option<String>,
    /// Target collection or actor URI.
    pub target: Option<String>,
    /// Primary recipients.
    pub to: Vec<String>,
    /// Carbon-copy recipients.
    pub cc: Vec<String>,
    /// Publication timestamp in RFC 3339 format.
    pub published: Option<String>,
}

/// Arguments for the `register_user` method of the Federation canister.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct RegisterUserArgs {
    pub user_id: Principal,
    pub user_handle: String,
    pub user_canister_id: Principal,
}

/// Response type for the `register_user` method of the Federation canister.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum RegisterUserResponse {
    Ok,
    Err(RegisterUserError),
}

/// Error type returned by the `register_user` method of the Federation canister.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum RegisterUserError {
    Internal(String),
}

/// An object sent to the `send_activity` inside of [`SendActivityArgs`] for a single activity.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct SendActivityArgsObject {
    /// JSON-encoded ActivityPub activity object to send.
    pub activity_json: String,
    /// URL of the remote actor's inbox to deliver the activity to.
    pub target_inbox: String,
}

/// Arguments for the `send_activity` method of the Federation canister, supporting both single and batch activities.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum SendActivityArgs {
    /// Send a single activity.
    One(SendActivityArgsObject),
    /// Send a batch of activities.
    Batch(Vec<SendActivityArgsObject>),
}

/// Error type returned by the `send_activity` method of the Federation canister.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum SendActivityError {
    /// the caller is not a registered User Canister.
    Unauthorized,
    /// the HTTP request to the target inbox failed.
    DeliveryFailed,
    /// the JSON could not be parsed as a valid ActivityPub activity.
    InvalidActivity,
}

/// Response type for the `send_activity` method of the Federation canister.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum SendActivityResponse {
    Ok,
    Err(SendActivityError),
}
