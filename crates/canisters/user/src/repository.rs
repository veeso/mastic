//! Canister-local repositories backed by the DBMS context.
//!
//! All repositories live under this module and implement the
//! [`db_utils::repository::Repository`] trait.

pub mod activity;
pub mod block;
pub mod boost;
pub mod feed;
pub mod follow_request;
pub mod follower;
pub mod following;
pub mod liked;
pub mod profile;
pub mod status;
