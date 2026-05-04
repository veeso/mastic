//! Following domain logic.

mod follow_user;
mod get_following;
mod unfollow_user;

pub use self::follow_user::follow_user;
pub use self::get_following::get_following;
pub use self::unfollow_user::unfollow_user;
