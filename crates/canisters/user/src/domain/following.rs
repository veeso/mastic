//! Following domain logic.

mod follow_user;
mod repository;

pub use self::follow_user::follow_user;
pub use self::repository::FollowingRepository;
