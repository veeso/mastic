//! Canonical ActivityPub URL builders.
//!
//! All URL construction for ActivityPub resources is centralized here
//! to guarantee consistent patterns across the codebase.

use crate::error::CanisterResult;

/// Build the actor URI for a user: `{public_url}/users/{handle}`.
pub fn actor_uri(handle: &str) -> CanisterResult<String> {
    let public_url = crate::settings::get_public_url()?;
    Ok(format!("{public_url}/users/{handle}"))
}

/// Build the inbox URL for a user: `{public_url}/users/{handle}/inbox`.
pub fn inbox_url(handle: &str) -> CanisterResult<String> {
    let public_url = crate::settings::get_public_url()?;
    Ok(format!("{public_url}/users/{handle}/inbox"))
}

/// Build the outbox URL for a user: `{public_url}/users/{handle}/outbox`.
pub fn outbox_url(handle: &str) -> CanisterResult<String> {
    let public_url = crate::settings::get_public_url()?;
    Ok(format!("{public_url}/users/{handle}/outbox"))
}

/// Build the followers collection URL: `{public_url}/users/{handle}/followers`.
pub fn followers_url(handle: &str) -> CanisterResult<String> {
    let public_url = crate::settings::get_public_url()?;
    Ok(format!("{public_url}/users/{handle}/followers"))
}

/// Build the following collection URL: `{public_url}/users/{handle}/following`.
pub fn following_url(handle: &str) -> CanisterResult<String> {
    let public_url = crate::settings::get_public_url()?;
    Ok(format!("{public_url}/users/{handle}/following"))
}

/// Derive the inbox URL from a full actor URI by appending `/inbox`.
///
/// For M0 all users are local, so the inbox is always `{actor_uri}/inbox`.
/// For remote actors (future), this would require a WebFinger lookup.
pub fn inbox_url_from_actor_uri(actor_uri: &str) -> String {
    format!("{actor_uri}/inbox")
}

/// Build the canonical status URI: `{public_url}/users/{handle}/statuses/{id}`.
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "used by future status URI minting")
)]
pub fn status_uri(handle: &str, id: u64) -> CanisterResult<String> {
    let public_url = crate::settings::get_public_url()?;
    Ok(format!("{public_url}/users/{handle}/statuses/{id}"))
}

/// Extract the author actor URI from a status URI by stripping the trailing
/// `/statuses/{id}` segment.
///
/// Returns [`None`] when the URI does not have the expected
/// `…/statuses/{id}` shape (e.g. it contains a sub-path after the id).
pub fn actor_uri_from_status_uri(uri: &str) -> Option<String> {
    let (head, tail) = uri.rsplit_once("/statuses/")?;
    if tail.is_empty() || tail.contains('/') {
        return None;
    }
    Some(head.to_string())
}

/// Parse a local status URI of the form
/// `{public_url}/users/{handle}/statuses/{id}`, returning `(handle, id)` when
/// the URI is hosted on this instance and the path matches the canonical
/// shape. Returns [`None`] for remote or malformed URIs.
pub fn parse_local_status_uri(uri: &str) -> CanisterResult<Option<(String, u64)>> {
    let public_url = crate::settings::get_public_url()?;
    let prefix = format!("{public_url}/users/");
    let Some(rest) = uri.strip_prefix(&prefix) else {
        return Ok(None);
    };
    let Some((handle, tail)) = rest.split_once("/statuses/") else {
        return Ok(None);
    };
    if handle.is_empty() || handle.contains('/') {
        return Ok(None);
    }
    let Ok(id) = tail.parse::<u64>() else {
        return Ok(None);
    };
    Ok(Some((handle.to_string(), id)))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::setup;

    #[test]
    fn test_actor_uri() {
        setup();
        assert_eq!(
            actor_uri("alice").unwrap(),
            "https://mastic.social/users/alice"
        );
    }

    #[test]
    fn test_inbox_url() {
        setup();
        assert_eq!(
            inbox_url("alice").unwrap(),
            "https://mastic.social/users/alice/inbox"
        );
    }

    #[test]
    fn test_outbox_url() {
        setup();
        assert_eq!(
            outbox_url("alice").unwrap(),
            "https://mastic.social/users/alice/outbox"
        );
    }

    #[test]
    fn test_followers_url() {
        setup();
        assert_eq!(
            followers_url("alice").unwrap(),
            "https://mastic.social/users/alice/followers"
        );
    }

    #[test]
    fn test_inbox_url_from_actor_uri() {
        assert_eq!(
            inbox_url_from_actor_uri("https://mastic.social/users/alice"),
            "https://mastic.social/users/alice/inbox"
        );
    }

    #[test]
    fn test_following_url() {
        setup();
        assert_eq!(
            following_url("alice").unwrap(),
            "https://mastic.social/users/alice/following"
        );
    }

    #[test]
    fn test_status_uri() {
        setup();
        assert_eq!(
            status_uri("alice", 42).unwrap(),
            "https://mastic.social/users/alice/statuses/42"
        );
    }

    #[test]
    fn test_actor_uri_from_status_uri() {
        assert_eq!(
            actor_uri_from_status_uri("https://mastic.social/users/alice/statuses/42").as_deref(),
            Some("https://mastic.social/users/alice")
        );
    }

    #[test]
    fn test_actor_uri_from_status_uri_rejects_subpath() {
        assert!(
            actor_uri_from_status_uri("https://mastic.social/users/alice/statuses/42/foo")
                .is_none()
        );
    }

    #[test]
    fn test_actor_uri_from_status_uri_rejects_missing_id() {
        assert!(actor_uri_from_status_uri("https://mastic.social/users/alice/statuses/").is_none());
    }

    #[test]
    fn test_parse_local_status_uri_ok() {
        setup();
        let parsed =
            parse_local_status_uri("https://mastic.social/users/alice/statuses/42").unwrap();
        assert_eq!(parsed, Some(("alice".to_string(), 42)));
    }

    #[test]
    fn test_parse_local_status_uri_rejects_remote() {
        setup();
        let parsed =
            parse_local_status_uri("https://other.example/users/alice/statuses/42").unwrap();
        assert!(parsed.is_none());
    }

    #[test]
    fn test_parse_local_status_uri_rejects_non_numeric_id() {
        setup();
        let parsed =
            parse_local_status_uri("https://mastic.social/users/alice/statuses/abc").unwrap();
        assert!(parsed.is_none());
    }

    #[test]
    fn test_parse_local_status_uri_rejects_unexpected_path() {
        setup();
        let parsed = parse_local_status_uri("https://mastic.social/users/alice/inbox").unwrap();
        assert!(parsed.is_none());
    }
}
