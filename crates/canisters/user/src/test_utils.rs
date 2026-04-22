//! Shared test utilities for the user canister.

use candid::Principal;
use did::user::UserInstallArgs;

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

pub fn directory() -> Principal {
    Principal::from_text("bw4dl-smaaa-aaaaa-qaacq-cai").unwrap()
}

pub fn setup() {
    crate::adapters::federation::mock::reset_captured();
    crate::api::init(UserInstallArgs::Init {
        owner: admin(),
        federation_canister: federation(),
        directory_canister: directory(),
        handle: "rey_canisteryo".to_string(),
        public_url: "https://mastic.social".to_string(),
    });
}

pub fn insert_status(id: u64, content: &str, visibility: did::common::Visibility, created_at: u64) {
    use ic_dbms_canister::prelude::DBMS_CONTEXT;
    use wasm_dbms::WasmDbmsDatabase;
    use wasm_dbms_api::prelude::{Database, Nullable};

    use crate::schema::{Schema, Status, StatusInsertRequest, Visibility as DbVisibility};

    DBMS_CONTEXT.with(|ctx| {
        let db = WasmDbmsDatabase::oneshot(ctx, Schema);
        db.insert::<Status>(StatusInsertRequest {
            id: id.into(),
            content: content.into(),
            visibility: DbVisibility::from(visibility),
            like_count: 0u64.into(),
            boost_count: 0u64.into(),
            in_reply_to_uri: Nullable::Null,
            spoiler_text: Nullable::Null,
            sensitive: false.into(),
            edited_at: Nullable::Null,
            created_at: created_at.into(),
        })
        .expect("should insert status");
    });
}
