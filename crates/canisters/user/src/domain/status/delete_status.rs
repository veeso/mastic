//! Delete status domain logic.
//!
//! Pipeline:
//!
//! 1. Parse the snowflake id from `status_uri`. Reject [`DeleteStatusError::InvalidUri`]
//!    when the URI does not match the canonical `…/statuses/{id}` shape.
//! 2. Resolve the row in `statuses`. Missing → [`DeleteStatusError::NotFound`].
//! 3. Cascade delete inside a single transaction:
//!    - `feed` row sharing the same id (no FK on it).
//!    - `boosts` row sharing the same id (own-boost wrapper).
//!    - `liked` row referencing the URI (defensive — own statuses aren't
//!      normally liked by self, but the URI is the only column we can clean).
//!    - `statuses` row with [`DeleteBehavior::Cascade`], which propagates to
//!      `media`, `edit_history`, `status_hashtags` and `pinned_statuses` via FK.
//! 4. Build a `Delete(Note)` activity addressed to followers (minus blocked
//!    actors) and dispatch a per-recipient batch through the federation
//!    adapter.

use activitypub::activity::{Activity, ActivityObject, ActivityType};
use activitypub::context::{ACTIVITY_STREAMS_CONTEXT, Context};
use activitypub::object::{BaseObject, OneOrMany};
use db_utils::repository::Repository;
use db_utils::transaction::Transaction;
use did::federation::{SendActivityArgs, SendActivityArgsObject};
use did::user::{DeleteStatusArgs, DeleteStatusError, DeleteStatusResponse};

use crate::adapters::federation;
use crate::domain::block::BlockRepository;
use crate::domain::boost::BoostRepository;
use crate::domain::feed::FeedRepository;
use crate::domain::follower::FollowerRepository;
use crate::domain::liked::LikedRepository;
use crate::domain::profile::ProfileRepository;
use crate::domain::status::StatusRepository;
use crate::domain::urls;
use crate::error::{CanisterError, CanisterResult};
use crate::schema::Schema;

const AS_PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";

/// Delete a local status owned by this canister and emit a `Delete(Note)`
/// activity to followers.
pub async fn delete_status(
    DeleteStatusArgs { status_uri }: DeleteStatusArgs,
) -> DeleteStatusResponse {
    ic_utils::log!("delete_status: {status_uri}");

    let Some(status_id) = urls::parse_status_id(&status_uri) else {
        ic_utils::log!("delete_status: invalid status URI: {status_uri}");
        return DeleteStatusResponse::Err(DeleteStatusError::InvalidUri);
    };

    match StatusRepository::oneshot().find_by_id(status_id) {
        Ok(Some(_)) => {}
        Ok(None) => {
            ic_utils::log!("delete_status: status {status_id} not found");
            return DeleteStatusResponse::Err(DeleteStatusError::NotFound);
        }
        Err(err) => {
            ic_utils::log!("delete_status: lookup failed: {err}");
            return DeleteStatusResponse::Err(DeleteStatusError::Internal(err.to_string()));
        }
    }

    if let Err(err) = cascade_delete(status_id, &status_uri) {
        ic_utils::log!("delete_status: cascade delete failed: {err}");
        return DeleteStatusResponse::Err(DeleteStatusError::Internal(err.to_string()));
    }

    if let Err(err) = dispatch_delete(&status_uri).await {
        ic_utils::log!("delete_status: dispatch failed: {err}");
        return DeleteStatusResponse::Err(DeleteStatusError::Internal(err.to_string()));
    }

    DeleteStatusResponse::Ok
}

fn cascade_delete(status_id: u64, status_uri: &str) -> CanisterResult<()> {
    Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
        FeedRepository::with_transaction(tx).delete_by_id(status_id)?;
        BoostRepository::with_transaction(tx).delete_by_id(status_id)?;
        LikedRepository::with_transaction(tx).unlike_status(status_uri)?;
        StatusRepository::with_transaction(tx).delete_by_id(status_id)?;
        Ok(())
    })
}

async fn dispatch_delete(status_uri: &str) -> CanisterResult<()> {
    let profile = ProfileRepository::oneshot().get_profile()?;
    let owner_uri = urls::actor_uri(&profile.handle.0)?;

    let recipients = followers_minus_blocked()?;
    if recipients.is_empty() {
        ic_utils::log!("delete_status: no recipients to dispatch to");
        return Ok(());
    }

    let activity = make_delete_activity(&owner_uri, status_uri);
    let activity_json =
        serde_json::to_string(&activity).expect("Activity serialization must not fail");

    let batch: Vec<SendActivityArgsObject> = recipients
        .iter()
        .map(|recipient| SendActivityArgsObject {
            activity_json: activity_json.clone(),
            target_inbox: urls::inbox_url_from_actor_uri(recipient),
        })
        .collect();

    federation::send_activity(SendActivityArgs::Batch(batch)).await
}

fn make_delete_activity(owner_uri: &str, status_uri: &str) -> Activity {
    Activity {
        base: BaseObject {
            id: Some(format!("{status_uri}#delete")),
            context: Some(Context::Uri(ACTIVITY_STREAMS_CONTEXT.to_string())),
            kind: ActivityType::Delete,
            to: Some(OneOrMany::One(AS_PUBLIC.to_string())),
            ..Default::default()
        },
        actor: Some(owner_uri.to_string()),
        object: Some(ActivityObject::Id(status_uri.to_string())),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    }
}

fn followers_minus_blocked() -> CanisterResult<Vec<String>> {
    let followers = FollowerRepository::oneshot().get_followers()?;
    let blocked = BlockRepository::oneshot().list_blocked_uris()?;
    Ok(followers
        .into_iter()
        .map(|f| f.actor_uri.0)
        .filter(|uri| !blocked.contains(uri))
        .collect())
}

#[cfg(test)]
mod tests {
    use activitypub::activity::ActivityType;
    use did::common::Visibility;
    use did::federation::SendActivityArgs;
    use did::user::{DeleteStatusArgs, DeleteStatusError, DeleteStatusResponse};
    use ic_dbms_canister::prelude::DBMS_CONTEXT;
    use wasm_dbms::WasmDbmsDatabase;
    use wasm_dbms_api::prelude::{Database, Query};

    use super::*;
    use crate::adapters::federation::mock::captured;
    use crate::domain::boost::BoostRepository;
    use crate::domain::feed::FeedRepository;
    use crate::domain::liked::LikedRepository;
    use crate::domain::status::StatusRepository;
    use crate::schema::{
        Block, BlockInsertRequest, FeedEntry, Follower, FollowerInsertRequest, Media,
        MediaInsertRequest, Schema,
    };
    use crate::test_utils::{insert_status, setup};

    const OWN_HANDLE: &str = "rey_canisteryo";
    const OWN_URI: &str = "https://mastic.social/users/rey_canisteryo";
    const STATUS_ID: u64 = 42;
    const ALICE_URI: &str = "https://remote.example/users/alice";
    const BOB_URI: &str = "https://remote.example/users/bob";

    fn local_status_uri() -> String {
        format!("{OWN_URI}/statuses/{STATUS_ID}")
    }

    fn insert_follower(uri: &str) {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Follower>(FollowerInsertRequest {
                actor_uri: uri.into(),
                created_at: ic_utils::now().into(),
            })
            .expect("insert follower");
        });
    }

    fn insert_block(uri: &str) {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Block>(BlockInsertRequest {
                actor_uri: uri.into(),
                created_at: ic_utils::now().into(),
            })
            .expect("insert block");
        });
    }

    fn insert_feed_outbox(id: u64, created_at: u64) {
        FeedRepository::oneshot()
            .insert_outbox(id, created_at)
            .expect("insert feed");
    }

    fn count_feed() -> usize {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.select::<FeedEntry>(Query::builder().all().build())
                .expect("select feed")
                .len()
        })
    }

    fn count_media() -> usize {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.select::<Media>(Query::builder().all().build())
                .expect("select media")
                .len()
        })
    }

    fn insert_media(id: u64, status_id: u64) {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Media>(MediaInsertRequest {
                id: id.into(),
                status_id: status_id.into(),
                media_type: "image/png".into(),
                description: wasm_dbms_api::prelude::Nullable::Null,
                blurhash: wasm_dbms_api::prelude::Nullable::Null,
                bytes: vec![1u8, 2, 3].into(),
                created_at: ic_utils::now().into(),
            })
            .expect("insert media");
        });
    }

    #[tokio::test]
    async fn test_should_delete_own_status_and_dispatch_to_followers() {
        setup();
        insert_status(STATUS_ID, "to delete", Visibility::Public, 1_000);
        insert_feed_outbox(STATUS_ID, 1_000);
        insert_follower(ALICE_URI);

        let resp = delete_status(DeleteStatusArgs {
            status_uri: local_status_uri(),
        })
        .await;
        assert_eq!(resp, DeleteStatusResponse::Ok);

        assert!(
            StatusRepository::oneshot()
                .find_by_id(STATUS_ID)
                .expect("query")
                .is_none()
        );
        assert_eq!(count_feed(), 0);

        let captured = captured();
        assert_eq!(captured.len(), 1);
        let SendActivityArgs::Batch(batch) = &captured[0] else {
            panic!("expected batch");
        };
        assert_eq!(batch.len(), 1);
        let activity: activitypub::Activity =
            serde_json::from_str(&batch[0].activity_json).unwrap();
        assert_eq!(activity.base.kind, ActivityType::Delete);
        assert_eq!(activity.actor.as_deref(), Some(OWN_URI));
        assert_eq!(batch[0].target_inbox, format!("{ALICE_URI}/inbox"));
    }

    #[tokio::test]
    async fn test_should_cascade_remove_media() {
        setup();
        insert_status(STATUS_ID, "with media", Visibility::Public, 1_000);
        insert_feed_outbox(STATUS_ID, 1_000);
        insert_media(900, STATUS_ID);
        assert_eq!(count_media(), 1);

        let resp = delete_status(DeleteStatusArgs {
            status_uri: local_status_uri(),
        })
        .await;
        assert_eq!(resp, DeleteStatusResponse::Ok);
        assert_eq!(count_media(), 0, "media must cascade-delete");
    }

    #[tokio::test]
    async fn test_should_remove_own_boost_wrapper() {
        setup();
        // Wrapper status + boost row sharing the same snowflake.
        insert_status(STATUS_ID, "wrapper", Visibility::Public, 1_000);
        BoostRepository::oneshot()
            .insert(STATUS_ID, "https://other.example/users/x/statuses/9", 1_000)
            .expect("insert boost");
        insert_feed_outbox(STATUS_ID, 1_000);

        let resp = delete_status(DeleteStatusArgs {
            status_uri: local_status_uri(),
        })
        .await;
        assert_eq!(resp, DeleteStatusResponse::Ok);

        assert!(
            !BoostRepository::oneshot()
                .is_boosted("https://other.example/users/x/statuses/9")
                .expect("query")
        );
    }

    #[tokio::test]
    async fn test_should_clear_liked_referencing_uri() {
        setup();
        insert_status(STATUS_ID, "with liked", Visibility::Public, 1_000);
        insert_feed_outbox(STATUS_ID, 1_000);
        // defensive: liked row pointing at the same URI gets cleared too.
        LikedRepository::oneshot()
            .like_status(&local_status_uri())
            .expect("like");
        assert!(
            LikedRepository::oneshot()
                .is_liked(&local_status_uri())
                .unwrap()
        );

        let resp = delete_status(DeleteStatusArgs {
            status_uri: local_status_uri(),
        })
        .await;
        assert_eq!(resp, DeleteStatusResponse::Ok);
        assert!(
            !LikedRepository::oneshot()
                .is_liked(&local_status_uri())
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_should_skip_blocked_followers() {
        setup();
        insert_status(STATUS_ID, "skip block", Visibility::Public, 1_000);
        insert_feed_outbox(STATUS_ID, 1_000);
        insert_follower(ALICE_URI);
        insert_follower(BOB_URI);
        insert_block(BOB_URI);

        let resp = delete_status(DeleteStatusArgs {
            status_uri: local_status_uri(),
        })
        .await;
        assert_eq!(resp, DeleteStatusResponse::Ok);

        let captured = captured();
        let SendActivityArgs::Batch(batch) = &captured[0] else {
            panic!("expected batch");
        };
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].target_inbox, format!("{ALICE_URI}/inbox"));
    }

    #[tokio::test]
    async fn test_should_succeed_with_no_followers_no_dispatch() {
        setup();
        insert_status(STATUS_ID, "lonely", Visibility::Public, 1_000);
        insert_feed_outbox(STATUS_ID, 1_000);

        let resp = delete_status(DeleteStatusArgs {
            status_uri: local_status_uri(),
        })
        .await;
        assert_eq!(resp, DeleteStatusResponse::Ok);
        assert!(captured().is_empty());
    }

    #[tokio::test]
    async fn test_should_return_invalid_uri_for_bad_input() {
        setup();

        let resp = delete_status(DeleteStatusArgs {
            status_uri: "not-a-status-uri".to_string(),
        })
        .await;
        assert_eq!(
            resp,
            DeleteStatusResponse::Err(DeleteStatusError::InvalidUri)
        );
    }

    #[tokio::test]
    async fn test_should_return_not_found_when_status_missing() {
        setup();

        let resp = delete_status(DeleteStatusArgs {
            status_uri: format!("{OWN_URI}/statuses/9999"),
        })
        .await;
        assert_eq!(resp, DeleteStatusResponse::Err(DeleteStatusError::NotFound));
        assert!(captured().is_empty());
    }

    #[test]
    fn test_make_delete_activity_addresses_status_uri() {
        let uri = format!("{OWN_URI}/statuses/{STATUS_ID}");
        let activity = make_delete_activity(OWN_URI, &uri);
        assert_eq!(activity.base.kind, ActivityType::Delete);
        assert_eq!(activity.actor.as_deref(), Some(OWN_URI));
        let activitypub::activity::ActivityObject::Id(id) =
            activity.object.expect("object present")
        else {
            panic!("expected Id variant");
        };
        assert_eq!(id, uri);
        let _ = OWN_HANDLE; // touch to silence dead-code lint if removed
    }
}
