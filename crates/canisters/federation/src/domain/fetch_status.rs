//! `fetch_status` flow — resolve an ActivityPub status URI to a [`Status`].
//!
//! - Local URI (host == public_url): directory lookup → user canister
//!   inter-canister call to `get_local_status`.
//! - Remote URI: returns [`FetchStatusError::Unsupported`]. M3 will replace
//!   this branch with an HTTPS outcall.

use did::common::Status;
use did::federation::{FetchStatusArgs, FetchStatusError, FetchStatusResponse};
use did::user::{GetLocalStatusArgs, GetLocalStatusResponse};
use url::Url;

use crate::adapters::user::UserCanisterClientError;

/// Entry point for the `fetch_status` canister method.
///
/// Resolves `args.uri` to a [`Status`] via the local directory + user
/// canister. Remote URIs return [`FetchStatusError::Unsupported`] until
/// HTTPS outcalls land in M3.
pub async fn fetch_status(args: FetchStatusArgs) -> FetchStatusResponse {
    match fetch_status_inner(args).await {
        Ok(status) => FetchStatusResponse::Ok(status),
        Err(failure) => failure.into(),
    }
}

async fn fetch_status_inner(
    FetchStatusArgs {
        uri,
        requester_actor_uri,
    }: FetchStatusArgs,
) -> Result<Status, FetchStatusFailure> {
    let target = Url::parse(&uri).map_err(|err| {
        ic_utils::log!("fetch_status: invalid uri {uri:?}: {err}");
        FetchStatusFailure::InvalidUri
    })?;

    let public_url_raw = crate::settings::get_public_url();
    let public_url = Url::parse(&public_url_raw).map_err(|err| {
        ic_utils::log!("fetch_status: public_url {public_url_raw:?} invalid: {err}");
        FetchStatusFailure::Internal(format!("public_url misconfigured: {err}"))
    })?;

    if !is_local(&target, &public_url) {
        ic_utils::log!("fetch_status: remote uri {target} unsupported (M3)");
        return Err(FetchStatusFailure::Unsupported);
    }

    let (handle, id) = parse_local_path(&target).ok_or_else(|| {
        ic_utils::log!(
            "fetch_status: unexpected local path shape: {}",
            target.path()
        );
        FetchStatusFailure::InvalidUri
    })?;

    let Some(user) = crate::directory::get_user_by_handle(&handle) else {
        ic_utils::log!("fetch_status: unknown local handle {handle}");
        return Err(FetchStatusFailure::NotFound);
    };

    let response = call_get_local_status(
        user.user_canister_id,
        GetLocalStatusArgs {
            id,
            requester_actor_uri,
        },
    )
    .await
    .map_err(|err| {
        ic_utils::log!("fetch_status: get_local_status call failed: {err}");
        FetchStatusFailure::Internal(err.to_string())
    })?;

    match response {
        GetLocalStatusResponse::Ok(status) => Ok(status),
        GetLocalStatusResponse::Err(_) => Err(FetchStatusFailure::NotFound),
    }
}

/// Native (test) build: route through the mock client so unit tests can
/// drive the canned-response queue.
#[cfg(not(target_family = "wasm"))]
async fn call_get_local_status(
    _canister_id: candid::Principal,
    args: GetLocalStatusArgs,
) -> Result<GetLocalStatusResponse, UserCanisterClientError> {
    use crate::adapters::user::UserCanister;
    use crate::adapters::user::mock::MockUserCanisterClient;
    MockUserCanisterClient.get_local_status(args).await
}

/// Wasm build: perform the real inter-canister query via
/// [`crate::adapters::user::IcUserCanisterClient`].
#[cfg(target_family = "wasm")]
async fn call_get_local_status(
    canister_id: candid::Principal,
    args: GetLocalStatusArgs,
) -> Result<GetLocalStatusResponse, UserCanisterClientError> {
    use crate::adapters::user::{IcUserCanisterClient, UserCanister};
    IcUserCanisterClient::from(canister_id)
        .get_local_status(args)
        .await
}

/// Returns `true` when `target` resolves to this Mastic instance.
fn is_local(target: &Url, public_url: &Url) -> bool {
    target.host_str() == public_url.host_str()
        && target.port_or_known_default() == public_url.port_or_known_default()
}

/// Parse a local status URL of shape `<public_url>/users/<handle>/statuses/<id>`.
fn parse_local_path(target: &Url) -> Option<(String, u64)> {
    let mut segments = target.path_segments()?;
    let users = segments.next()?;
    let handle = segments.next()?;
    let statuses = segments.next()?;
    let id = segments.next()?;
    if segments.next().is_some() || users != "users" || statuses != "statuses" || handle.is_empty()
    {
        return None;
    }
    let id = id.parse::<u64>().ok()?;
    Some((handle.to_string(), id))
}

#[derive(Debug, thiserror::Error)]
enum FetchStatusFailure {
    #[error("unsupported (remote URI)")]
    Unsupported,
    #[error("invalid uri")]
    InvalidUri,
    #[error("not found")]
    NotFound,
    #[error("internal: {0}")]
    Internal(String),
}

impl From<FetchStatusFailure> for FetchStatusResponse {
    fn from(value: FetchStatusFailure) -> Self {
        FetchStatusResponse::Err(match value {
            FetchStatusFailure::Unsupported => FetchStatusError::Unsupported,
            FetchStatusFailure::InvalidUri => FetchStatusError::InvalidUri,
            FetchStatusFailure::NotFound => FetchStatusError::NotFound,
            FetchStatusFailure::Internal(s) => FetchStatusError::Internal(s),
        })
    }
}

#[cfg(test)]
mod tests {
    use candid::Principal;
    use did::common::{Status, Visibility};
    use did::federation::{FetchStatusArgs, FetchStatusError, FetchStatusResponse};
    use did::user::{GetLocalStatusError, GetLocalStatusResponse};

    use super::fetch_status;
    use crate::adapters::user::mock::push_get_local_status_response;
    use crate::test_utils::{alice, public_url, setup};

    fn fixture_status() -> Status {
        Status {
            id: 42,
            content: "hi".into(),
            author: format!("{}/users/alice", public_url()),
            created_at: 0,
            visibility: Visibility::Public,
            like_count: 0,
            boost_count: 0,
            spoiler_text: None,
            sensitive: false,
        }
    }

    fn register_alice() {
        crate::directory::insert_user(
            Principal::from_text("mfufu-x6j4c-gomzb-geilq").unwrap(),
            "alice".to_string(),
            alice(),
        );
    }

    // M-UNIT-TEST: a local URI is resolved through the directory and forwarded
    // to the target user canister.
    #[tokio::test]
    async fn test_local_uri_resolves_via_directory() {
        setup();
        register_alice();
        push_get_local_status_response(GetLocalStatusResponse::Ok(fixture_status()));

        let resp = fetch_status(FetchStatusArgs {
            uri: format!("{}/users/alice/statuses/42", public_url()),
            requester_actor_uri: None,
        })
        .await;

        assert!(matches!(resp, FetchStatusResponse::Ok(_)));
    }

    // M-UNIT-TEST: a remote URI is reported as Unsupported until HTTPS outcalls
    // are wired up in M3.
    #[tokio::test]
    async fn test_remote_uri_returns_unsupported() {
        setup();
        let resp = fetch_status(FetchStatusArgs {
            uri: "https://other.example/users/bob/statuses/9".to_string(),
            requester_actor_uri: None,
        })
        .await;
        assert_eq!(
            resp,
            FetchStatusResponse::Err(FetchStatusError::Unsupported)
        );
    }

    // M-UNIT-TEST: an unparseable URI yields InvalidUri.
    #[tokio::test]
    async fn test_invalid_uri_returns_invalid() {
        setup();
        let resp = fetch_status(FetchStatusArgs {
            uri: "not-a-url".to_string(),
            requester_actor_uri: None,
        })
        .await;
        assert_eq!(resp, FetchStatusResponse::Err(FetchStatusError::InvalidUri));
    }

    // M-UNIT-TEST: a local URI for an unknown handle yields NotFound.
    #[tokio::test]
    async fn test_unknown_handle_returns_not_found() {
        setup();
        let resp = fetch_status(FetchStatusArgs {
            uri: format!("{}/users/unknown/statuses/1", public_url()),
            requester_actor_uri: None,
        })
        .await;
        assert_eq!(resp, FetchStatusResponse::Err(FetchStatusError::NotFound));
    }

    // M-UNIT-TEST: a NotFound response from the target user canister bubbles
    // up unchanged.
    #[tokio::test]
    async fn test_target_user_canister_returns_not_found() {
        setup();
        register_alice();
        push_get_local_status_response(GetLocalStatusResponse::Err(GetLocalStatusError::NotFound));

        let resp = fetch_status(FetchStatusArgs {
            uri: format!("{}/users/alice/statuses/42", public_url()),
            requester_actor_uri: None,
        })
        .await;
        assert_eq!(resp, FetchStatusResponse::Err(FetchStatusError::NotFound));
    }

    // M-UNIT-TEST: a local URL whose path is not /users/{handle}/statuses/{id}
    // is rejected as InvalidUri.
    #[tokio::test]
    async fn test_unexpected_path_shape_returns_invalid_uri() {
        setup();
        let resp = fetch_status(FetchStatusArgs {
            uri: format!("{}/users/alice/foo", public_url()),
            requester_actor_uri: None,
        })
        .await;
        assert_eq!(resp, FetchStatusResponse::Err(FetchStatusError::InvalidUri));
    }
}
