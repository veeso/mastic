//! Domain logic for the `follow_user` flow.
//!
//! The flow consists of:
//! 1. Validate the target handle is not the caller's own handle.
//! 2. Build the actor URI from the handle.
//! 3. Check the user is not already following the target.
//! 4. Insert a pending follow entry in the `following` table.
//! 5. Build a `Follow` activity and send it to the Federation Canister.

use activitypub::activity::{Activity, ActivityType};
use activitypub::context::ACTIVITY_STREAMS_CONTEXT;
use activitypub::object::{BaseObject, OneOrMany};
use did::federation::{SendActivityArgs, SendActivityArgsObject};
use did::user::{FollowUserArgs, FollowUserError, FollowUserResponse};

use super::FollowingRepository;
use crate::domain::profile::ProfileRepository;
use crate::error::{CanisterError, CanisterResult};

/// Execute the follow-user flow.
pub async fn follow_user(args: FollowUserArgs) -> FollowUserResponse {
    let handle = args.handle;

    ic_utils::log!("follow_user: attempting to follow @{handle}");

    match follow_user_inner(&handle).await {
        Ok(()) => FollowUserResponse::Ok,
        Err(FollowUserDomainError::CannotFollowSelf) => {
            FollowUserResponse::Err(FollowUserError::CannotFollowSelf)
        }
        Err(FollowUserDomainError::AlreadyFollowing) => {
            FollowUserResponse::Err(FollowUserError::AlreadyFollowing)
        }
        Err(FollowUserDomainError::Internal(e)) => {
            ic_utils::log!("follow_user: error: {e}");
            FollowUserResponse::Err(FollowUserError::Internal(e))
        }
    }
}

enum FollowUserDomainError {
    CannotFollowSelf,
    AlreadyFollowing,
    Internal(String),
}

impl From<CanisterError> for FollowUserDomainError {
    fn from(e: CanisterError) -> Self {
        Self::Internal(e.to_string())
    }
}

async fn follow_user_inner(handle: &str) -> Result<(), FollowUserDomainError> {
    // check if trying to follow self
    let own_profile = ProfileRepository::oneshot().get_profile()?;
    if handle == own_profile.handle.as_str() {
        ic_utils::log!("follow_user: cannot follow own handle");
        return Err(FollowUserDomainError::CannotFollowSelf);
    }

    // build actor URIs using centralized URL module
    let own_actor_uri = crate::domain::urls::actor_uri(&own_profile.handle.0)?;
    let target_actor_uri = crate::domain::urls::actor_uri(handle)?;

    // check if already following
    if FollowingRepository::find_by_actor_uri(&target_actor_uri)?.is_some() {
        ic_utils::log!("follow_user: already following {target_actor_uri}");
        return Err(FollowUserDomainError::AlreadyFollowing);
    }

    // insert pending follow and send activity
    FollowingRepository::insert_pending(&target_actor_uri)?;

    let activity = make_follow_activity(&own_actor_uri, &target_actor_uri)?;
    let args = SendActivityArgs::One(SendActivityArgsObject {
        activity_json: serde_json::to_string(&activity)
            .expect("Activity serialization must not fail"),
        target_inbox: crate::domain::urls::inbox_url(handle)?,
    });

    crate::adapters::federation::send_activity(args).await?;

    Ok(())
}

/// Build a `Follow` [`Activity`] targeting the given actor URI.
fn make_follow_activity(own_actor_uri: &str, target_actor_uri: &str) -> CanisterResult<Activity> {
    Ok(Activity {
        base: BaseObject {
            context: Some(activitypub::context::Context::Uri(
                ACTIVITY_STREAMS_CONTEXT.to_string(),
            )),
            kind: ActivityType::Follow,
            to: Some(OneOrMany::One(target_actor_uri.to_string())),
            ..Default::default()
        },
        actor: Some(own_actor_uri.to_string()),
        object: Some(activitypub::activity::ActivityObject::Id(
            target_actor_uri.to_string(),
        )),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    })
}

#[cfg(test)]
mod tests {

    use did::user::{FollowUserArgs, FollowUserError, FollowUserResponse};

    use super::*;
    use crate::schema::FollowStatus;
    use crate::test_utils::setup;

    #[tokio::test]
    async fn test_should_follow_user() {
        setup();

        let response = follow_user(FollowUserArgs {
            handle: "alice".to_string(),
        })
        .await;

        assert_eq!(response, FollowUserResponse::Ok);

        // verify the entry was stored as pending with proper actor URI
        let entry = FollowingRepository::find_by_actor_uri("https://mastic.social/users/alice")
            .expect("should query following")
            .expect("should find following entry");
        assert_eq!(entry.status, FollowStatus::Pending);
    }

    #[tokio::test]
    async fn test_should_reject_follow_self() {
        setup();

        // own handle is "rey_canisteryo" from test setup
        let response = follow_user(FollowUserArgs {
            handle: "rey_canisteryo".to_string(),
        })
        .await;

        assert_eq!(
            response,
            FollowUserResponse::Err(FollowUserError::CannotFollowSelf)
        );
    }

    #[tokio::test]
    async fn test_should_reject_already_following() {
        setup();

        let first = follow_user(FollowUserArgs {
            handle: "alice".to_string(),
        })
        .await;
        assert_eq!(first, FollowUserResponse::Ok);

        let second = follow_user(FollowUserArgs {
            handle: "alice".to_string(),
        })
        .await;
        assert_eq!(
            second,
            FollowUserResponse::Err(FollowUserError::AlreadyFollowing)
        );
    }

    #[test]
    fn test_make_follow_activity_should_build_follow() {
        setup();

        let own = "https://mastic.social/users/rey_canisteryo";
        let target = "https://mastic.social/users/bob";
        let activity = make_follow_activity(own, target).expect("should build activity");

        assert_eq!(activity.base.kind, ActivityType::Follow);
        assert_eq!(activity.actor.as_deref(), Some(own));
        assert_eq!(activity.base.to, Some(OneOrMany::One(target.to_string())));

        let activitypub::activity::ActivityObject::Id(obj_uri) =
            activity.object.expect("should have object")
        else {
            panic!("expected Id variant");
        };
        assert_eq!(obj_uri, target);
    }
}
