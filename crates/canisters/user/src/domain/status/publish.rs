//! Logic for publishing a new status.

use activitypub::activity::{Activity, ActivityObject, ActivityType};
use activitypub::context::ACTIVITY_STREAMS_CONTEXT;
use activitypub::object::{BaseObject, ObjectType, OneOrMany};
use did::common::{Status, Visibility};
use did::federation::{SendActivityArgs, SendActivityArgsObject};
use did::user::{PublishStatusArgs, PublishStatusError, PublishStatusResponse};

use crate::domain::follower::FollowerRepository;
use crate::domain::status::StatusRepository;
use crate::error::CanisterResult;
use crate::schema::{Follower, StatusContentSanitizer};

/// The ActivityStreams public addressing constant.
const AS_PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";

/// Publish a new status with the given content and visibility.
///
/// The publish flow consists in:
///
/// 1. Sanitize and validate the content
/// 2. Create a new status in the database with the given content and visibility, and get the generated ID.
/// 3. Publish a new activity to federation, for each follower of the user, with a `Create(Note)` activity.
pub async fn publish_status(
    PublishStatusArgs {
        content,
        visibility,
    }: PublishStatusArgs,
) -> PublishStatusResponse {
    ic_utils::log!("Publishing status with content: {content} and visibility: {visibility:?}");

    let content = StatusContentSanitizer::sanitize_content(&content);
    if content.is_empty() {
        ic_utils::log!("Status content is empty after sanitization");
        return PublishStatusResponse::Err(PublishStatusError::ContentEmpty);
    }
    if content.chars().count() > crate::domain::status::MAX_STATUS_LENGTH {
        ic_utils::log!("Status content exceeds max length");
        return PublishStatusResponse::Err(PublishStatusError::ContentTooLong);
    }

    match save_status_and_publish_to_federation(content, visibility).await {
        Ok(status) => PublishStatusResponse::Ok(status),
        Err(e) => {
            ic_utils::log!("Error publishing status: {e}");
            PublishStatusResponse::Err(PublishStatusError::Internal(e.to_string()))
        }
    }
}

/// Internal helper function to save the status in the database and publish the corresponding activity to federation.
///
/// This is used because by returning result we can short-circuit with `?`
async fn save_status_and_publish_to_federation(
    content: String,
    visibility: Visibility,
) -> CanisterResult<Status> {
    // get all followers
    let followers = FollowerRepository::get_followers()?;

    // insert
    let created_at = ic_utils::now();
    let snowflake_id = StatusRepository::create(content.clone(), visibility, created_at)?;
    ic_utils::log!("Status created with ID: {snowflake_id}");

    // make status object
    let status = Status {
        id: snowflake_id.into(),
        content,
        author: ic_utils::caller(),
        created_at,
        visibility,
    };

    let mut activities = Vec::with_capacity(followers.len());
    for follower in followers {
        activities.push(make_follower_activity(&follower, &status));
    }
    ic_utils::log!(
        "Prepared {len} activities for federation",
        len = activities.len()
    );

    // publish to federation
    let _args = SendActivityArgs::Batch(activities);
    #[cfg(test)]
    {
        use crate::adapters::federation::FederationCanister as _;
        crate::adapters::federation::mock::FederationCanisterMockClient
            .send_activity(_args)
            .await?;
    }
    #[cfg(target_family = "wasm")]
    {
        use crate::adapters::federation::FederationCanister as _;
        let federation_canister = crate::settings::get_federation_canister()?;
        crate::adapters::federation::IcFederationCanisterClient::from(federation_canister)
            .send_activity(_args)
            .await?;
    }

    Ok(status)
}

/// Build a [`SendActivityArgsObject`] containing a `Create(Note)` activity
/// addressed to a single follower.
///
/// The `target_inbox` is derived by appending `/inbox` to the follower's
/// actor URI, following the standard ActivityPub convention.
fn make_follower_activity(follower: &Follower, status: &Status) -> SendActivityArgsObject {
    let actor = status.author.to_text();
    let (to, cc) = visibility_addressing(&status.visibility);

    let note = BaseObject {
        id: Some(status.id.to_string()),
        kind: ObjectType::Note,
        content: Some(status.content.clone()),
        to: to.clone(),
        cc: cc.clone(),
        attributed_to: Some(OneOrMany::One(actor.clone())),
        ..Default::default()
    };

    let activity = Activity {
        base: BaseObject {
            context: Some(activitypub::context::Context::Uri(
                ACTIVITY_STREAMS_CONTEXT.to_string(),
            )),
            kind: ActivityType::Create,
            to,
            cc,
            ..Default::default()
        },
        actor: Some(actor),
        object: Some(ActivityObject::Object(Box::new(note))),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    };

    let activity_json =
        serde_json::to_string(&activity).expect("Activity serialization must not fail");

    SendActivityArgsObject {
        activity_json,
        target_inbox: format!("{actor_uri}/inbox", actor_uri = follower.actor_uri),
    }
}

/// Map a [`Visibility`] value to the ActivityPub `to` and `cc` fields.
///
/// Returns `(to, cc)` as optional [`OneOrMany<String>`] values following
/// Mastodon's addressing conventions.
fn visibility_addressing(
    visibility: &Visibility,
) -> (Option<OneOrMany<String>>, Option<OneOrMany<String>>) {
    match visibility {
        Visibility::Public => (Some(OneOrMany::One(AS_PUBLIC.to_string())), None),
        Visibility::Unlisted => (None, Some(OneOrMany::One(AS_PUBLIC.to_string()))),
        Visibility::FollowersOnly | Visibility::Direct => (None, None),
    }
}

#[cfg(test)]
mod tests {

    use activitypub::activity::{ActivityObject, ActivityType};
    use activitypub::object::{ObjectType, OneOrMany};
    use did::common::{Status, Visibility};
    use did::user::{PublishStatusArgs, PublishStatusError, PublishStatusResponse};
    use ic_dbms_canister::prelude::DBMS_CONTEXT;
    use wasm_dbms::WasmDbmsDatabase;
    use wasm_dbms_api::prelude::{Database, Query};

    use super::*;
    use crate::schema::{Follower, FollowerInsertRequest, Schema};
    use crate::test_utils::setup;

    /// Helper to insert a follower into the database for testing.
    fn insert_follower(actor_uri: &str) {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Follower>(FollowerInsertRequest {
                actor_uri: actor_uri.into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert follower");
        });
    }

    /// Helper to count statuses stored in the database.
    fn count_statuses() -> usize {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.select::<crate::schema::Status>(Query::builder().all().build())
                .expect("should select statuses")
                .len()
        })
    }

    // ── publish_status ──────────────────────────────────────────────

    #[tokio::test]
    async fn test_should_publish_status_with_no_followers() {
        setup();

        let response = publish_status(PublishStatusArgs {
            content: "Hello, Fediverse!".to_string(),
            visibility: Visibility::Public,
        })
        .await;

        let PublishStatusResponse::Ok(status) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(status.content, "Hello, Fediverse!");
        assert_eq!(status.visibility, Visibility::Public);
        assert_eq!(status.author, ic_utils::caller());
        assert!(status.id > 0);
        assert_eq!(count_statuses(), 1);
    }

    #[tokio::test]
    async fn test_should_publish_status_with_followers() {
        setup();
        insert_follower("https://remote.example/users/bob");
        insert_follower("https://remote.example/users/carol");

        let response = publish_status(PublishStatusArgs {
            content: "Status with followers".to_string(),
            visibility: Visibility::Public,
        })
        .await;

        let PublishStatusResponse::Ok(status) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(status.content, "Status with followers");
        assert_eq!(count_statuses(), 1);
    }

    #[tokio::test]
    async fn test_should_reject_empty_content() {
        setup();

        let response = publish_status(PublishStatusArgs {
            content: String::new(),
            visibility: Visibility::Public,
        })
        .await;

        assert_eq!(
            response,
            PublishStatusResponse::Err(PublishStatusError::ContentEmpty),
        );
        assert_eq!(count_statuses(), 0);
    }

    #[tokio::test]
    async fn test_should_reject_whitespace_only_content() {
        setup();

        let response = publish_status(PublishStatusArgs {
            content: "   \t\n  ".to_string(),
            visibility: Visibility::Public,
        })
        .await;

        assert_eq!(
            response,
            PublishStatusResponse::Err(PublishStatusError::ContentEmpty),
        );
        assert_eq!(count_statuses(), 0);
    }

    #[tokio::test]
    async fn test_should_reject_content_exceeding_max_length() {
        setup();

        let long_content = "a".repeat(crate::domain::status::MAX_STATUS_LENGTH + 1);
        let response = publish_status(PublishStatusArgs {
            content: long_content,
            visibility: Visibility::Public,
        })
        .await;

        assert_eq!(
            response,
            PublishStatusResponse::Err(PublishStatusError::ContentTooLong),
        );
        assert_eq!(count_statuses(), 0);
    }

    #[tokio::test]
    async fn test_should_accept_content_at_max_length() {
        setup();

        let exact_content = "a".repeat(crate::domain::status::MAX_STATUS_LENGTH);
        let response = publish_status(PublishStatusArgs {
            content: exact_content.clone(),
            visibility: Visibility::Public,
        })
        .await;

        let PublishStatusResponse::Ok(status) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(status.content, exact_content);
    }

    #[tokio::test]
    async fn test_should_trim_whitespace_from_content() {
        setup();

        let response = publish_status(PublishStatusArgs {
            content: "  Hello, world!  ".to_string(),
            visibility: Visibility::Public,
        })
        .await;

        let PublishStatusResponse::Ok(status) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(status.content, "Hello, world!");
    }

    #[tokio::test]
    async fn test_should_publish_with_all_visibility_levels() {
        setup();

        let visibilities = [
            Visibility::Public,
            Visibility::Unlisted,
            Visibility::FollowersOnly,
            Visibility::Direct,
        ];

        for vis in visibilities {
            let response = publish_status(PublishStatusArgs {
                content: format!("Status with {vis:?}"),
                visibility: vis,
            })
            .await;

            let PublishStatusResponse::Ok(status) = response else {
                panic!("expected Ok for {vis:?}, got {response:?}");
            };
            assert_eq!(status.visibility, vis);
        }

        assert_eq!(count_statuses(), 4);
    }

    #[tokio::test]
    async fn test_should_accept_multibyte_content_at_max_char_length() {
        setup();

        // 500 emoji characters — each is 4 bytes, so byte length would be 2000
        let emoji_content = "😀".repeat(crate::domain::status::MAX_STATUS_LENGTH);
        let response = publish_status(PublishStatusArgs {
            content: emoji_content.clone(),
            visibility: Visibility::Public,
        })
        .await;

        let PublishStatusResponse::Ok(status) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(status.content, emoji_content);
    }

    #[tokio::test]
    async fn test_should_reject_multibyte_content_exceeding_max_char_length() {
        setup();

        let emoji_content = "😀".repeat(crate::domain::status::MAX_STATUS_LENGTH + 1);
        let response = publish_status(PublishStatusArgs {
            content: emoji_content,
            visibility: Visibility::Public,
        })
        .await;

        assert_eq!(
            response,
            PublishStatusResponse::Err(PublishStatusError::ContentTooLong),
        );
        assert_eq!(count_statuses(), 0);
    }

    #[tokio::test]
    async fn test_should_assign_unique_snowflake_ids() {
        setup();

        let mut ids = Vec::new();
        for i in 0..5 {
            let response = publish_status(PublishStatusArgs {
                content: format!("Status {i}"),
                visibility: Visibility::Public,
            })
            .await;

            let PublishStatusResponse::Ok(status) = response else {
                panic!("expected Ok, got {response:?}");
            };
            ids.push(status.id);
        }

        // all IDs must be unique
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), ids.len(), "snowflake IDs must be unique");

        // IDs should be monotonically increasing
        for window in ids.windows(2) {
            assert!(
                window[0] < window[1],
                "IDs must be monotonically increasing"
            );
        }
    }

    // ── make_follower_activity ──────────────────────────────────────

    #[test]
    fn test_make_follower_activity_should_build_create_note() {
        let follower = crate::schema::Follower {
            actor_uri: "https://remote.example/users/bob".to_string().into(),
            created_at: 0u64.into(),
        };
        let status = Status {
            id: 42,
            content: "Hello!".to_string(),
            author: ic_utils::caller(),
            created_at: 1_000_000,
            visibility: Visibility::Public,
        };

        let args = make_follower_activity(&follower, &status);

        assert_eq!(args.target_inbox, "https://remote.example/users/bob/inbox");

        let activity: activitypub::activity::Activity =
            serde_json::from_str(&args.activity_json).expect("should deserialize activity");
        assert_eq!(activity.base.kind, ActivityType::Create);
        assert_eq!(
            activity.actor.as_deref(),
            Some(ic_utils::caller().to_text().as_str())
        );

        let ActivityObject::Object(note) = activity.object.expect("should have object") else {
            panic!("expected Object variant");
        };
        assert_eq!(note.kind, ObjectType::Note);
        assert_eq!(note.content.as_deref(), Some("Hello!"));
        assert_eq!(note.id.as_deref(), Some("42"));
    }

    #[test]
    fn test_make_follower_activity_should_set_public_addressing() {
        let follower = crate::schema::Follower {
            actor_uri: "https://remote.example/users/bob".to_string().into(),
            created_at: 0u64.into(),
        };
        let status = Status {
            id: 1,
            content: "Public post".to_string(),
            author: ic_utils::caller(),
            created_at: 0,
            visibility: Visibility::Public,
        };

        let args = make_follower_activity(&follower, &status);
        let activity: activitypub::activity::Activity =
            serde_json::from_str(&args.activity_json).expect("should deserialize");

        assert_eq!(
            activity.base.to,
            Some(OneOrMany::One(AS_PUBLIC.to_string()))
        );
        assert!(activity.base.cc.is_none());
    }

    #[test]
    fn test_make_follower_activity_should_set_unlisted_addressing() {
        let follower = crate::schema::Follower {
            actor_uri: "https://remote.example/users/bob".to_string().into(),
            created_at: 0u64.into(),
        };
        let status = Status {
            id: 1,
            content: "Unlisted post".to_string(),
            author: ic_utils::caller(),
            created_at: 0,
            visibility: Visibility::Unlisted,
        };

        let args = make_follower_activity(&follower, &status);
        let activity: activitypub::activity::Activity =
            serde_json::from_str(&args.activity_json).expect("should deserialize");

        assert!(activity.base.to.is_none());
        assert_eq!(
            activity.base.cc,
            Some(OneOrMany::One(AS_PUBLIC.to_string()))
        );
    }

    #[test]
    fn test_make_follower_activity_should_set_followers_only_addressing() {
        let follower = crate::schema::Follower {
            actor_uri: "https://remote.example/users/bob".to_string().into(),
            created_at: 0u64.into(),
        };
        let status = Status {
            id: 1,
            content: "Followers only post".to_string(),
            author: ic_utils::caller(),
            created_at: 0,
            visibility: Visibility::FollowersOnly,
        };

        let args = make_follower_activity(&follower, &status);
        let activity: activitypub::activity::Activity =
            serde_json::from_str(&args.activity_json).expect("should deserialize");

        assert!(activity.base.to.is_none());
        assert!(activity.base.cc.is_none());
    }

    #[test]
    fn test_make_follower_activity_should_set_direct_addressing() {
        let follower = crate::schema::Follower {
            actor_uri: "https://remote.example/users/bob".to_string().into(),
            created_at: 0u64.into(),
        };
        let status = Status {
            id: 1,
            content: "Direct post".to_string(),
            author: ic_utils::caller(),
            created_at: 0,
            visibility: Visibility::Direct,
        };

        let args = make_follower_activity(&follower, &status);
        let activity: activitypub::activity::Activity =
            serde_json::from_str(&args.activity_json).expect("should deserialize");

        assert!(activity.base.to.is_none());
        assert!(activity.base.cc.is_none());
    }

    // ── visibility_addressing ───────────────────────────────────────

    #[test]
    fn test_visibility_addressing_public() {
        let (to, cc) = visibility_addressing(&Visibility::Public);
        assert_eq!(to, Some(OneOrMany::One(AS_PUBLIC.to_string())));
        assert!(cc.is_none());
    }

    #[test]
    fn test_visibility_addressing_unlisted() {
        let (to, cc) = visibility_addressing(&Visibility::Unlisted);
        assert!(to.is_none());
        assert_eq!(cc, Some(OneOrMany::One(AS_PUBLIC.to_string())));
    }

    #[test]
    fn test_visibility_addressing_followers_only() {
        let (to, cc) = visibility_addressing(&Visibility::FollowersOnly);
        assert!(to.is_none());
        assert!(cc.is_none());
    }

    #[test]
    fn test_visibility_addressing_direct() {
        let (to, cc) = visibility_addressing(&Visibility::Direct);
        assert!(to.is_none());
        assert!(cc.is_none());
    }
}
