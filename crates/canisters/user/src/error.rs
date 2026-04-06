/// A convenient alias for results returned by canister methods.
pub type CanisterResult<T> = Result<T, CanisterError>;

/// Errors that can occur in the user canister.
#[derive(Debug, thiserror::Error)]
pub enum CanisterError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] wasm_dbms_api::prelude::DbmsError),
    /// Directory error
    #[error("Directory call failed: {0}")]
    #[cfg(any(target_family = "wasm", test))]
    Directory(#[from] crate::adapters::directory::DirectoryCanisterClientError),
    /// Federation error
    #[error("Federation call failed: {0}")]
    #[cfg(any(target_family = "wasm", test))]
    Federation(#[from] crate::adapters::federation::FederationCanisterClientError),
    /// A Schnorr management canister call failed.
    #[error("Schnorr call failed: {0}")]
    #[allow(dead_code, reason = "will be used by upcoming canister methods")]
    SchnorrCall(String),
    /// Settings error.
    #[error("Settings error: {0}")]
    Settings(#[from] db_utils::settings::SettingsError),
}
