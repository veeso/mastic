//! Shared test utilities for the federation canister.

use candid::Principal;
use did::federation::FederationInstallArgs;

pub fn directory() -> Principal {
    Principal::from_text("bs5l3-6b3zu-dpqyj-p2x4a-jyg4k-goneb-afof2-y5d62-skt67-3756q-dqe").unwrap()
}

pub fn public_url() -> String {
    "https://mastic.social".to_string()
}

pub fn setup() {
    crate::api::init(FederationInstallArgs::Init {
        directory_canister: directory(),
        public_url: public_url(),
    });
}
