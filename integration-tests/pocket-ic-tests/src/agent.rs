use ic_agent::Agent;

use crate::{PocketIcTestEnv, TestEnv};

pub async fn init_new_agent(ctx: &PocketIcTestEnv) -> Agent {
    let endpoint = ctx.endpoint().expect("context must be in live mode");

    let agent = Agent::builder()
        .with_url(endpoint)
        .build()
        .expect("Failed to create agent");

    agent
        .fetch_root_key()
        .await
        .expect("Failed to fetch root key");

    agent
}
