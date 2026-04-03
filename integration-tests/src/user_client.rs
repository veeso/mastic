use candid::Principal;
use did::user::GetProfileResponse;
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
}
