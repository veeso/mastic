//! Type definitions common to all the canisters

#[cfg(test)]
mod tests;

use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Controls the audience of a status post. Maps to ActivityPub addressing
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, CandidType, Serialize, Deserialize,
)]
pub enum Visibility {
    /// visible to everyone and included in public timelines
    Public,
    /// visible to everyone via direct link, but excluded from public timelines
    Unlisted,
    /// visible only to the author's followers
    FollowersOnly,
    /// visible only to explicitly mentioned users
    Direct,
}

/// A user's public profile information. Stored in the User Canister and returned
/// by profile queries.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct UserProfile {
    /// Unique username chosen at sign-up (e.g. `alice`).
    pub handle: String,
    /// Optional human-readable name shown in the UI.
    pub display_name: Option<String>,
    /// Optional free-text biography.
    pub bio: Option<String>,
    /// Optional image data for the user's avatar. Can be empty if no avatar is set.
    pub avatar: Option<Vec<u8>>,
    /// Optional header image data for the user's profile. Can be empty if no header is set.
    pub header: Option<Vec<u8>>,
    /// Timestamp (milliseconds since epoch) of account creation.
    pub created_at: u64,
}

/// A single post authored by a user. Each status has a unique ID, content body,
/// author principal, creation timestamp, and visibility setting.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct Status {
    /// Unique identifier for this status, assigned by the User Canister. (Snowflake id)
    pub id: u64,
    /// The text content of the status.
    pub content: String,
    /// The actor URI of the status author (e.g. `https://mastic.social/users/alice`).
    pub author: String,
    /// Timestamp (milliseconds since epoch) of when the status was created.
    pub created_at: u64,
    /// The visibility setting of the status, controlling its audience.
    pub visibility: Visibility,
}

/// A single entry in a user's feed. Wraps a [`Status`] and optionally indicates
/// that it was boosted (reblogged) by another user.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct FeedItem {
    /// The status associated with this feed item.
    pub status: Status,
    /// If this feed item was boosted (reblogged) by another user, the actor URI
    /// of the user that performed the boost. Otherwise [`None`].
    ///
    /// It works like this: if Alice creates a status, and Bob boosts it,
    /// then a new Feed Item is created with `boosted_by` set to Bob's actor URI.
    pub boosted_by: Option<String>,
}

/// A helper enum for update operations on optional fields.
/// For example, when updating a user profile,
/// the client can specify
/// - [`FieldUpdate::Clear`] to remove the value (set it to `None`),
/// - [`FieldUpdate::Leave`] to keep the existing value,
/// - [`FieldUpdate::Set`] to update it to a new value.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum FieldUpdate<T> {
    /// Clear the field (set it to `None`)
    Clear,
    /// Leave the field unchanged
    Leave,
    /// Set the field to a new value
    Set(T),
}

impl<T> FieldUpdate<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> FieldUpdate<U> {
        match self {
            FieldUpdate::Clear => FieldUpdate::Clear,
            FieldUpdate::Leave => FieldUpdate::Leave,
            FieldUpdate::Set(value) => FieldUpdate::Set(f(value)),
        }
    }
}
