//! Domain logic for updating the user's profile.

use activitypub::activity::{Activity, ActivityObject, ActivityType};
use activitypub::actor::{Actor, ActorType};
use activitypub::context::{ACTIVITY_STREAMS_CONTEXT, Context};
use activitypub::object::{BaseObject, OneOrMany};
use did::federation::{SendActivityArgs, SendActivityArgsObject};
use did::user::{UpdateProfileArgs, UpdateProfileError, UpdateProfileResponse};

use crate::adapters::federation;
use crate::domain::block::BlockRepository;
use crate::domain::follower::FollowerRepository;
use crate::domain::profile::ProfileRepository;
use crate::domain::urls;
use crate::error::CanisterResult;
use crate::schema::Profile;

/// The ActivityStreams public-audience constant.
const AS_PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";

/// Update the user's profile with the given arguments.
///
/// On a successful mutation the function emits one `Update(Person)` activity
/// per follower (minus blocked actors) via the Federation Canister.
pub async fn update_profile(args: UpdateProfileArgs) -> UpdateProfileResponse {
    ic_utils::log!("Updating profile with args: {args:?}");
    match update_profile_inner(args).await {
        Ok(()) => UpdateProfileResponse::Ok,
        Err(e) => UpdateProfileResponse::Err(e),
    }
}

async fn update_profile_inner(
    UpdateProfileArgs { bio, display_name }: UpdateProfileArgs,
) -> Result<(), UpdateProfileError> {
    let written = ProfileRepository::update_profile(bio, display_name).map_err(|err| {
        ic_utils::log!("Failed to update profile: {err}");
        UpdateProfileError::Internal(err.to_string())
    })?;

    if !written {
        return Ok(());
    }

    dispatch_update_activity()
        .await
        .map_err(|err| UpdateProfileError::Internal(err.to_string()))
}

async fn dispatch_update_activity() -> CanisterResult<()> {
    let profile = ProfileRepository::get_profile()?;
    let handle = profile.handle.0.clone();
    let owner_uri = urls::actor_uri(&handle)?;

    let recipients = followers_minus_blocked()?;
    if recipients.is_empty() {
        ic_utils::log!("No recipients for Update(Person); skipping dispatch");
        return Ok(());
    }

    let activities: Vec<SendActivityArgsObject> = recipients
        .iter()
        .map(|recipient_uri| make_update_activity(&profile, &owner_uri, &handle, recipient_uri))
        .collect::<CanisterResult<Vec<_>>>()?;

    ic_utils::log!(
        "Prepared {len} Update(Person) activities for federation",
        len = activities.len()
    );

    federation::send_activity(SendActivityArgs::Batch(activities)).await
}

/// Return the follower actor URIs, excluding blocked actors.
fn followers_minus_blocked() -> CanisterResult<Vec<String>> {
    let followers = FollowerRepository::get_followers()?;
    let blocked = BlockRepository::list_blocked_uris()?;
    Ok(followers
        .into_iter()
        .map(|f| f.actor_uri.0)
        .filter(|uri| !blocked.contains(uri))
        .collect())
}

/// Build a `SendActivityArgsObject` carrying an `Update(Person)` activity
/// addressed to a single recipient actor.
fn make_update_activity(
    profile: &Profile,
    owner_uri: &str,
    handle: &str,
    recipient_uri: &str,
) -> CanisterResult<SendActivityArgsObject> {
    let person = build_person_actor(profile, owner_uri, handle)?;
    let followers_uri = urls::followers_url(handle)?;

    let activity = Activity {
        base: BaseObject {
            context: Some(Context::Uri(ACTIVITY_STREAMS_CONTEXT.to_string())),
            kind: ActivityType::Update,
            to: Some(OneOrMany::One(AS_PUBLIC.to_string())),
            cc: Some(OneOrMany::One(followers_uri)),
            ..Default::default()
        },
        actor: Some(owner_uri.to_string()),
        object: Some(ActivityObject::Actor(Box::new(person))),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    };

    let activity_json =
        serde_json::to_string(&activity).expect("Activity serialization must not fail");

    Ok(SendActivityArgsObject {
        activity_json,
        target_inbox: urls::inbox_url_from_actor_uri(recipient_uri),
    })
}

/// Build a Mastodon-style Person actor for the owner from the current profile.
fn build_person_actor(profile: &Profile, owner_uri: &str, handle: &str) -> CanisterResult<Actor> {
    let updated = ic_utils::rfc3339(profile.updated_at.0);
    let display_name = profile.display_name.clone().into_opt().map(|t| t.0);
    let bio = profile.bio.clone().into_opt().map(|t| t.0);

    Ok(Actor {
        base: BaseObject::<ActorType> {
            context: Some(Context::Uri(ACTIVITY_STREAMS_CONTEXT.to_string())),
            id: Some(owner_uri.to_string()),
            kind: ActorType::Person,
            name: display_name,
            summary: bio,
            updated: Some(updated),
            ..Default::default()
        },
        inbox: urls::inbox_url(handle)?,
        outbox: urls::outbox_url(handle)?,
        following: urls::following_url(handle)?,
        followers: urls::followers_url(handle)?,
        liked: format!("{owner_uri}/liked"),
        preferred_username: Some(handle.to_string()),
        public_key: None,
        endpoints: None,
        manually_approves_followers: None,
        discoverable: None,
        indexable: None,
        suspended: None,
        memorial: None,
        featured: None,
        featured_tags: None,
        also_known_as: None,
        attribution_domains: None,
        icon: None,
        image: None,
    })
}

#[cfg(test)]
mod tests {
    use activitypub::activity::{ActivityObject, ActivityType};
    use activitypub::actor::ActorType;
    use activitypub::context::ACTIVITY_STREAMS_CONTEXT;
    use activitypub::object::OneOrMany;
    use did::common::FieldUpdate;
    use did::federation::SendActivityArgs;
    use did::user::{GetProfileResponse, UpdateProfileArgs, UpdateProfileResponse};
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
    async fn test_should_set_display_name_and_bio() {
        setup();

        let resp = update_profile(UpdateProfileArgs {
            display_name: FieldUpdate::Set("Rey".to_string()),
            bio: FieldUpdate::Set("hello world".to_string()),
        })
        .await;
        assert_eq!(resp, UpdateProfileResponse::Ok);

        let profile = match crate::domain::profile::get_profile() {
            GetProfileResponse::Ok(p) => p,
            other => panic!("expected Ok, got {other:?}"),
        };
        assert_eq!(profile.display_name.as_deref(), Some("Rey"));
        assert_eq!(profile.bio.as_deref(), Some("hello world"));
    }

    #[tokio::test]
    async fn test_should_clear_fields() {
        setup();
        update_profile(UpdateProfileArgs {
            display_name: FieldUpdate::Set("Rey".to_string()),
            bio: FieldUpdate::Set("hi".to_string()),
        })
        .await;

        let resp = update_profile(UpdateProfileArgs {
            display_name: FieldUpdate::Clear,
            bio: FieldUpdate::Clear,
        })
        .await;
        assert_eq!(resp, UpdateProfileResponse::Ok);

        let profile = match crate::domain::profile::get_profile() {
            GetProfileResponse::Ok(p) => p,
            other => panic!("expected Ok, got {other:?}"),
        };
        assert!(profile.display_name.is_none());
        assert!(profile.bio.is_none());
    }

    #[tokio::test]
    async fn test_should_leave_fields_unchanged() {
        setup();
        update_profile(UpdateProfileArgs {
            display_name: FieldUpdate::Set("Rey".to_string()),
            bio: FieldUpdate::Set("hi".to_string()),
        })
        .await;

        let resp = update_profile(UpdateProfileArgs {
            display_name: FieldUpdate::Leave,
            bio: FieldUpdate::Set("updated".to_string()),
        })
        .await;
        assert_eq!(resp, UpdateProfileResponse::Ok);

        let profile = match crate::domain::profile::get_profile() {
            GetProfileResponse::Ok(p) => p,
            other => panic!("expected Ok, got {other:?}"),
        };
        assert_eq!(profile.display_name.as_deref(), Some("Rey"));
        assert_eq!(profile.bio.as_deref(), Some("updated"));
    }

    #[tokio::test]
    async fn test_should_skip_on_all_leave() {
        setup();
        let resp = update_profile(UpdateProfileArgs {
            display_name: FieldUpdate::Leave,
            bio: FieldUpdate::Leave,
        })
        .await;
        assert_eq!(resp, UpdateProfileResponse::Ok);
        assert!(captured().is_empty(), "no activity must be dispatched");
    }

    #[tokio::test]
    async fn test_should_fan_out_to_all_followers() {
        setup();
        insert_follower(ALICE_URI);
        insert_follower(BOB_URI);

        let resp = update_profile(UpdateProfileArgs {
            display_name: FieldUpdate::Set("Rey".to_string()),
            bio: FieldUpdate::Leave,
        })
        .await;
        assert_eq!(resp, UpdateProfileResponse::Ok);

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

        let resp = update_profile(UpdateProfileArgs {
            display_name: FieldUpdate::Set("Rey".to_string()),
            bio: FieldUpdate::Leave,
        })
        .await;
        assert_eq!(resp, UpdateProfileResponse::Ok);

        let targets = captured_flat_targets();
        assert_eq!(targets, vec![format!("{ALICE_URI}/inbox")]);
    }

    #[tokio::test]
    async fn test_should_build_update_person_activity() {
        setup();
        insert_follower(ALICE_URI);

        update_profile(UpdateProfileArgs {
            display_name: FieldUpdate::Set("Rey".to_string()),
            bio: FieldUpdate::Set("hi".to_string()),
        })
        .await;

        let args = captured();
        assert_eq!(args.len(), 1);
        let batch = match &args[0] {
            SendActivityArgs::Batch(v) => v.clone(),
            SendActivityArgs::One(_) => panic!("expected Batch"),
        };
        assert_eq!(batch.len(), 1);
        let activity: activitypub::activity::Activity =
            serde_json::from_str(&batch[0].activity_json).expect("deserialize");

        assert_eq!(activity.base.kind, ActivityType::Update);
        assert_eq!(activity.actor.as_deref(), Some(OWNER_URI));
        assert_eq!(
            activity.base.to,
            Some(OneOrMany::One(AS_PUBLIC.to_string()))
        );
        assert_eq!(
            activity.base.cc,
            Some(OneOrMany::One(format!("{OWNER_URI}/followers")))
        );
        assert_eq!(
            activity.base.context,
            Some(activitypub::context::Context::Uri(
                ACTIVITY_STREAMS_CONTEXT.to_string()
            ))
        );

        let ActivityObject::Actor(actor) = activity.object.expect("object present") else {
            panic!("expected Actor variant");
        };
        assert_eq!(actor.base.kind, ActorType::Person);
        assert_eq!(actor.base.id.as_deref(), Some(OWNER_URI));
        assert_eq!(actor.base.name.as_deref(), Some("Rey"));
        assert_eq!(actor.base.summary.as_deref(), Some("hi"));
        assert!(actor.base.updated.is_some());
        assert_eq!(actor.inbox, format!("{OWNER_URI}/inbox"));
        assert_eq!(actor.outbox, format!("{OWNER_URI}/outbox"));
        assert_eq!(actor.followers, format!("{OWNER_URI}/followers"));
        assert_eq!(actor.following, format!("{OWNER_URI}/following"));
        assert_eq!(actor.preferred_username.as_deref(), Some("rey_canisteryo"));
    }
}
