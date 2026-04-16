//! Adapter traits abstracting external IC calls for testability.
//!
//! Each sub-module defines a trait describing an inter-canister surface
//! the Federation Canister depends on, a production implementation that
//! delegates to `ic_cdk::call` on `wasm32-unknown-unknown` targets, and a
//! mock implementation used by unit tests on native targets. Domain code
//! selects the appropriate implementation via `cfg` gating so the same
//! call sites work in both environments.

pub mod user;
