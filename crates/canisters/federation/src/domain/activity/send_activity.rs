//! Domain logic for the `send_activity` flow.
//!
//! The Federation Canister acts as a dumb router for locally-originated
//! activities: User Canister code (installed and controlled by the
//! Directory Canister) is trusted to pre-filter recipients according to
//! the status's visibility, so this module performs no visibility
//! enforcement and does not parse the activity JSON. It parses the target
//! inbox URL, resolves local targets via the Directory Canister, and
//! forwards the opaque `activity_json` to the target User Canister. Remote
//! targets are logged and skipped (remote HTTP delivery is Milestone 2).

use did::federation::{
    SendActivityArgs, SendActivityArgsObject, SendActivityError, SendActivityResponse,
    SendActivityResult,
};
use did::user::ReceiveActivityArgs;
use url::Url;

/// Entry point for the `send_activity` canister method.
///
/// Dispatches each [`SendActivityArgsObject`] through
/// [`send_activity_inner`] and aggregates the per-activity results.
pub async fn send_activity(args: SendActivityArgs) -> SendActivityResponse {
    match args {
        SendActivityArgs::One(activity_args) => {
            SendActivityResponse::One(send_activity_inner(activity_args).await)
        }
        SendActivityArgs::Batch(activities) => {
            let mut results = Vec::with_capacity(activities.len());
            for activity_args in activities {
                results.push(send_activity_inner(activity_args).await);
            }
            SendActivityResponse::Batch(results)
        }
    }
}

/// Routes a single activity to its target inbox.
///
/// Parses `target_inbox`, classifies it as local or remote, and for local
/// targets resolves the User Canister principal via the Directory Canister
/// and invokes `receive_activity` on it. Remote targets are logged and
/// skipped. The activity JSON is forwarded to the target canister
/// unchanged.
async fn send_activity_inner(args: SendActivityArgsObject) -> SendActivityResult {
    let target = match Url::parse(&args.target_inbox) {
        Ok(url) => url,
        Err(err) => {
            ic_utils::log!(
                "send_activity_inner: invalid target_inbox {:?}: {err}",
                args.target_inbox
            );
            return SendActivityResult::Err(SendActivityError::InvalidTargetInbox(err.to_string()));
        }
    };

    let public_url_raw = crate::settings::get_public_url();
    let public_url = match Url::parse(&public_url_raw) {
        Ok(url) => url,
        Err(err) => {
            ic_utils::log!(
                "send_activity_inner: public_url {public_url_raw:?} is not a valid URL: {err}"
            );
            return SendActivityResult::Err(SendActivityError::DeliveryFailed(format!(
                "public_url misconfigured: {err}"
            )));
        }
    };

    if !is_local_target(&target, &public_url) {
        ic_utils::log!(
            "send_activity_inner: skipping remote target {target} (Mastic is currently local-only)"
        );
        // TODO: forward to remote instance when multi-instance support is implemented.
        return SendActivityResult::Ok;
    }

    let handle = match extract_handle(&target) {
        Some(handle) => handle,
        None => {
            ic_utils::log!("send_activity_inner: unexpected path shape for local inbox {target}");
            return SendActivityResult::Err(SendActivityError::InvalidTargetInbox(format!(
                "unexpected path shape: {}",
                target.path()
            )));
        }
    };

    let Some(user) = crate::directory::get_user_by_handle(&handle) else {
        ic_utils::log!("send_activity_inner: unknown local handle {handle}");
        return SendActivityResult::Err(SendActivityError::UnknownLocalUser(handle));
    };

    let receive_args = ReceiveActivityArgs {
        activity_json: args.activity_json,
    };

    deliver(user.user_canister_id, receive_args).await
}

/// Returns `true` when the `target` inbox is hosted on this Mastic instance.
fn is_local_target(target: &Url, public_url: &Url) -> bool {
    target.host_str() == public_url.host_str()
        && target.port_or_known_default() == public_url.port_or_known_default()
}

/// Extracts the handle from an inbox URL whose path is exactly
/// `/users/{handle}/inbox`, returning `None` on any other path shape.
fn extract_handle(target: &Url) -> Option<String> {
    let mut segments = target.path_segments()?;
    let users = segments.next()?;
    let handle = segments.next()?;
    let inbox = segments.next()?;
    if segments.next().is_some() || users != "users" || inbox != "inbox" || handle.is_empty() {
        return None;
    }
    Some(handle.to_string())
}

/// Invokes `receive_activity` on the target User Canister and translates
/// the client-level outcome into a [`SendActivityResult`].
///
/// On non-wasm targets this uses
/// [`crate::adapters::user::mock::MockUserCanisterClient`] so unit tests
/// can exercise the routing logic without a replica. On wasm targets it
/// uses [`crate::adapters::user::IcUserCanisterClient`] which performs the
/// actual inter-canister call.
#[cfg(not(target_family = "wasm"))]
async fn deliver(
    _user_canister_id: candid::Principal,
    args: ReceiveActivityArgs,
) -> SendActivityResult {
    use crate::adapters::user::mock::MockUserCanisterClient;
    use crate::adapters::user::{UserCanister, UserCanisterClientError};

    match MockUserCanisterClient.receive_activity(args).await {
        Ok(()) => SendActivityResult::Ok,
        Err(UserCanisterClientError::Rejected(e)) => {
            SendActivityResult::Err(SendActivityError::Rejected(e))
        }
        Err(e) => SendActivityResult::Err(SendActivityError::DeliveryFailed(e.to_string())),
    }
}

/// Wasm implementation of [`deliver`] that performs the real
/// inter-canister call via [`crate::adapters::user::IcUserCanisterClient`].
#[cfg(target_family = "wasm")]
async fn deliver(
    user_canister_id: candid::Principal,
    args: ReceiveActivityArgs,
) -> SendActivityResult {
    use crate::adapters::user::{IcUserCanisterClient, UserCanister, UserCanisterClientError};

    match IcUserCanisterClient::from(user_canister_id)
        .receive_activity(args)
        .await
    {
        Ok(()) => SendActivityResult::Ok,
        Err(UserCanisterClientError::Rejected(e)) => {
            SendActivityResult::Err(SendActivityError::Rejected(e))
        }
        Err(e) => SendActivityResult::Err(SendActivityError::DeliveryFailed(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use candid::Principal;

    use super::*;
    use crate::test_utils::{public_url, setup};

    fn alice_canister() -> Principal {
        Principal::from_text("bkyz2-fmaaa-aaaaa-qaaaq-cai").unwrap()
    }

    fn alice_user_principal() -> Principal {
        Principal::from_text("mfufu-x6j4c-gomzb-geilq").unwrap()
    }

    fn seed_alice() {
        crate::directory::insert_user(
            alice_user_principal(),
            "alice".to_string(),
            alice_canister(),
        );
    }

    fn obj(target_inbox: &str) -> SendActivityArgsObject {
        SendActivityArgsObject {
            activity_json: r#"{"type":"Create"}"#.to_string(),
            target_inbox: target_inbox.to_string(),
        }
    }

    // M-UNIT-TEST: an unparseable target_inbox yields InvalidTargetInbox.
    #[tokio::test]
    async fn test_should_reject_bad_target_inbox_url() {
        setup();

        let result = send_activity_inner(obj("not a url")).await;

        assert!(matches!(
            result,
            SendActivityResult::Err(SendActivityError::InvalidTargetInbox(_))
        ));
    }

    // M-UNIT-TEST: a remote target (different host) is skipped with Ok.
    #[tokio::test]
    async fn test_should_skip_remote_target() {
        setup();

        let result = send_activity_inner(obj("https://other.social/users/alice/inbox")).await;

        assert_eq!(result, SendActivityResult::Ok);
    }

    // M-UNIT-TEST: a local URL whose path is not /users/{handle}/inbox is
    // rejected as InvalidTargetInbox.
    #[tokio::test]
    async fn test_should_reject_unexpected_local_path() {
        setup();

        let result = send_activity_inner(obj(&format!("{}/foo/bar", public_url()))).await;

        assert!(matches!(
            result,
            SendActivityResult::Err(SendActivityError::InvalidTargetInbox(_))
        ));
    }

    // M-UNIT-TEST: a local inbox whose handle is not in the directory yields
    // UnknownLocalUser.
    #[tokio::test]
    async fn test_should_reject_unknown_local_handle() {
        setup();

        let result = send_activity_inner(obj(&format!("{}/users/ghost/inbox", public_url()))).await;

        assert!(matches!(
            result,
            SendActivityResult::Err(SendActivityError::UnknownLocalUser(ref h)) if h == "ghost"
        ));
    }

    // M-UNIT-TEST: a known local handle is routed and returns Ok when the
    // target user canister accepts the activity.
    #[tokio::test]
    async fn test_should_deliver_to_known_local_handle() {
        setup();
        seed_alice();

        let result = send_activity_inner(obj(&format!("{}/users/alice/inbox", public_url()))).await;

        assert_eq!(result, SendActivityResult::Ok);
    }

    // M-UNIT-TEST: a batch call aggregates per-item routing results.
    #[tokio::test]
    async fn test_should_aggregate_batch_results() {
        setup();
        seed_alice();

        let args = SendActivityArgs::Batch(vec![
            obj(&format!("{}/users/alice/inbox", public_url())),
            obj(&format!("{}/users/ghost/inbox", public_url())),
        ]);

        let response = send_activity(args).await;

        match response {
            SendActivityResponse::Batch(results) => {
                assert_eq!(results.len(), 2);
                assert_eq!(results[0], SendActivityResult::Ok);
                assert!(matches!(
                    results[1],
                    SendActivityResult::Err(SendActivityError::UnknownLocalUser(ref h)) if h == "ghost"
                ));
            }
            other => panic!("expected Batch response, got {other:?}"),
        }
    }
}
