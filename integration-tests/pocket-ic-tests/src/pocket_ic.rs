mod env;

use std::io::Read as _;
use std::path::PathBuf;

use candid::{CandidType, Decode, Encode, Principal};
use pocket_ic::nonblocking::PocketIc;
use serde::de::DeserializeOwned;

use crate::TestEnv;
use crate::actor::{admin, alice, bob};
use crate::wasm::Canister;

const DEFAULT_CYCLES: u128 = 2_000_000_000_000_000;

/// Test environment
pub struct PocketIcTestEnv {
    pub pic: PocketIc,
}

impl TestEnv for PocketIcTestEnv {
    async fn query<R>(
        &self,
        canister: Principal,
        caller: Principal,
        method: &str,
        payload: Vec<u8>,
    ) -> anyhow::Result<R>
    where
        R: DeserializeOwned + CandidType,
    {
        let reply = match self.pic.query_call(canister, caller, method, payload).await {
            Ok(result) => result,
            Err(e) => anyhow::bail!("Error calling {}: {:?}", method, e),
        };
        let ret_type = Decode!(&reply, R)?;

        Ok(ret_type)
    }

    async fn update<R>(
        &self,
        canister: Principal,
        caller: Principal,
        method: &str,
        payload: Vec<u8>,
    ) -> anyhow::Result<R>
    where
        R: DeserializeOwned + CandidType,
    {
        let reply = if self.is_live() {
            let id = self
                .pic
                .submit_call(canister, caller, method, payload)
                .await
                .map_err(|e| anyhow::anyhow!("Error submitting call {}: {:?}", method, e))?;
            self.pic.await_call_no_ticks(id).await
        } else {
            self.pic
                .update_call(canister, caller, method, payload)
                .await
        };

        let reply = match reply {
            Ok(r) => r,
            Err(r) => anyhow::bail!("{} was rejected: {:?}", method, r),
        };
        let ret_type = Decode!(&reply, R)?;

        Ok(ret_type)
    }

    fn admin(&self) -> Principal {
        admin()
    }
    fn bob(&self) -> Principal {
        bob()
    }

    fn alice(&self) -> Principal {
        alice()
    }

    fn endpoint(&self) -> Option<url::Url> {
        self.pic.url()
    }
}

impl PocketIcTestEnv {
    /// Install the canisters needed for the tests
    pub async fn init() -> Self {
        let pic = env::init_pocket_ic()
            .await
            .with_nns_subnet()
            .with_ii_subnet()
            .with_fiduciary_subnet()
            .with_application_subnet()
            .with_max_request_time_ms(Some(30_000))
            .build_async()
            .await;

        // create canisters

        // install canisters

        Self { pic }
    }

    /// Stop instance -  Should be called after each test
    pub async fn stop(self) {
        self.pic.drop().await
    }

    fn is_live(&self) -> bool {
        self.pic.url().is_some()
    }

    fn load_wasm(canister: Canister) -> Vec<u8> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(canister.as_path());

        let mut file = std::fs::File::open(path).expect("Failed to open wasm file");
        let mut wasm_bytes = Vec::new();
        file.read_to_end(&mut wasm_bytes)
            .expect("Failed to read wasm file");

        wasm_bytes
    }

    pub async fn live(&mut self, live: bool) {
        if live {
            self.pic.make_live(None).await;
        } else {
            self.pic.stop_live().await;
        }
    }
}
