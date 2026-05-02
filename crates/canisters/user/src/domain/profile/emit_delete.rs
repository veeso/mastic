//! Domain logic for emitting a `Delete(Person)` activity to followers on profile deletion.

use activitypub::activity::{Activity, ActivityObject, ActivityType};
use activitypub::context::{ACTIVITY_STREAMS_CONTEXT, Context};
use activitypub::object::{BaseObject, OneOrMany};
use did::federation::{SendActivityArgs, SendActivityArgsObject};
use did::user::{EmitDeleteProfileActivityError, EmitDeleteProfileActivityResponse};

use crate::adapters::federation;
use crate::domain::block::BlockRepository;
use crate::domain::follower::FollowerRepository;
use crate::domain::profile::ProfileRepository;
use crate::domain::urls;
use crate::error::CanisterResult;

/// The ActivityStreams public-audience constant.
const AS_PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";

/// Emit a `Delete(Person)` activity for the owner to all (non-blocked) followers.
///
/// Called by the Directory Canister during the delete_profile flow before the
/// User Canister is stopped and deleted.
pub async fn emit_delete_profile_activity() -> EmitDeleteProfileActivityResponse {
    ic_utils::log!("emit_delete_profile_activity: dispatching Delete(Person) to followers");

    match dispatch().await {
        Ok(()) => EmitDeleteProfileActivityResponse::Ok,
        Err(err) => {
            ic_utils::log!("emit_delete_profile_activity: failed: {err}");
            EmitDeleteProfileActivityResponse::Err(EmitDeleteProfileActivityError::Internal(
                err.to_string(),
            ))
        }
    }
}

async fn dispatch() -> CanisterResult<()> {
    let profile = ProfileRepository::get_profile()?;
    let handle = profile.handle.0.clone();
    let owner_uri = urls::actor_uri(&handle)?;

    let recipients = followers_minus_blocked()?;
    if recipients.is_empty() {
        ic_utils::log!("emit_delete_profile_activity: no recipients; skipping dispatch");
        return Ok(());
    }

    let followers_uri = urls::followers_url(&handle)?;

    let activities: Vec<SendActivityArgsObject> = recipients
        .iter()
        .map(|recipient_uri| {
            let activity = Activity {
                base: BaseObject {
                    context: Some(Context::Uri(ACTIVITY_STREAMS_CONTEXT.to_string())),
                    kind: ActivityType::Delete,
                    to: Some(OneOrMany::One(AS_PUBLIC.to_string())),
                    cc: Some(OneOrMany::One(followers_uri.clone())),
                    ..Default::default()
                },
                actor: Some(owner_uri.clone()),
                object: Some(ActivityObject::Id(owner_uri.clone())),
                target: None,
                result: None,
                origin: None,
                instrument: None,
            };

            let activity_json =
                serde_json::to_string(&activity).expect("Activity serialization must not fail");

            SendActivityArgsObject {
                activity_json,
                target_inbox: urls::inbox_url_from_actor_uri(recipient_uri),
            }
        })
        .collect();

    ic_utils::log!(
        "emit_delete_profile_activity: prepared {len} activities for federation",
        len = activities.len()
    );

    federation::send_activity(SendActivityArgs::Batch(activities)).await
}

fn followers_minus_blocked() -> CanisterResult<Vec<String>> {
    let followers = FollowerRepository::get_followers()?;
    let blocked = BlockRepository::oneshot().list_blocked_uris()?;
    Ok(followers
        .into_iter()
        .map(|f| f.actor_uri.0)
        .filter(|uri| !blocked.contains(uri))
        .collect())
}

#[cfg(test)]
mod tests {
    use activitypub::activity::{ActivityObject, ActivityType};
    use did::federation::SendActivityArgs;
    use ic_dbms_canister::prelude::DBMS_CONTEXT;
    use wasm_dbms::WasmDbmsDatabase;
    use wasm_dbms_api::prelude::Database;

    use super::*;
    use crate::adapters::federation::mock::captured;
    use crate::schema::{Block, BlockInsertRequest, Follower, FollowerInsertRequest, Schema};
    use crate::test_utils::setup;

    const ALICE_URI: &str = "https://remote.example/users/alice";
    const BOB_URI: &str = "https://remote.example/users/bob";
    const OWNER_URI: &str = "https://mastic.social/users/rey_canisteryo";

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

    fn insert_block(actor_uri: &str) {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Block>(BlockInsertRequest {
                actor_uri: actor_uri.into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert block");
        });
    }

    fn captured_flat_targets() -> Vec<String> {
        captured()
            .into_iter()
            .flat_map(|args| match args {
                SendActivityArgs::One(obj) => vec![obj.target_inbox],
                SendActivityArgs::Batch(objs) => objs.into_iter().map(|o| o.target_inbox).collect(),
            })
            .collect()
    }

    #[tokio::test]
    async fn test_should_skip_when_no_followers() {
        setup();
        let resp = emit_delete_profile_activity().await;
        assert_eq!(resp, EmitDeleteProfileActivityResponse::Ok);
        assert!(captured().is_empty());
    }

    #[tokio::test]
    async fn test_should_fan_out_to_all_followers() {
        setup();
        insert_follower(ALICE_URI);
        insert_follower(BOB_URI);

        let resp = emit_delete_profile_activity().await;
        assert_eq!(resp, EmitDeleteProfileActivityResponse::Ok);

        let targets = captured_flat_targets();
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&format!("{ALICE_URI}/inbox")));
        assert!(targets.contains(&format!("{BOB_URI}/inbox")));
    }

    #[tokio::test]
    async fn test_should_exclude_blocked_followers() {
        setup();
        insert_follower(ALICE_URI);
        insert_follower(BOB_URI);
        insert_block(BOB_URI);

        let resp = emit_delete_profile_activity().await;
        assert_eq!(resp, EmitDeleteProfileActivityResponse::Ok);

        let targets = captured_flat_targets();
        assert_eq!(targets, vec![format!("{ALICE_URI}/inbox")]);
    }

    #[tokio::test]
    async fn test_should_build_delete_person_activity() {
        setup();
        insert_follower(ALICE_URI);

        emit_delete_profile_activity().await;

        let args = captured();
        assert_eq!(args.len(), 1);
        let batch = match &args[0] {
            SendActivityArgs::Batch(v) => v.clone(),
            SendActivityArgs::One(_) => panic!("expected Batch"),
        };
        assert_eq!(batch.len(), 1);
        let activity: activitypub::activity::Activity =
            serde_json::from_str(&batch[0].activity_json).expect("deserialize");

        assert_eq!(activity.base.kind, ActivityType::Delete);
        assert_eq!(activity.actor.as_deref(), Some(OWNER_URI));
        let ActivityObject::Id(id) = activity.object.expect("object present") else {
            panic!("expected Id variant");
        };
        assert_eq!(id, OWNER_URI);
    }
}
