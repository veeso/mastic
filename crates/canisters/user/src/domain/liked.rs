//! Liked status domain.

mod get_liked;
mod like;
mod unlike;

pub use get_liked::get_liked;
pub use like::like_status;
pub use unlike::unlike_status;
