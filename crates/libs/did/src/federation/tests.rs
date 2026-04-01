use candid::{Decode, Encode};

use super::*;

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
fn test_should_roundtrip_send_activity_args() {
    let args = SendActivityArgs {
        activity_json: r#"{"type":"Create"}"#.to_string(),
        target_inbox: "https://remote.example/inbox".to_string(),
    };
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
