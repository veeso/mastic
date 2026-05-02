//! User canister domains

/// Maximum number of items that can be requested in a single paginated query.
pub const MAX_PAGE_LIMIT: u64 = 50;

pub mod activity;
pub mod block;
pub mod boost;
pub mod ed25519;
pub mod feed;
pub mod follow_request;
pub mod follower;
pub mod following;
pub mod liked;
pub mod profile;
pub mod snowflake;
pub mod status;
pub mod urls;
