//! Flow for boosting a status.

use did::user::{BoostStatusArgs, BoostStatusError, BoostStatusResponse};

use crate::domain::boost::repository::BoostRepository;
use crate::error::CanisterResult;

/// Boost of a status.
pub async fn boost_status(BoostStatusArgs { status_url }: BoostStatusArgs) -> BoostStatusResponse {
    match boost_status_impl(status_url).await {
        Ok(()) => BoostStatusResponse::Ok,
        Err(err) => {
            ic_utils::log!("Failed to boost status: {err}");
            BoostStatusResponse::Err(BoostStatusError::Internal(err.to_string()))
        }
    }
}

async fn boost_status_impl(status_uri: String) -> CanisterResult<()> {
    ic_utils::log!("Boosting status with URI: {status_uri}");

    // Idempotent: if already boosted, do not duplicate or re-emit.
    if BoostRepository::is_boosted(&status_uri)? {
        ic_utils::log!("Status already boosted: {status_uri}");
        return Ok(());
    }

    // Insert the boost into the database first; if federation dispatch
    // later fails, the user can re-trigger and the row already exists,
    // making the second call a no-op.
    BoostRepository::boost_status(&status_uri)?;

    Ok(())
}
