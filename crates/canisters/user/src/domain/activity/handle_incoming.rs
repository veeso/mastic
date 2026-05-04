//! Handle incoming activity flow

mod announce;
mod create;
mod delete;
mod follow;
mod like;
mod undo;

#[cfg(test)]
mod test_helpers;

use activitypub::Activity;
use activitypub::activity::{ActivityObject, ActivityType};
use db_utils::repository::Repository;
use did::user::{ReceiveActivityArgs, ReceiveActivityError, ReceiveActivityResponse};

/// Handles an incoming [`Activity`] from the federation canister.
///
/// Tries to decode the activity object from JSON into an [`Activity`] struct,
/// then it matches on the activity type and performs the appropriate action based on the type of activity received.
pub fn handle_incoming(
    ReceiveActivityArgs { activity_json }: ReceiveActivityArgs,
) -> ReceiveActivityResponse {
    // Try to decode the activity JSON into an Activity struct
    let Ok(activity) = serde_json::from_str::<Activity>(&activity_json) else {
        return ReceiveActivityResponse::Err(ReceiveActivityError::InvalidActivity);
    };

    let result = match activity.base.kind {
        ActivityType::Create => create::handle_create(&activity, &activity_json),
        ActivityType::Follow => follow::handle_follow(&activity),
        ActivityType::Accept => follow::handle_accept(&activity),
        ActivityType::Reject => follow::handle_reject(&activity),
        ActivityType::Like => like::handle_like(&activity),
        ActivityType::Announce => announce::handle_announce(&activity, &activity_json),
        ActivityType::Delete => delete::handle_delete(&activity),
        ActivityType::Undo => undo::handle_undo(&activity),
        other => {
            // Unknown / not-yet-implemented activity types are silently accepted.
            // ActivityPub receivers should not reject deliveries they can't act
            // on — unknown verbs are absorbed so the sender does not retry.
            ic_utils::log!("handle_incoming: ignoring unsupported activity type: {other:?}");
            Ok(())
        }
    };

    match result {
        Ok(()) => ReceiveActivityResponse::Ok,
        Err(e) => ReceiveActivityResponse::Err(e),
    }
}

/// Extract the object URI from an `Id`-form or `Object`-form `ActivityObject`.
pub(super) fn extract_object_uri(activity: &Activity) -> Option<String> {
    match activity.object.as_ref()? {
        ActivityObject::Id(uri) => Some(uri.clone()),
        ActivityObject::Object(obj) => obj.id.clone(),
        ActivityObject::Activity(_) | ActivityObject::Actor(_) => None,
    }
}

/// Confirm the status URI is hosted on this canister's instance and points
/// at this user's handle, returning `(handle, id)` when so.
pub(super) fn parse_local_status(
    status_uri: &str,
) -> Result<Option<(String, u64)>, ReceiveActivityError> {
    let parsed = crate::domain::urls::parse_local_status_uri(status_uri).map_err(|e| {
        ic_utils::log!("handle_incoming: failed to parse status URI: {e}");
        ReceiveActivityError::Internal(e.to_string())
    })?;
    let Some((handle, id)) = parsed else {
        return Ok(None);
    };

    let own = crate::domain::profile::ProfileRepository::oneshot()
        .get_profile()
        .map_err(|e| {
            ic_utils::log!("handle_incoming: failed to load own profile: {e}");
            ReceiveActivityError::Internal(e.to_string())
        })?;
    if handle != own.handle.0 {
        return Ok(None);
    }

    Ok(Some((handle, id)))
}

#[cfg(test)]
mod tests {

    use did::user::{ReceiveActivityArgs, ReceiveActivityError, ReceiveActivityResponse};

    use super::handle_incoming;
    use crate::test_utils::setup;

    #[test]
    fn test_should_return_invalid_activity_for_bad_json() {
        setup();

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: "not valid json".to_string(),
        });

        assert_eq!(
            response,
            ReceiveActivityResponse::Err(ReceiveActivityError::InvalidActivity)
        );
    }
}
