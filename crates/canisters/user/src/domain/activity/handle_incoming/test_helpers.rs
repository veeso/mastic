//! Shared helpers for `handle_incoming` test modules.

use activitypub::activity::{Activity, ActivityObject, ActivityType};
use activitypub::object::BaseObject;

pub(super) const REMOTE_BOOSTER: &str = "https://remote.example/users/alice";
pub(super) const LOCAL_STATUS_URI: &str = "https://mastic.social/users/rey_canisteryo/statuses/42";
pub(super) const REMOTE_AUTHOR: &str = "https://remote.example/users/dave";
pub(super) const REMOTE_NOTE_URI: &str = "https://remote.example/users/dave/statuses/7";

pub(super) fn make_follow_json(follower_actor_uri: &str, target_actor_uri: &str) -> String {
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

pub(super) fn make_accept_follow_json(
    acceptor_actor_uri: &str,
    follower_actor_uri: &str,
) -> String {
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

pub(super) fn make_reject_follow_json(
    rejector_actor_uri: &str,
    follower_actor_uri: &str,
) -> String {
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

pub(super) fn make_undo_follow_json(unfollower_actor_uri: &str, target_actor_uri: &str) -> String {
    let activity = Activity {
        base: BaseObject {
            kind: ActivityType::Undo,
            ..Default::default()
        },
        actor: Some(unfollower_actor_uri.to_string()),
        object: Some(ActivityObject::Activity(Box::new(Activity {
            base: BaseObject {
                kind: ActivityType::Follow,
                ..Default::default()
            },
            actor: Some(unfollower_actor_uri.to_string()),
            object: Some(ActivityObject::Id(target_actor_uri.to_string())),
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

pub(super) fn make_like_json(actor_uri: &str, status_uri: &str) -> String {
    let activity = Activity {
        base: BaseObject {
            kind: ActivityType::Like,
            ..Default::default()
        },
        actor: Some(actor_uri.to_string()),
        object: Some(ActivityObject::Id(status_uri.to_string())),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    };
    serde_json::to_string(&activity).unwrap()
}

pub(super) fn make_undo_like_json(actor_uri: &str, status_uri: &str) -> String {
    let activity = Activity {
        base: BaseObject {
            kind: ActivityType::Undo,
            ..Default::default()
        },
        actor: Some(actor_uri.to_string()),
        object: Some(ActivityObject::Activity(Box::new(Activity {
            base: BaseObject {
                kind: ActivityType::Like,
                ..Default::default()
            },
            actor: Some(actor_uri.to_string()),
            object: Some(ActivityObject::Id(status_uri.to_string())),
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

pub(super) fn make_announce_json(actor: &str, target_uri: &str) -> String {
    let activity = Activity {
        base: BaseObject {
            kind: ActivityType::Announce,
            ..Default::default()
        },
        actor: Some(actor.into()),
        object: Some(ActivityObject::Id(target_uri.into())),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    };
    serde_json::to_string(&activity).unwrap()
}

pub(super) fn make_undo_announce_json(actor: &str, target_uri: &str) -> String {
    let activity = Activity {
        base: BaseObject {
            kind: ActivityType::Undo,
            ..Default::default()
        },
        actor: Some(actor.into()),
        object: Some(ActivityObject::Activity(Box::new(Activity {
            base: BaseObject {
                kind: ActivityType::Announce,
                ..Default::default()
            },
            actor: Some(actor.into()),
            object: Some(ActivityObject::Id(target_uri.into())),
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

pub(super) fn make_create_note_json(actor_uri: &str, note_id: &str, content: &str) -> String {
    let activity = Activity {
        base: BaseObject {
            kind: ActivityType::Create,
            ..Default::default()
        },
        actor: Some(actor_uri.to_string()),
        object: Some(ActivityObject::Object(Box::new(BaseObject {
            id: Some(note_id.to_string()),
            kind: activitypub::object::ObjectType::Note,
            content: Some(content.to_string()),
            ..Default::default()
        }))),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    };
    serde_json::to_string(&activity).unwrap()
}

pub(super) fn make_delete_json(actor_uri: &str, target_uri: &str) -> String {
    let activity = Activity {
        base: BaseObject {
            kind: ActivityType::Delete,
            ..Default::default()
        },
        actor: Some(actor_uri.to_string()),
        object: Some(ActivityObject::Id(target_uri.to_string())),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    };
    serde_json::to_string(&activity).unwrap()
}

pub(super) fn count_inbox() -> usize {
    use ic_dbms_canister::prelude::DBMS_CONTEXT;
    use wasm_dbms::WasmDbmsDatabase;
    use wasm_dbms_api::prelude::{Database, Query};
    DBMS_CONTEXT.with(|ctx| {
        let db = WasmDbmsDatabase::oneshot(ctx, crate::schema::Schema);
        db.select::<crate::schema::InboxActivity>(Query::builder().all().build())
            .unwrap()
            .len()
    })
}

pub(super) fn count_feed() -> usize {
    use ic_dbms_canister::prelude::DBMS_CONTEXT;
    use wasm_dbms::WasmDbmsDatabase;
    use wasm_dbms_api::prelude::{Database, Query};
    DBMS_CONTEXT.with(|ctx| {
        let db = WasmDbmsDatabase::oneshot(ctx, crate::schema::Schema);
        db.select::<crate::schema::FeedEntry>(Query::builder().all().build())
            .unwrap()
            .len()
    })
}
