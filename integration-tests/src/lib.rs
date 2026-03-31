use std::path::Path;

use candid::Encode;
use pocket_ic_harness::{Canister, CanisterSetup, PocketIcTestEnv};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MasticCanister {
    Directory,
    Federation,
}

impl Canister for MasticCanister {
    fn as_path(&self) -> &Path {
        match self {
            MasticCanister::Directory => Path::new("../.artifact/directory.wasm.gz"),
            MasticCanister::Federation => Path::new("../.artifact/federation.wasm.gz"),
        }
    }
}

pub struct MasticCanisterSetup;

impl CanisterSetup for MasticCanisterSetup {
    type Canister = MasticCanister;

    async fn setup(env: &mut PocketIcTestEnv<Self>) {
        let directory_init_args = Encode!(&()).expect("Failed to encode directory init args");
        env.install_canister(MasticCanister::Directory, directory_init_args)
            .await;

        let federation_init_args = Encode!(&()).expect("Failed to encode federation init args");
        env.install_canister(MasticCanister::Federation, federation_init_args)
            .await;
    }
}
