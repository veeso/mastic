use candid::{Encode, Principal};
use did::common::Visibility;
use did::user::{GetProfileResponse, PublishStatusArgs, PublishStatusResponse};
use pocket_ic_harness::PocketIcTestEnv;

use crate::MasticCanisterSetup;

pub struct UserClient<'a> {
    env: &'a PocketIcTestEnv<MasticCanisterSetup>,
    canister_id: Principal,
}

impl<'a> UserClient<'a> {
    pub fn new(env: &'a PocketIcTestEnv<MasticCanisterSetup>, canister_id: Principal) -> Self {
        Self { env, canister_id }
    }
}

impl UserClient<'_> {
    pub async fn get_profile(&self, user: Principal) -> GetProfileResponse {
        self.env
            .query(self.canister_id, user, "get_profile", vec![])
            .await
            .expect("Failed to call get_profile")
    }

    pub async fn publish_status(
        &self,
        caller: Principal,
        content: String,
        visibility: Visibility,
        mentions: Vec<String>,
    ) -> PublishStatusResponse {
        let args = PublishStatusArgs {
            content,
            visibility,
            mentions,
        };

        self.env
            .update(
                self.canister_id,
                caller,
                "publish_status",
                Encode!(&args).expect("Failed to encode publish_status arguments"),
            )
            .await
            .expect("Failed to call publish_status")
    }
}
