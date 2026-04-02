//! Shared test utilities for the directory canister.

use candid::Principal;
use did::directory::DirectoryInstallArgs;

pub fn admin() -> Principal {
    Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").unwrap()
}

pub fn federation() -> Principal {
    Principal::from_text("bs5l3-6b3zu-dpqyj-p2x4a-jyg4k-goneb-afof2-y5d62-skt67-3756q-dqe").unwrap()
}

pub fn alice() -> Principal {
    Principal::from_text("bkyz2-fmaaa-aaaaa-qaaaq-cai").unwrap()
}

pub fn bob() -> Principal {
    Principal::from_text("br5f7-7uaaa-aaaaa-qaaca-cai").unwrap()
}

pub fn setup() {
    crate::api::init(DirectoryInstallArgs::Init {
        initial_moderator: admin(),
        federation_canister: federation(),
    });
}
