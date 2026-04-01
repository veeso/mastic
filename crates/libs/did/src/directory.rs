//! Type definitions for the Directory canister

#[cfg(test)]
mod tests;

use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Install arguments for the Directory canister.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum DirectoryInstallArgs {
    /// Initial installation argument, provided on `init`
    Init {
        /// The principal of the initial moderator who has permission to manage the directory.
        initial_moderator: candid::Principal,
        /// Principal of the Federation canister
        federation_canister: candid::Principal,
    },
    /// Upgrade argument, provided on `upgrade`
    Upgrade {},
}

/// Response error types for the `sign_up` method. Registers a new user in the
/// directory, creating a User Canister and mapping the caller's principal to the
/// chosen handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum SignUpError {
    /// The caller has already an account
    AlreadyRegistered,
    /// The chosen handle is already taken by another user
    HandleTaken,
    /// The chosen handle is invalid (e.g. empty or contains disallowed characters)
    InvalidHandle,
}

/// Response result type for the `sign_up` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum SignUpResponse {
    Ok,
    Err(SignUpError),
}

/// `who_am_i` method data to be returned in case the caller is registered in the directory.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct WhoAmI {
    /// The unique username (handle) of the caller.
    pub handle: String,
    /// The principal of the caller's User Canister.
    pub user_canister: candid::Principal,
}

/// Error types for the `who_am_i` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum WhoAmIError {
    /// The caller has no account in the directory.
    NotRegistered,
}

/// Response type for the `who_am_i` method, returning either the caller's identity
/// information or an error if they are not registered.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum WhoAmIResponse {
    Ok(WhoAmI),
    Err(WhoAmIError),
}

/// Error types for the `user_canister` method.
/// Resolves the caller's principal to their User Canister ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum UserCanisterError {
    /// The caller has no account in the directory.
    NotRegistered,
}

/// Response type for the `user_canister` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum UserCanisterResponse {
    Ok(candid::Principal),
    Err(UserCanisterError),
}

/// Request arguments for the `get_user` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct GetUserArgs {
    /// The handle to look up.
    pub handle: String,
}

/// Data returned by the `get_user` method on success.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct GetUser {
    /// The matched user's handle.
    pub handle: String,
    /// Principal of the looked-up user's canister.
    pub canister_id: candid::Principal,
}

/// Error types for the `get_user` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum GetUserError {
    /// No user exists with the given handle.
    NotFound,
}

/// Response type for the `get_user` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum GetUserResponse {
    Ok(GetUser),
    Err(GetUserError),
}

/// Request arguments for the `add_moderator` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct AddModeratorArgs {
    /// The principal to promote to moderator.
    pub principal: candid::Principal,
}

/// Error types for the `add_moderator` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum AddModeratorError {
    /// The caller is not a moderator.
    Unauthorized,
    /// The target principal is already a moderator.
    AlreadyModerator,
}

/// Response type for the `add_moderator` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum AddModeratorResponse {
    Ok,
    Err(AddModeratorError),
}

/// Request arguments for the `remove_moderator` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct RemoveModeratorArgs {
    /// The principal to demote.
    pub principal: candid::Principal,
}

/// Error types for the `remove_moderator` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum RemoveModeratorError {
    /// The caller is not a moderator.
    Unauthorized,
    /// The target principal is not currently a moderator.
    NotModerator,
}

/// Response type for the `remove_moderator` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum RemoveModeratorResponse {
    Ok,
    Err(RemoveModeratorError),
}

/// Request arguments for the `suspend` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct SuspendArgs {
    /// The principal of the user to suspend.
    pub principal: candid::Principal,
}

/// Error types for the `suspend` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum SuspendError {
    /// The caller is not a moderator.
    Unauthorized,
    /// No user exists with the given principal.
    NotFound,
}

/// Response type for the `suspend` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum SuspendResponse {
    Ok,
    Err(SuspendError),
}

/// Request arguments for the `search_profiles` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct SearchProfilesArgs {
    /// Free-text search string matched against handles.
    pub query: String,
    /// Number of results to skip (for pagination).
    pub offset: u64,
    /// Maximum number of results to return.
    pub limit: u64,
}

/// A single entry in the search results for the `search_profiles` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct SearchProfileEntry {
    /// The matched user's handle.
    pub handle: String,
    /// Principal of the matched user's canister.
    pub canister_id: candid::Principal,
}

/// Error types for the `search_profiles` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum SearchProfilesError {
    /// The caller is not permitted to search profiles.
    Unauthorized,
}

/// Response type for the `search_profiles` method.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum SearchProfilesResponse {
    Ok(Vec<SearchProfileEntry>),
    Err(SearchProfilesError),
}

/// Error types for the `delete_profile` method on the Directory Canister.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum DeleteProfileError {
    /// The caller has no account to delete.
    NotRegistered,
}

/// Response type for the `delete_profile` method on the Directory Canister.
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum DeleteProfileResponse {
    Ok,
    Err(DeleteProfileError),
}
