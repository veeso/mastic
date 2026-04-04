//! Handle incoming activity flow

use activitypub::Activity;
use did::user::{ReceiveActivityArgs, ReceiveActivityError, ReceiveActivityResponse};

/// Handles an incoming [`Activity`] from the federation canister.
///
/// Tries to decode the activity object from JSON into an [`Activity`] struct,
/// then it matches on the activity type and performs the appropriate action based on the type of activity received.
pub fn handle_incoming(
    ReceiveActivityArgs { activity_json }: ReceiveActivityArgs,
) -> ReceiveActivityResponse {
    // Try to decode the activity JSON into an Activity struct
    let Ok(_activity) = serde_json::from_str::<Activity>(&activity_json) else {
        return ReceiveActivityResponse::Err(ReceiveActivityError::InvalidActivity);
    };

    ReceiveActivityResponse::Ok
}
