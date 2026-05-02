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

// M-UNIT-TEST: RegisterUserArgs round-trips through Candid encoding.
#[test]
fn test_should_roundtrip_register_user_args() {
    let args = RegisterUserArgs {
        user_id: candid::Principal::from_text("mfufu-x6j4c-gomzb-geilq").unwrap(),
        user_handle: "alice".to_string(),
        user_canister_id: candid::Principal::from_text("bkyz2-fmaaa-aaaaa-qaaaq-cai").unwrap(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, RegisterUserArgs).unwrap();
    assert_eq!(args, decoded);
}

// M-UNIT-TEST: RegisterUserResponse::Ok round-trips through Candid encoding.
#[test]
fn test_should_roundtrip_register_user_response_ok() {
    let resp = RegisterUserResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, RegisterUserResponse).unwrap();
    assert_eq!(resp, decoded);
}

// M-UNIT-TEST: RegisterUserResponse::Err round-trips through Candid encoding.
#[test]
fn test_should_roundtrip_register_user_response_err() {
    let resp = RegisterUserResponse::Err(RegisterUserError::Internal("db failure".to_string()));
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, RegisterUserResponse).unwrap();
    assert_eq!(resp, decoded);
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

// M-UNIT-TEST: SendActivityArgs::Batch round-trips through Candid encoding.
#[test]
fn test_should_roundtrip_send_activity_args_batch() {
    let args = SendActivityArgs::Batch(vec![
        SendActivityArgsObject {
            activity_json: r#"{"type":"Create"}"#.to_string(),
            target_inbox: "https://remote.example/inbox".to_string(),
        },
        SendActivityArgsObject {
            activity_json: r#"{"type":"Follow"}"#.to_string(),
            target_inbox: "https://other.example/inbox".to_string(),
        },
    ]);
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, SendActivityArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_send_activity_error() {
    for error in [
        SendActivityError::InvalidTargetInbox("bad url".to_string()),
        SendActivityError::UnknownLocalUser("alice".to_string()),
        SendActivityError::DeliveryFailed("call trapped".to_string()),
        SendActivityError::Rejected("user rejected".to_string()),
    ] {
        let bytes = Encode!(&error).unwrap();
        let decoded = Decode!(&bytes, SendActivityError).unwrap();
        assert_eq!(error, decoded);
    }
}

#[test]
fn test_should_roundtrip_send_activity_result_ok() {
    let result = SendActivityResult::Ok;
    let bytes = Encode!(&result).unwrap();
    let decoded = Decode!(&bytes, SendActivityResult).unwrap();
    assert_eq!(result, decoded);
}

#[test]
fn test_should_roundtrip_send_activity_result_err() {
    for error in [
        SendActivityError::InvalidTargetInbox("bad url".to_string()),
        SendActivityError::UnknownLocalUser("alice".to_string()),
        SendActivityError::DeliveryFailed("call trapped".to_string()),
        SendActivityError::Rejected("user rejected".to_string()),
    ] {
        let result = SendActivityResult::Err(error);
        let bytes = Encode!(&result).unwrap();
        let decoded = Decode!(&bytes, SendActivityResult).unwrap();
        assert_eq!(result, decoded);
    }
}

// M-UNIT-TEST: SendActivityResponse::One round-trips through Candid encoding.
#[test]
fn test_should_roundtrip_send_activity_response_one() {
    let resp = SendActivityResponse::One(SendActivityResult::Ok);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, SendActivityResponse).unwrap();
    assert_eq!(resp, decoded);
}

// M-UNIT-TEST: SendActivityResponse::Batch round-trips through Candid encoding
// with a mix of success and error outcomes.
#[test]
fn test_should_roundtrip_send_activity_response_batch() {
    let resp = SendActivityResponse::Batch(vec![
        SendActivityResult::Ok,
        SendActivityResult::Err(SendActivityError::DeliveryFailed(
            "call trapped".to_string(),
        )),
        SendActivityResult::Err(SendActivityError::UnknownLocalUser("alice".to_string())),
        SendActivityResult::Err(SendActivityError::Rejected("user rejected".to_string())),
    ]);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, SendActivityResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_fetch_status_args_with_requester() {
    let args = FetchStatusArgs {
        uri: "https://mastic.social/users/alice/statuses/123".to_string(),
        requester_actor_uri: Some("https://mastic.social/users/bob".to_string()),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, FetchStatusArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_fetch_status_args_no_requester() {
    let args = FetchStatusArgs {
        uri: "https://mastic.social/users/alice/statuses/456".to_string(),
        requester_actor_uri: None,
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, FetchStatusArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_fetch_status_response_ok() {
    let resp = FetchStatusResponse::Ok(crate::common::Status {
        id: 42,
        content: "Hello, world!".to_string(),
        author: "https://mastic.social/users/alice".to_string(),
        created_at: 1_000_000_000,
        visibility: crate::common::Visibility::Public,
        like_count: 3,
        boost_count: 1,
        spoiler_text: Some("Content warning".to_string()),
        sensitive: true,
    });
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, FetchStatusResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_fetch_status_response_err() {
    for error in [
        FetchStatusError::Unsupported,
        FetchStatusError::InvalidUri,
        FetchStatusError::NotFound,
        FetchStatusError::Internal("db failure".to_string()),
    ] {
        let resp = FetchStatusResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, FetchStatusResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}
