use candid::{Encode, Principal};
use did::directory::{RetrySignUpResponse, SignUpRequest, SignUpResponse};
use pocket_ic_harness::PocketIcTestEnv;

use crate::{MasticCanister, MasticCanisterSetup};

pub struct DirectoryClient<'a> {
    env: &'a PocketIcTestEnv<MasticCanisterSetup>,
}

impl<'a> DirectoryClient<'a> {
    pub fn new(env: &'a PocketIcTestEnv<MasticCanisterSetup>) -> Self {
        Self { env }
    }
}

impl DirectoryClient<'_> {
    pub async fn sign_up(&self, user: Principal, handle: String) -> SignUpResponse {
        let args = SignUpRequest { handle };

        self.env
            .update(
                self.canister_id(),
                user,
                "sign_up",
                Encode!(&args).expect("Failed to encode sign up args"),
            )
            .await
            .expect("Failed to call sign_up")
    }

    pub async fn retry_sign_up(&self, user: Principal) -> RetrySignUpResponse {
        self.env
            .update(self.canister_id(), user, "retry_sign_up", vec![])
            .await
            .expect("Failed to call retry_sign_up")
    }

    fn canister_id(&self) -> Principal {
        self.env.canister_id(&MasticCanister::Directory)
    }
}
