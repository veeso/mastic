//! Canister-local repositories backed by the DBMS context.
//!
//! All repositories live under this module and implement the
//! [`db_utils::repository::Repository`] trait.

pub mod moderators;
pub mod tombstone;
pub mod users;
