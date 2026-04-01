use candid::{Decode, Encode};

use super::*;

#[test]
fn test_should_roundtrip_visibility() {
    for variant in [
        Visibility::Public,
        Visibility::Unlisted,
        Visibility::FollowersOnly,
        Visibility::Direct,
    ] {
        let bytes = Encode!(&variant).unwrap();
        let decoded = Decode!(&bytes, Visibility).unwrap();
        assert_eq!(variant, decoded);
    }
}

#[test]
fn test_should_roundtrip_user_profile() {
    let profile = UserProfile {
        handle: "alice".to_string(),
        display_name: Some("Alice".to_string()),
        bio: Some("Hello world".to_string()),
        avatar_url: Some("https://example.com/avatar.png".to_string()),
        created_at: 1_000_000_000,
    };
    let bytes = Encode!(&profile).unwrap();
    let decoded = Decode!(&bytes, UserProfile).unwrap();
    assert_eq!(profile, decoded);
}

#[test]
fn test_should_roundtrip_user_profile_with_none_fields() {
    let profile = UserProfile {
        handle: "bob".to_string(),
        display_name: None,
        bio: None,
        avatar_url: None,
        created_at: 0,
    };
    let bytes = Encode!(&profile).unwrap();
    let decoded = Decode!(&bytes, UserProfile).unwrap();
    assert_eq!(profile, decoded);
}

#[test]
fn test_should_roundtrip_status() {
    let status = Status {
        id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        content: "Hello, world!".to_string(),
        author: candid::Principal::anonymous(),
        created_at: 1_000_000_000,
        visibility: Visibility::Public,
    };
    let bytes = Encode!(&status).unwrap();
    let decoded = Decode!(&bytes, Status).unwrap();
    assert_eq!(status, decoded);
}

#[test]
fn test_should_roundtrip_feed_item() {
    let item = FeedItem {
        status: Status {
            id: "test-id".to_string(),
            content: "A post".to_string(),
            author: candid::Principal::anonymous(),
            created_at: 42,
            visibility: Visibility::FollowersOnly,
        },
        boosted_by: Some(candid::Principal::anonymous()),
    };
    let bytes = Encode!(&item).unwrap();
    let decoded = Decode!(&bytes, FeedItem).unwrap();
    assert_eq!(item, decoded);
}

#[test]
fn test_should_roundtrip_feed_item_without_boost() {
    let item = FeedItem {
        status: Status {
            id: "test-id".to_string(),
            content: "A post".to_string(),
            author: candid::Principal::anonymous(),
            created_at: 42,
            visibility: Visibility::Unlisted,
        },
        boosted_by: None,
    };
    let bytes = Encode!(&item).unwrap();
    let decoded = Decode!(&bytes, FeedItem).unwrap();
    assert_eq!(item, decoded);
}
