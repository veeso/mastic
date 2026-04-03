//! Status domain

mod publish;
mod repository;

pub use self::publish::publish_status;
pub use self::repository::StatusRepository;

/// Maximum allowed length for the status content.
pub const MAX_STATUS_LENGTH: usize = 500;
