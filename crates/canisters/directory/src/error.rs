use wasm_dbms_api::error::DbmsError;

/// A convenient alias for results returned by canister methods.
pub type CanisterResult<T> = Result<T, CanisterError>;

/// Errors that can occur in the directory canister.
#[derive(Debug, thiserror::Error)]
pub enum CanisterError {
    /// Errors related to database operations.
    #[error("Database error: {0}")]
    Database(#[from] DbmsError),
    /// Settings error
    #[error("Settings error: {0}")]
    Settings(db_utils::settings::SettingsError),
    /// Sign up process failed for a user.
    #[error("Sign up failed: {0}")]
    SignUpFailed(String),
    /// Internal error not caused by misuse.
    #[error("Internal error: {0}")]
    #[allow(dead_code)]
    Internal(String),
}
