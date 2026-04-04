//! Handle incoming activity flow

use activitypub::Activity;
use activitypub::activity::{ActivityObject, ActivityType};
use did::user::{ReceiveActivityArgs, ReceiveActivityError, ReceiveActivityResponse};

use crate::domain::follow_request::FollowRequestRepository;
use crate::domain::following::FollowingRepository;
use crate::error::CanisterError;
use crate::schema::FollowStatus;

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
        ActivityType::Follow => handle_follow(&activity),
        ActivityType::Accept => handle_accept_or_reject(&activity, FollowStatus::Accepted),
        ActivityType::Reject => handle_accept_or_reject(&activity, FollowStatus::Rejected),
        other => {
            ic_utils::log!("handle_incoming: unsupported activity type: {other:?}");
            Err(ReceiveActivityError::ProcessingFailed)
        }
    };

    match result {
        Ok(()) => ReceiveActivityResponse::Ok,
        Err(e) => ReceiveActivityResponse::Err(e),
    }
}

/// Handle an incoming `Follow` activity.
///
/// Extracts the actor URI (the follower) and stores a pending follow request.
fn handle_follow(activity: &Activity) -> Result<(), ReceiveActivityError> {
    let actor_uri = activity
        .actor
        .as_deref()
        .ok_or(ReceiveActivityError::ProcessingFailed)?;

    ic_utils::log!("handle_incoming: Follow from {actor_uri}");

    FollowRequestRepository::insert(actor_uri).map_err(|e| {
        ic_utils::log!("handle_incoming: failed to insert follow request: {e}");
        ReceiveActivityError::Internal(e.to_string())
    })
}

/// Handle an incoming `Accept(Follow)` or `Reject(Follow)` activity.
///
/// Extracts the actor URI of the remote user (from `activity.actor`) who accepted/rejected,
/// finds the pending entry in the `following` table, and updates its status.
fn handle_accept_or_reject(
    activity: &Activity,
    new_status: FollowStatus,
) -> Result<(), ReceiveActivityError> {
    // The actor who sent Accept/Reject is the user we were trying to follow
    let remote_actor_uri = activity
        .actor
        .as_deref()
        .ok_or(ReceiveActivityError::ProcessingFailed)?;

    // Validate that the inner object is a Follow activity
    let Some(ActivityObject::Activity(inner)) = &activity.object else {
        ic_utils::log!("handle_incoming: Accept/Reject missing inner Follow activity");
        return Err(ReceiveActivityError::ProcessingFailed);
    };

    if inner.base.kind != ActivityType::Follow {
        ic_utils::log!(
            "handle_incoming: Accept/Reject inner activity is not Follow: {:?}",
            inner.base.kind
        );
        return Err(ReceiveActivityError::ProcessingFailed);
    }

    ic_utils::log!(
        "handle_incoming: updating following status for {remote_actor_uri} to {new_status}"
    );

    FollowingRepository::update_status(remote_actor_uri, new_status).map_err(|e| {
        ic_utils::log!("handle_incoming: failed to update following status: {e}");
        match e {
            CanisterError::Database(_) => ReceiveActivityError::ProcessingFailed,
            _ => ReceiveActivityError::Internal(e.to_string()),
        }
    })
}

#[cfg(test)]
mod tests {

    use activitypub::activity::{Activity, ActivityObject, ActivityType};
    use activitypub::object::BaseObject;
    use did::user::{ReceiveActivityArgs, ReceiveActivityError, ReceiveActivityResponse};

    use super::handle_incoming;
    use crate::domain::follow_request::FollowRequestRepository;
    use crate::domain::following::FollowingRepository;
    use crate::schema::FollowStatus;
    use crate::test_utils::setup;

    fn make_follow_json(follower_actor_uri: &str, target_actor_uri: &str) -> String {
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Follow,
                ..Default::default()
            },
            actor: Some(follower_actor_uri.to_string()),
            object: Some(ActivityObject::Id(target_actor_uri.to_string())),
            target: None,
            result: None,
            origin: None,
            instrument: None,
        };
        serde_json::to_string(&activity).unwrap()
    }

    fn make_accept_follow_json(acceptor_actor_uri: &str, follower_actor_uri: &str) -> String {
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Accept,
                ..Default::default()
            },
            actor: Some(acceptor_actor_uri.to_string()),
            object: Some(ActivityObject::Activity(Box::new(Activity {
                base: BaseObject {
                    kind: ActivityType::Follow,
                    ..Default::default()
                },
                actor: Some(follower_actor_uri.to_string()),
                object: Some(ActivityObject::Id(acceptor_actor_uri.to_string())),
                target: None,
                result: None,
                origin: None,
                instrument: None,
            }))),
            target: None,
            result: None,
            origin: None,
            instrument: None,
        };
        serde_json::to_string(&activity).unwrap()
    }

    fn make_reject_follow_json(rejector_actor_uri: &str, follower_actor_uri: &str) -> String {
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Reject,
                ..Default::default()
            },
            actor: Some(rejector_actor_uri.to_string()),
            object: Some(ActivityObject::Activity(Box::new(Activity {
                base: BaseObject {
                    kind: ActivityType::Follow,
                    ..Default::default()
                },
                actor: Some(follower_actor_uri.to_string()),
                object: Some(ActivityObject::Id(rejector_actor_uri.to_string())),
                target: None,
                result: None,
                origin: None,
                instrument: None,
            }))),
            target: None,
            result: None,
            origin: None,
            instrument: None,
        };
        serde_json::to_string(&activity).unwrap()
    }

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

    #[test]
    fn test_should_store_follow_as_pending_request() {
        setup();

        let json = make_follow_json(
            "https://mastic.social/users/alice",
            "https://mastic.social/users/rey_canisteryo",
        );

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });

        assert_eq!(response, ReceiveActivityResponse::Ok);

        let request =
            FollowRequestRepository::find_by_actor_uri("https://mastic.social/users/alice")
                .expect("should query")
                .expect("should find follow request");
        assert_eq!(request.actor_uri.0, "https://mastic.social/users/alice");
    }

    #[test]
    fn test_should_update_following_to_accepted_on_accept() {
        setup();

        // first, create a pending following entry (simulates follow_user having run)
        FollowingRepository::insert_pending("https://mastic.social/users/bob")
            .expect("should insert pending");

        let json = make_accept_follow_json(
            "https://mastic.social/users/bob",
            "https://mastic.social/users/rey_canisteryo",
        );

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });

        assert_eq!(response, ReceiveActivityResponse::Ok);

        let entry = FollowingRepository::find_by_actor_uri("https://mastic.social/users/bob")
            .expect("should query")
            .expect("should find following entry");
        assert_eq!(entry.status, FollowStatus::Accepted);
    }

    #[test]
    fn test_should_update_following_to_rejected_on_reject() {
        setup();

        // first, create a pending following entry
        FollowingRepository::insert_pending("https://mastic.social/users/bob")
            .expect("should insert pending");

        let json = make_reject_follow_json(
            "https://mastic.social/users/bob",
            "https://mastic.social/users/rey_canisteryo",
        );

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });

        assert_eq!(response, ReceiveActivityResponse::Ok);

        let entry = FollowingRepository::find_by_actor_uri("https://mastic.social/users/bob")
            .expect("should query")
            .expect("should find following entry");
        assert_eq!(entry.status, FollowStatus::Rejected);
    }

    #[test]
    fn test_should_fail_accept_when_no_pending_following() {
        setup();

        let json = make_accept_follow_json(
            "https://mastic.social/users/bob",
            "https://mastic.social/users/rey_canisteryo",
        );

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });

        assert_eq!(
            response,
            ReceiveActivityResponse::Err(ReceiveActivityError::ProcessingFailed)
        );
    }

    #[test]
    fn test_should_fail_reject_when_no_pending_following() {
        setup();

        let json = make_reject_follow_json(
            "https://mastic.social/users/bob",
            "https://mastic.social/users/rey_canisteryo",
        );

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });

        assert_eq!(
            response,
            ReceiveActivityResponse::Err(ReceiveActivityError::ProcessingFailed)
        );
    }

    #[test]
    fn test_should_fail_accept_when_missing_inner_activity() {
        setup();

        // Accept with a plain URI object instead of nested Follow activity
        let activity = Activity {
            base: BaseObject {
                kind: ActivityType::Accept,
                ..Default::default()
            },
            actor: Some("https://mastic.social/users/bob".to_string()),
            object: Some(ActivityObject::Id(
                "https://mastic.social/users/bob".to_string(),
            )),
            target: None,
            result: None,
            origin: None,
            instrument: None,
        };
        let json = serde_json::to_string(&activity).unwrap();

        let response = handle_incoming(ReceiveActivityArgs {
            activity_json: json,
        });

        assert_eq!(
            response,
            ReceiveActivityResponse::Err(ReceiveActivityError::ProcessingFailed)
        );
    }
}
