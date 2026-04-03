/// A convenient alias for results returned by canister methods.
pub type CanisterResult<T> = Result<T, CanisterError>;

/// Errors that can occur in the user canister.
#[derive(Debug, thiserror::Error)]
pub enum CanisterError {
    /// A Schnorr management canister call failed.
    #[error("Schnorr call failed: {0}")]
    #[allow(dead_code, reason = "will be used by upcoming canister methods")]
    SchnorrCall(String),
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] wasm_dbms_api::prelude::DbmsError),
    /// Settings error.
    #[error("Settings error: {0}")]
    Settings(#[from] db_utils::settings::SettingsError),
}
