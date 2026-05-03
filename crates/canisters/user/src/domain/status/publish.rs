//! Logic for publishing a new status.

use activitypub::activity::{Activity, ActivityObject, ActivityType};
use activitypub::context::ACTIVITY_STREAMS_CONTEXT;
use activitypub::object::{BaseObject, ObjectType, OneOrMany};
use db_utils::transaction::Transaction;
use did::common::{Status, Visibility};
use did::federation::{SendActivityArgs, SendActivityArgsObject};
use did::user::{PublishStatusArgs, PublishStatusError, PublishStatusResponse};

use crate::domain::follower::FollowerRepository;
use crate::domain::snowflake::Snowflake;
use crate::error::{CanisterError, CanisterResult};
use crate::schema::{Schema, StatusContentSanitizer};

/// The ActivityStreams public addressing constant.
const AS_PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";

/// Publish a new status with the given content, visibility, and mentions.
///
/// The publish flow consists in:
///
/// 1. Sanitize and validate the content; reject empty, too-long, or
///    [`Visibility::Direct`] with no mentioned recipients.
/// 2. Create the status in the database and mint its snowflake ID.
/// 3. Compute the recipient list from the visibility:
///    - [`Visibility::Direct`]: the explicitly mentioned actors.
///    - All other visibilities: the author's followers.
/// 4. Emit one `Create(Note)` activity per recipient to the Federation
///    Canister as a single batch.
pub async fn publish_status(
    PublishStatusArgs {
        content,
        visibility,
        mentions,
    }: PublishStatusArgs,
) -> PublishStatusResponse {
    ic_utils::log!(
        "Publishing status with content: {content}, visibility: {visibility:?}, \
         mentions: {mentions:?}"
    );

    let content = StatusContentSanitizer::sanitize_content(&content);
    if content.is_empty() {
        ic_utils::log!("Status content is empty after sanitization");
        return PublishStatusResponse::Err(PublishStatusError::ContentEmpty);
    }
    if content.chars().count() > crate::domain::status::MAX_STATUS_LENGTH {
        ic_utils::log!("Status content exceeds max length");
        return PublishStatusResponse::Err(PublishStatusError::ContentTooLong);
    }
    if visibility == Visibility::Direct && mentions.is_empty() {
        ic_utils::log!("Direct status has no mentioned recipients");
        return PublishStatusResponse::Err(PublishStatusError::NoRecipients);
    }

    match save_status_and_publish_to_federation(content, visibility, mentions).await {
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
    mentions: Vec<String>,
) -> CanisterResult<Status> {
    // insert
    let created_at = ic_utils::now();
    let snowflake_id = Snowflake::new();
    Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
        crate::domain::status::create_status_with_feed(
            tx,
            snowflake_id.into(),
            content.clone(),
            visibility,
            created_at,
        )
    })?;
    ic_utils::log!("Status created with ID: {snowflake_id}");

    // build owner actor URI
    let own_profile = crate::domain::profile::ProfileRepository::oneshot().get_profile()?;
    let owner_actor_uri = crate::domain::urls::actor_uri(&own_profile.handle.0)?;

    // make status object
    let status = Status {
        id: snowflake_id.into(),
        content,
        author: owner_actor_uri,
        created_at,
        visibility,
        like_count: 0,
        boost_count: 0,
        spoiler_text: None,
        sensitive: false,
    };

    let recipients = match visibility {
        Visibility::Direct => mentions.clone(),
        Visibility::Public | Visibility::Unlisted | Visibility::FollowersOnly => {
            FollowerRepository::oneshot()
                .get_followers()?
                .into_iter()
                .map(|f| f.actor_uri.0)
                .collect()
        }
    };

    let activities: Vec<SendActivityArgsObject> = recipients
        .iter()
        .map(|recipient_uri| make_activity(recipient_uri, &status, &mentions))
        .collect();
    ic_utils::log!(
        "Prepared {len} activities for federation",
        len = activities.len()
    );

    // publish to federation
    let args = SendActivityArgs::Batch(activities);
    crate::adapters::federation::send_activity(args).await?;

    Ok(status)
}

/// Build a [`SendActivityArgsObject`] containing a `Create(Note)` activity
/// addressed to a single recipient actor.
///
/// The `target_inbox` is derived by appending `/inbox` to the recipient's
/// actor URI, following the standard ActivityPub convention. The activity's
/// `to`/`cc` fields are set per [`visibility_addressing`] and augmented
/// with the caller-supplied `mentions`.
fn make_activity(
    recipient_uri: &str,
    status: &Status,
    mentions: &[String],
) -> SendActivityArgsObject {
    let actor = status.author.clone();
    let (to, cc) = visibility_addressing(&status.visibility, mentions);

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
        target_inbox: format!("{recipient_uri}/inbox"),
    }
}

/// Map a [`Visibility`] value to the ActivityPub `to` and `cc` fields.
///
/// Returns `(to, cc)` as optional [`OneOrMany<String>`] values following
/// Mastodon's addressing conventions. `mentions` are added to the
/// appropriate field so the mentioned actors are notified:
///
/// - [`Visibility::Public`]: `to = [AS_PUBLIC]`, `cc = mentions`.
/// - [`Visibility::Unlisted`]: `to = mentions`, `cc = [AS_PUBLIC]`.
/// - [`Visibility::FollowersOnly`]: `to = mentions`, `cc = None`.
/// - [`Visibility::Direct`]: `to = mentions`, `cc = None`.
fn visibility_addressing(
    visibility: &Visibility,
    mentions: &[String],
) -> (Option<OneOrMany<String>>, Option<OneOrMany<String>>) {
    let mentions = vec_to_one_or_many(mentions);
    match visibility {
        Visibility::Public => (Some(OneOrMany::One(AS_PUBLIC.to_string())), mentions),
        Visibility::Unlisted => (mentions, Some(OneOrMany::One(AS_PUBLIC.to_string()))),
        Visibility::FollowersOnly | Visibility::Direct => (mentions, None),
    }
}

/// Convert a slice of strings into an optional [`OneOrMany`], returning
/// `None` when the slice is empty.
fn vec_to_one_or_many(items: &[String]) -> Option<OneOrMany<String>> {
    match items.len() {
        0 => None,
        1 => Some(OneOrMany::One(items[0].clone())),
        _ => Some(OneOrMany::Many(items.to_vec())),
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

    const BOB_URI: &str = "https://remote.example/users/bob";

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

    #[tokio::test]
    async fn test_should_publish_status_with_no_followers() {
        setup();

        let response = publish_status(PublishStatusArgs {
            content: "Hello, Fediverse!".to_string(),
            visibility: Visibility::Public,
            mentions: vec![],
        })
        .await;

        let PublishStatusResponse::Ok(status) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(status.content, "Hello, Fediverse!");
        assert_eq!(status.visibility, Visibility::Public);
        assert_eq!(status.author, "https://mastic.social/users/rey_canisteryo");
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
            mentions: vec![],
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
            mentions: vec![],
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
            mentions: vec![],
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
            mentions: vec![],
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
            mentions: vec![],
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
            mentions: vec![],
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
            let mentions = if vis == Visibility::Direct {
                vec![BOB_URI.to_string()]
            } else {
                vec![]
            };
            let response = publish_status(PublishStatusArgs {
                content: format!("Status with {vis:?}"),
                visibility: vis,
                mentions,
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
            mentions: vec![],
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
            mentions: vec![],
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
                mentions: vec![],
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

    fn public_status(visibility: Visibility) -> Status {
        Status {
            id: 1,
            content: format!("{visibility:?} post"),
            author: "https://mastic.social/users/rey_canisteryo".to_string(),
            created_at: 0,
            visibility,
            like_count: 0,
            boost_count: 0,
            spoiler_text: None,
            sensitive: false,
        }
    }

    #[test]
    fn test_make_activity_should_build_create_note() {
        let status = Status {
            id: 42,
            content: "Hello!".to_string(),
            author: "https://mastic.social/users/rey_canisteryo".to_string(),
            created_at: 1_000_000,
            visibility: Visibility::Public,
            like_count: 0,
            boost_count: 0,
            spoiler_text: None,
            sensitive: false,
        };

        let args = make_activity(BOB_URI, &status, &[]);

        assert_eq!(args.target_inbox, format!("{BOB_URI}/inbox"));

        let activity: activitypub::activity::Activity =
            serde_json::from_str(&args.activity_json).expect("should deserialize activity");
        assert_eq!(activity.base.kind, ActivityType::Create);
        assert_eq!(
            activity.actor.as_deref(),
            Some("https://mastic.social/users/rey_canisteryo")
        );

        let ActivityObject::Object(note) = activity.object.expect("should have object") else {
            panic!("expected Object variant");
        };
        assert_eq!(note.kind, ObjectType::Note);
        assert_eq!(note.content.as_deref(), Some("Hello!"));
        assert_eq!(note.id.as_deref(), Some("42"));
    }

    #[test]
    fn test_make_activity_should_set_public_addressing() {
        let args = make_activity(BOB_URI, &public_status(Visibility::Public), &[]);
        let activity: activitypub::activity::Activity =
            serde_json::from_str(&args.activity_json).expect("should deserialize");

        assert_eq!(
            activity.base.to,
            Some(OneOrMany::One(AS_PUBLIC.to_string()))
        );
        assert!(activity.base.cc.is_none());
    }

    #[test]
    fn test_make_activity_should_set_unlisted_addressing() {
        let args = make_activity(BOB_URI, &public_status(Visibility::Unlisted), &[]);
        let activity: activitypub::activity::Activity =
            serde_json::from_str(&args.activity_json).expect("should deserialize");

        assert!(activity.base.to.is_none());
        assert_eq!(
            activity.base.cc,
            Some(OneOrMany::One(AS_PUBLIC.to_string()))
        );
    }

    #[test]
    fn test_make_activity_should_set_followers_only_addressing() {
        let args = make_activity(BOB_URI, &public_status(Visibility::FollowersOnly), &[]);
        let activity: activitypub::activity::Activity =
            serde_json::from_str(&args.activity_json).expect("should deserialize");

        assert!(activity.base.to.is_none());
        assert!(activity.base.cc.is_none());
    }

    #[test]
    fn test_make_activity_should_set_direct_addressing_with_mentions() {
        let args = make_activity(
            BOB_URI,
            &public_status(Visibility::Direct),
            &[BOB_URI.to_string()],
        );
        let activity: activitypub::activity::Activity =
            serde_json::from_str(&args.activity_json).expect("should deserialize");

        assert_eq!(activity.base.to, Some(OneOrMany::One(BOB_URI.to_string())));
        assert!(activity.base.cc.is_none());
    }

    #[test]
    fn test_make_activity_should_include_mentions_in_cc_for_public() {
        let mentions = vec![BOB_URI.to_string()];
        let args = make_activity(BOB_URI, &public_status(Visibility::Public), &mentions);
        let activity: activitypub::activity::Activity =
            serde_json::from_str(&args.activity_json).expect("should deserialize");

        assert_eq!(
            activity.base.to,
            Some(OneOrMany::One(AS_PUBLIC.to_string()))
        );
        assert_eq!(activity.base.cc, Some(OneOrMany::One(BOB_URI.to_string())));
    }

    #[test]
    fn test_visibility_addressing_public_no_mentions() {
        let (to, cc) = visibility_addressing(&Visibility::Public, &[]);
        assert_eq!(to, Some(OneOrMany::One(AS_PUBLIC.to_string())));
        assert!(cc.is_none());
    }

    #[test]
    fn test_visibility_addressing_unlisted_no_mentions() {
        let (to, cc) = visibility_addressing(&Visibility::Unlisted, &[]);
        assert!(to.is_none());
        assert_eq!(cc, Some(OneOrMany::One(AS_PUBLIC.to_string())));
    }

    #[test]
    fn test_visibility_addressing_followers_only_no_mentions() {
        let (to, cc) = visibility_addressing(&Visibility::FollowersOnly, &[]);
        assert!(to.is_none());
        assert!(cc.is_none());
    }

    #[test]
    fn test_visibility_addressing_direct_with_mentions() {
        let mentions = vec![BOB_URI.to_string()];
        let (to, cc) = visibility_addressing(&Visibility::Direct, &mentions);
        assert_eq!(to, Some(OneOrMany::One(BOB_URI.to_string())));
        assert!(cc.is_none());
    }

    #[test]
    fn test_visibility_addressing_direct_with_multiple_mentions() {
        let mentions = vec![
            BOB_URI.to_string(),
            "https://remote.example/users/carol".to_string(),
        ];
        let (to, cc) = visibility_addressing(&Visibility::Direct, &mentions);
        assert_eq!(to, Some(OneOrMany::Many(mentions)));
        assert!(cc.is_none());
    }

    #[tokio::test]
    async fn test_should_reject_direct_without_mentions() {
        setup();

        let response = publish_status(PublishStatusArgs {
            content: "Secret".to_string(),
            visibility: Visibility::Direct,
            mentions: vec![],
        })
        .await;

        assert_eq!(
            response,
            PublishStatusResponse::Err(PublishStatusError::NoRecipients),
        );
        assert_eq!(count_statuses(), 0);
    }

    #[tokio::test]
    async fn test_should_publish_direct_with_mentions() {
        setup();
        insert_follower("https://remote.example/users/carol");

        let response = publish_status(PublishStatusArgs {
            content: "Hi Bob".to_string(),
            visibility: Visibility::Direct,
            mentions: vec![BOB_URI.to_string()],
        })
        .await;

        let PublishStatusResponse::Ok(status) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(status.visibility, Visibility::Direct);
        assert_eq!(count_statuses(), 1);
    }
}
