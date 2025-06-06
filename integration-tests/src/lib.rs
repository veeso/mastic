use candid::{CandidType, Principal};
use serde::de::DeserializeOwned;

pub mod actor;
#[cfg(feature = "pocket-ic")]
mod pocket_ic;
mod wasm;

#[cfg(feature = "pocket-ic")]
pub use self::pocket_ic::PocketIcTestEnv;

pub trait TestEnv {
    fn query<R>(
        &self,
        canister: Principal,
        caller: Principal,
        method: &str,
        payload: Vec<u8>,
    ) -> impl Future<Output = anyhow::Result<R>>
    where
        R: DeserializeOwned + CandidType;

    fn update<R>(
        &self,
        canister: Principal,
        caller: Principal,
        method: &str,
        payload: Vec<u8>,
    ) -> impl Future<Output = anyhow::Result<R>>
    where
        R: DeserializeOwned + CandidType;

    /// Get the principal of the orbit station canister.
    fn orbit_station(&self) -> Principal;

    /// Get the principal of the federation canister.
    fn federation(&self) -> Principal;

    /// Get the principal of the directory canister.
    fn directory(&self) -> Principal;

    /// Get the uuid of the station admin.
    fn orbit_station_admin(&self) -> &str;
}
