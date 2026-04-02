/// A convenient alias for results returned by canister methods.
pub type CanisterResult<T> = Result<T, CanisterError>;

/// Errors that can occur in the user canister.
#[derive(Debug, thiserror::Error)]
pub enum CanisterError {
    /// Settings error
    #[error("Settings error: {0}")]
    Settings(#[from] db_utils::settings::SettingsError),
}
