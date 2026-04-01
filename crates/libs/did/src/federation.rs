//! Type definitions for the Federation canister

#[cfg(test)]
mod tests;

use candid::CandidType;
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

/// Arguments for the `send_activity` method of the Federation canister.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct SendActivityArgs {
    /// JSON-encoded ActivityPub activity object to send.
    pub activity_json: String,
    /// URL of the remote actor's inbox to deliver the activity to.
    pub target_inbox: String,
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
