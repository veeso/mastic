//! User feed domain logic.

mod read_feed;
mod repository;

pub use read_feed::read_feed;
pub use repository::FeedRepository;
