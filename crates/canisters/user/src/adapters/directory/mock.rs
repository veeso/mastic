use candid::Principal;

use crate::adapters::directory::DirectoryCanister;

#[derive(Debug)]
pub struct DirectoryCanisterMockClient;

impl DirectoryCanister for DirectoryCanisterMockClient {
    async fn resolve_handle(
        &self,
        _principal: Principal,
    ) -> Result<Option<String>, crate::adapters::directory::DirectoryCanisterClientError> {
        Ok(Some("testuser".to_string()))
    }
}
