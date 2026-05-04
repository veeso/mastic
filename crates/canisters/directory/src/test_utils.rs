//! Shared test utilities for the directory canister.

use candid::Principal;
use db_utils::repository::Repository;
use did::directory::{DirectoryInstallArgs, SignUpRequest, SignUpResponse};

pub fn admin() -> Principal {
    Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").unwrap()
}

pub fn federation() -> Principal {
    Principal::from_text("bs5l3-6b3zu-dpqyj-p2x4a-jyg4k-goneb-afof2-y5d62-skt67-3756q-dqe").unwrap()
}

pub fn rey_canisteryo() -> Principal {
    Principal::from_text("bkyz2-fmaaa-aaaaa-qaaaq-cai").unwrap()
}

pub fn bob() -> Principal {
    Principal::from_text("br5f7-7uaaa-aaaaa-qaaca-cai").unwrap()
}

pub fn setup() {
    crate::api::init(DirectoryInstallArgs::Init {
        initial_moderator: admin(),
        federation_canister: federation(),
        public_url: "https://mastic.social".to_string(),
    });
}

/// Registers a user with the given principal and handle.
///
/// The user will have canister status [`did::directory::UserCanisterStatus::CreationPending`]
/// and no canister ID assigned.
///
/// # Panics
///
/// Panics if the sign-up fails.
pub fn setup_registered_user(principal: Principal, handle: &str) {
    let response = crate::domain::users::sign_up(
        principal,
        SignUpRequest {
            handle: handle.to_string(),
        },
    );
    assert_eq!(response, SignUpResponse::Ok, "setup_registered_user failed");
}

/// Registers a user with the given principal and handle, then assigns a canister ID
/// and sets the canister status to [`did::directory::UserCanisterStatus::Active`].
///
/// # Panics
///
/// Panics if sign-up or canister assignment fails.
pub fn setup_registered_user_with_canister(
    principal: Principal,
    handle: &str,
    canister_id: Principal,
) {
    setup_registered_user(principal, handle);
    crate::repository::users::UserRepository::oneshot()
        .set_user_canister(principal, canister_id)
        .expect("setup_registered_user_with_canister: failed to set user canister");
}
