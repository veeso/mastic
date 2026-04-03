mod directory_client;
mod user_client;

use std::path::Path;

use candid::{Encode, Principal};
use did::directory::DirectoryInstallArgs;
use did::federation::FederationInstallArgs;
use did::user::UserInstallArgs;
use pocket_ic_harness::{Canister, CanisterSetup, PocketIcTestEnv, alice};

pub use self::directory_client::DirectoryClient;
pub use self::user_client::UserClient;

pub fn rey_canisteryo() -> Principal {
    Principal::from_text("duo63-t5gbk-nptmp-gq7dy-saoed-ni2jl-5uuzr-ikrjk-o6vhp-2c3p5-pqe")
        .expect("Failed to parse Rey canister ID")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MasticCanister {
    Directory,
    Federation,
    User,
}

impl Canister for MasticCanister {
    fn as_path(&self) -> &Path {
        match self {
            MasticCanister::Directory => Path::new("../.artifact/directory.wasm.gz"),
            MasticCanister::Federation => Path::new("../.artifact/federation.wasm.gz"),
            MasticCanister::User => Path::new("../.artifact/user.wasm.gz"),
        }
    }

    fn all_canisters() -> &'static [Self] {
        &[Self::Directory, Self::Federation, Self::User]
    }
}

pub struct MasticCanisterSetup;

impl CanisterSetup for MasticCanisterSetup {
    type Canister = MasticCanister;

    async fn setup(env: &mut PocketIcTestEnv<Self>) {
        let directory_canister = env.canister_id(&MasticCanister::Directory);
        let federation_canister = env.canister_id(&MasticCanister::Federation);

        // install the Directory canister first, since the Federation canister depends on it
        let directory_init_args = DirectoryInstallArgs::Init {
            initial_moderator: alice(),
            federation_canister,
        };
        let directory_init_args =
            Encode!(&directory_init_args).expect("Failed to encode directory init args");
        env.install_canister(MasticCanister::Directory, directory_init_args)
            .await;

        // install the Federation canister with the Directory canister's ID as an argument
        let federation_init_args = FederationInstallArgs::Init {
            directory_canister,
            public_url: "https://mastic.social".to_string(),
        };
        let federation_init_args =
            Encode!(&federation_init_args).expect("Failed to encode federation init args");
        env.install_canister(MasticCanister::Federation, federation_init_args)
            .await;

        // install a user canister with the Federation canister's ID as an argument
        let user_init_args = Encode!(&UserInstallArgs::Init {
            owner: rey_canisteryo(),
            federation_canister,
            handle: "rey_canisteryo".to_string(),
        })
        .expect("Failed to encode user init args");
        env.install_canister(MasticCanister::User, user_init_args)
            .await;
    }
}
