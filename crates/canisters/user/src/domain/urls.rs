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

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "will be used by upcoming canister methods")
)]
/// Build the outbox URL for a user: `{public_url}/users/{handle}/outbox`.
pub fn outbox_url(handle: &str) -> CanisterResult<String> {
    let public_url = crate::settings::get_public_url()?;
    Ok(format!("{public_url}/users/{handle}/outbox"))
}

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "will be used by upcoming canister methods")
)]
/// Build the followers collection URL: `{public_url}/users/{handle}/followers`.
pub fn followers_url(handle: &str) -> CanisterResult<String> {
    let public_url = crate::settings::get_public_url()?;
    Ok(format!("{public_url}/users/{handle}/followers"))
}

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "will be used by upcoming canister methods")
)]
/// Build the following collection URL: `{public_url}/users/{handle}/following`.
pub fn following_url(handle: &str) -> CanisterResult<String> {
    let public_url = crate::settings::get_public_url()?;
    Ok(format!("{public_url}/users/{handle}/following"))
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
    fn test_following_url() {
        setup();
        assert_eq!(
            following_url("alice").unwrap(),
            "https://mastic.social/users/alice/following"
        );
    }
}
