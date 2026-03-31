use integration_tests::{MasticCanister, MasticCanisterSetup};
use pocket_ic_harness::PocketIcTestEnv;

#[pocket_ic_harness::test]
async fn test_should_init_canisters(env: PocketIcTestEnv<MasticCanisterSetup>) {
    let _directory_canister_id = env.canister_id(&MasticCanister::Directory);
    let _federation_canister_id = env.canister_id(&MasticCanister::Federation);
}
