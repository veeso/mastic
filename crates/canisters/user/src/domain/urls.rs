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
}
