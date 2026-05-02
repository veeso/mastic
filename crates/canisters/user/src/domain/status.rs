//! Status domain

mod get_local_status;
mod get_statuses;
mod publish;
mod repository;

/// Maximum allowed length for the status content.
pub const MAX_STATUS_LENGTH: usize = 500;

pub use self::get_local_status::{get_local_status, get_local_status_with_caller};
pub use self::get_statuses::get_statuses;
pub use self::publish::publish_status;
pub use self::repository::StatusRepository;
