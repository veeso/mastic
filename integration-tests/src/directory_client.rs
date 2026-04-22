use candid::{Encode, Principal};
use did::directory::{
    DeleteProfileResponse, GetUserArgs, GetUserResponse, RetryDeleteProfileResponse,
    RetrySignUpResponse, SignUpRequest, SignUpResponse, UserCanisterResponse, WhoAmIResponse,
};
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

    pub async fn delete_profile(&self, user: Principal) -> DeleteProfileResponse {
        self.env
            .update(self.canister_id(), user, "delete_profile", vec![])
            .await
            .expect("Failed to call delete_profile")
    }

    pub async fn retry_delete_profile(&self, user: Principal) -> RetryDeleteProfileResponse {
        self.env
            .update(self.canister_id(), user, "retry_delete_profile", vec![])
            .await
            .expect("Failed to call retry_delete_profile")
    }

    pub async fn get_user(&self, args: GetUserArgs) -> GetUserResponse {
        self.env
            .query(
                self.canister_id(),
                Principal::anonymous(),
                "get_user",
                Encode!(&args).expect("Failed to encode get_user args"),
            )
            .await
            .expect("Failed to call get_user")
    }

    pub async fn retry_sign_up(&self, user: Principal) -> RetrySignUpResponse {
        self.env
            .update(self.canister_id(), user, "retry_sign_up", vec![])
            .await
            .expect("Failed to call retry_sign_up")
    }

    pub async fn user_canister(
        &self,
        caller: Principal,
        principal: Option<Principal>,
    ) -> UserCanisterResponse {
        self.env
            .query(
                self.canister_id(),
                caller,
                "user_canister",
                Encode!(&principal).expect("Failed to encode user_canister args"),
            )
            .await
            .expect("Failed to call user_canister")
    }

    pub async fn whoami(&self, user: Principal) -> WhoAmIResponse {
        self.env
            .query(self.canister_id(), user, "whoami", vec![])
            .await
            .expect("Failed to call whoami")
    }

    fn canister_id(&self) -> Principal {
        self.env.canister_id(&MasticCanister::Directory)
    }
}
