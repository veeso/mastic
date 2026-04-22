//! Block domain — stores actor URIs the owner has blocked.
//!
//! Only the repository read path is implemented in WI-1.2 (needed to filter
//! `Update(Person)` fan-out). The full `block_user` flow is WI-1.10.

mod repository;

pub use self::repository::BlockRepository;
