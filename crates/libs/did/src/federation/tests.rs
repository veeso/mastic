use candid::{Decode, Encode};

use super::*;

#[test]
fn test_should_roundtrip_activity_type() {
    for activity_type in [
        ActivityType::Create,
        ActivityType::Update,
        ActivityType::Delete,
        ActivityType::Follow,
        ActivityType::Accept,
        ActivityType::Reject,
        ActivityType::Like,
        ActivityType::Announce,
        ActivityType::Undo,
        ActivityType::Block,
        ActivityType::Add,
        ActivityType::Remove,
        ActivityType::Flag,
        ActivityType::Move,
    ] {
        let bytes = Encode!(&activity_type).unwrap();
        let decoded = Decode!(&bytes, ActivityType).unwrap();
        assert_eq!(activity_type, decoded);
    }
}

#[test]
fn test_should_roundtrip_activity() {
    let activity = Activity {
        id: Some("https://example.com/activities/1".to_string()),
        activity_type: ActivityType::Create,
        actor: Some("https://example.com/users/alice".to_string()),
        object_json: Some(r#"{"type":"Note","content":"hello"}"#.to_string()),
        target: None,
        to: vec!["https://example.com/users/bob".to_string()],
        cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        published: Some("2025-01-01T00:00:00Z".to_string()),
    };
    let bytes = Encode!(&activity).unwrap();
    let decoded = Decode!(&bytes, Activity).unwrap();
    assert_eq!(activity, decoded);
}

#[test]
fn test_should_roundtrip_activity_minimal() {
    let activity = Activity {
        id: None,
        activity_type: ActivityType::Follow,
        actor: Some("https://example.com/users/alice".to_string()),
        object_json: Some("\"https://example.com/users/bob\"".to_string()),
        target: None,
        to: vec![],
        cc: vec![],
        published: None,
    };
    let bytes = Encode!(&activity).unwrap();
    let decoded = Decode!(&bytes, Activity).unwrap();
    assert_eq!(activity, decoded);
}

#[test]
fn test_should_roundtrip_federation_install_args_init() {
    let args = FederationInstallArgs::Init {
        directory_canister: candid::Principal::anonymous(),
        public_url: "https://example.com".to_string(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, FederationInstallArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_federation_install_args_upgrade() {
    let args = FederationInstallArgs::Upgrade {};
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, FederationInstallArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_send_activity_args_object() {
    let args = SendActivityArgsObject {
        activity_json: r#"{"type":"Create"}"#.to_string(),
        target_inbox: "https://remote.example/inbox".to_string(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, SendActivityArgsObject).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_send_activity_args() {
    let args = SendActivityArgs::One(SendActivityArgsObject {
        activity_json: r#"{"type":"Create"}"#.to_string(),
        target_inbox: "https://remote.example/inbox".to_string(),
    });
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, SendActivityArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_send_activity_error() {
    for error in [
        SendActivityError::Unauthorized,
        SendActivityError::DeliveryFailed,
        SendActivityError::InvalidActivity,
    ] {
        let bytes = Encode!(&error).unwrap();
        let decoded = Decode!(&bytes, SendActivityError).unwrap();
        assert_eq!(error, decoded);
    }
}

#[test]
fn test_should_roundtrip_send_activity_response_ok() {
    let resp = SendActivityResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, SendActivityResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_send_activity_response_err() {
    for error in [
        SendActivityError::Unauthorized,
        SendActivityError::DeliveryFailed,
        SendActivityError::InvalidActivity,
    ] {
        let resp = SendActivityResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, SendActivityResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}
