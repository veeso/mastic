//! Conversions between `activitypub` protocol types and Candid-compatible DID types.

use did::federation::{Activity, ActivityType};

/// Convert an [`activitypub::ActivityType`] to the Candid-compatible [`ActivityType`].
pub fn activity_type_from_ap(value: activitypub::ActivityType) -> ActivityType {
    match value {
        activitypub::ActivityType::Create => ActivityType::Create,
        activitypub::ActivityType::Update => ActivityType::Update,
        activitypub::ActivityType::Delete => ActivityType::Delete,
        activitypub::ActivityType::Follow => ActivityType::Follow,
        activitypub::ActivityType::Accept => ActivityType::Accept,
        activitypub::ActivityType::Reject => ActivityType::Reject,
        activitypub::ActivityType::Like => ActivityType::Like,
        activitypub::ActivityType::Announce => ActivityType::Announce,
        activitypub::ActivityType::Undo => ActivityType::Undo,
        activitypub::ActivityType::Block => ActivityType::Block,
        activitypub::ActivityType::Add => ActivityType::Add,
        activitypub::ActivityType::Remove => ActivityType::Remove,
        activitypub::ActivityType::Flag => ActivityType::Flag,
        activitypub::ActivityType::Move => ActivityType::Move,
    }
}

/// Convert a Candid-compatible [`ActivityType`] to [`activitypub::ActivityType`].
pub fn activity_type_to_ap(value: ActivityType) -> activitypub::ActivityType {
    match value {
        ActivityType::Create => activitypub::ActivityType::Create,
        ActivityType::Update => activitypub::ActivityType::Update,
        ActivityType::Delete => activitypub::ActivityType::Delete,
        ActivityType::Follow => activitypub::ActivityType::Follow,
        ActivityType::Accept => activitypub::ActivityType::Accept,
        ActivityType::Reject => activitypub::ActivityType::Reject,
        ActivityType::Like => activitypub::ActivityType::Like,
        ActivityType::Announce => activitypub::ActivityType::Announce,
        ActivityType::Undo => activitypub::ActivityType::Undo,
        ActivityType::Block => activitypub::ActivityType::Block,
        ActivityType::Add => activitypub::ActivityType::Add,
        ActivityType::Remove => activitypub::ActivityType::Remove,
        ActivityType::Flag => activitypub::ActivityType::Flag,
        ActivityType::Move => activitypub::ActivityType::Move,
    }
}

/// Flattens an optional [`activitypub::object::OneOrMany<String>`] into a [`Vec<String>`].
pub fn one_or_many_to_vec(value: Option<activitypub::object::OneOrMany<String>>) -> Vec<String> {
    match value {
        None => Vec::new(),
        Some(activitypub::object::OneOrMany::One(s)) => vec![s],
        Some(activitypub::object::OneOrMany::Many(v)) => v,
    }
}

/// Converts a [`Vec<String>`] back into an optional [`activitypub::object::OneOrMany<String>`].
pub fn vec_to_one_or_many(value: Vec<String>) -> Option<activitypub::object::OneOrMany<String>> {
    match value.len() {
        0 => None,
        1 => Some(activitypub::object::OneOrMany::One(
            value.into_iter().next().unwrap(),
        )),
        _ => Some(activitypub::object::OneOrMany::Many(value)),
    }
}

/// Convert an [`activitypub::Activity`] to the Candid-compatible [`Activity`].
pub fn activity_from_ap(activity: activitypub::Activity) -> Activity {
    let object_json = activity
        .object
        .as_ref()
        .map(|o| serde_json::to_string(o).unwrap());

    Activity {
        id: activity.base.id,
        activity_type: activity_type_from_ap(activity.base.kind),
        actor: activity.actor,
        object_json,
        target: activity.target,
        to: one_or_many_to_vec(activity.base.to),
        cc: one_or_many_to_vec(activity.base.cc),
        published: activity.base.published,
    }
}

/// Convert a Candid-compatible [`Activity`] to [`activitypub::Activity`].
pub fn activity_to_ap(activity: Activity) -> activitypub::Activity {
    let object = activity
        .object_json
        .as_deref()
        .map(|json| serde_json::from_str(json).unwrap());

    activitypub::Activity {
        base: activitypub::BaseObject {
            id: activity.id,
            kind: activity_type_to_ap(activity.activity_type),
            published: activity.published,
            to: vec_to_one_or_many(activity.to),
            cc: vec_to_one_or_many(activity.cc),
            ..Default::default()
        },
        actor: activity.actor,
        object,
        target: activity.target,
        result: None,
        origin: None,
        instrument: None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_should_convert_activity_type_roundtrip() {
        for ap_type in [
            activitypub::ActivityType::Create,
            activitypub::ActivityType::Update,
            activitypub::ActivityType::Delete,
            activitypub::ActivityType::Follow,
            activitypub::ActivityType::Accept,
            activitypub::ActivityType::Reject,
            activitypub::ActivityType::Like,
            activitypub::ActivityType::Announce,
            activitypub::ActivityType::Undo,
            activitypub::ActivityType::Block,
            activitypub::ActivityType::Add,
            activitypub::ActivityType::Remove,
            activitypub::ActivityType::Flag,
            activitypub::ActivityType::Move,
        ] {
            let did_type = activity_type_from_ap(ap_type);
            let back = activity_type_to_ap(did_type);
            assert_eq!(ap_type, back);
        }
    }

    #[test]
    fn test_should_convert_activity_to_did_and_back() {
        let json = r##"{
          "@context":"https://www.w3.org/ns/activitystreams",
          "type":"Create",
          "actor":"https://example.com/users/alice",
          "object":{"type":"Note","content":"hello"},
          "to":["https://example.com/users/bob"],
          "cc":["https://www.w3.org/ns/activitystreams#Public"],
          "published":"2025-01-01T00:00:00Z"
        }"##;

        let original: activitypub::Activity = serde_json::from_str(json).expect("must deserialize");
        let did_activity = activity_from_ap(original.clone());

        assert_eq!(did_activity.activity_type, ActivityType::Create);
        assert_eq!(
            did_activity.actor.as_deref(),
            Some("https://example.com/users/alice")
        );
        assert!(did_activity.object_json.is_some());
        assert_eq!(did_activity.to, vec!["https://example.com/users/bob"]);
        assert_eq!(
            did_activity.cc,
            vec!["https://www.w3.org/ns/activitystreams#Public"]
        );
        assert_eq!(
            did_activity.published.as_deref(),
            Some("2025-01-01T00:00:00Z")
        );

        // Convert back and verify essential fields survive the round-trip.
        let restored = activity_to_ap(did_activity);
        assert_eq!(restored.base.kind, activitypub::ActivityType::Create);
        assert_eq!(restored.actor, original.actor);
        assert_eq!(restored.target, original.target);
        assert_eq!(
            one_or_many_to_vec(restored.base.to),
            one_or_many_to_vec(original.base.to)
        );
        assert_eq!(
            one_or_many_to_vec(restored.base.cc),
            one_or_many_to_vec(original.base.cc)
        );
        assert_eq!(restored.base.published, original.base.published);

        assert!(matches!(
            restored.object,
            Some(activitypub::ActivityObject::Object(_))
        ));
    }
}
