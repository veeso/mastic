use did::federation::SendActivityArgs;

use crate::adapters::federation::FederationCanister;

#[derive(Debug)]
pub struct FederationCanisterMockClient;

impl FederationCanister for FederationCanisterMockClient {
    async fn send_activity(
        &self,
        _args: SendActivityArgs,
    ) -> Result<(), crate::adapters::federation::FederationCanisterClientError> {
        Ok(())
    }
}
