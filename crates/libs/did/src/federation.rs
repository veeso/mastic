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
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum SendActivityError {
    /// The `target_inbox` URL failed to parse or has an unexpected path shape.
    InvalidTargetInbox(String),
    /// The local inbox references a handle that is not registered in the
    /// Directory Canister.
    UnknownLocalUser(String),
    /// The inter-canister call to the target User Canister failed (transport
    /// or decode).
    DeliveryFailed(String),
    /// The target User Canister accepted the call but rejected the activity.
    Rejected(String),
}

/// Per-activity outcome of a `send_activity` call.
///
/// Carries the delivery result for a single [`SendActivityArgsObject`]. Batch
/// calls return one result per input object, index-aligned with the request.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum SendActivityResult {
    Ok,
    Err(SendActivityError),
}

/// Response type for the `send_activity` method of the Federation canister.
///
/// Mirrors the shape of [`SendActivityArgs`]: a [`SendActivityArgs::One`]
/// request returns [`SendActivityResponse::One`], and a
/// [`SendActivityArgs::Batch`] request returns [`SendActivityResponse::Batch`]
/// with one [`SendActivityResult`] per input object in the same order.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum SendActivityResponse {
    /// Outcome for a single activity delivery.
    One(SendActivityResult),
    /// Per-activity outcomes for a batch delivery, index-aligned with the
    /// request.
    Batch(Vec<SendActivityResult>),
}

/// Request arguments for the `fetch_status` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct FetchStatusArgs {
    /// ActivityPub URI of the status to fetch.
    pub uri: String,
    /// Optional actor URI of the requester. Forwarded to the target user
    /// canister so it can apply visibility rules.
    pub requester_actor_uri: Option<String>,
}

/// Error type for the `fetch_status` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum FetchStatusError {
    /// The URI host is not local. M1 only supports local fetch; HTTPS
    /// outcalls land in M3.
    Unsupported,
    /// The URI could not be parsed (host or path shape).
    InvalidUri,
    /// The status was not found, or the requester is not allowed to see it.
    NotFound,
    /// Internal error occurred while fetching the status.
    Internal(String),
}

/// Response type for the `fetch_status` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum FetchStatusResponse {
    Ok(crate::common::Status),
    Err(FetchStatusError),
}
