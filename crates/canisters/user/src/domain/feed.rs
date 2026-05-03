//! User feed domain logic.

mod read_feed;
mod repository;

pub use read_feed::read_feed;
#[allow(unused_imports)]
pub use repository::FeedRepository;
