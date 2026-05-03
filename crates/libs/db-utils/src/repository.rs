//! Repository trait — common shape for canister-local repositories.

use ic_dbms_canister::prelude::{IcAccessControlList, IcMemoryProvider};
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms::prelude::{DatabaseSchema, DbmsContext};
use wasm_dbms_api::prelude::TransactionId;

/// Common shape for repositories backed by the canister-local
/// `DBMS_CONTEXT`.
///
/// Implementors provide the [`DatabaseSchema`] via [`Self::schema`], the
/// two constructors [`Self::oneshot`] / [`Self::with_transaction`], and the
/// transaction-id read-back [`Self::tx`]. The trait then provides a default
/// [`Self::db`] accessor that routes between an oneshot auto-committed
/// handle and a transaction-bound handle.
pub trait Repository: Sized {
    /// Schema type used by this repository.
    type Schema: DatabaseSchema<IcMemoryProvider, IcAccessControlList> + Copy + 'static;

    /// Schema instance shared by oneshot and transactional handles.
    fn schema() -> Self::Schema;

    /// Build an oneshot repository: each operation runs in its own
    /// auto-committed transaction.
    fn oneshot() -> Self;

    /// Build a repository whose operations splice into an externally-driven
    /// transaction. Lifecycle (commit/rollback) is owned by the caller —
    /// typically via [`crate::transaction::Transaction::run`].
    fn with_transaction(tx: TransactionId) -> Self;

    /// Externally-driven transaction id, when the repository was constructed
    /// with [`Self::with_transaction`]. Oneshot repositories return `None`.
    fn tx(&self) -> Option<TransactionId>;

    /// Build a [`WasmDbmsDatabase`] handle bound to this repository's
    /// transaction lifetime.
    fn db<'a>(
        &self,
        ctx: &'a DbmsContext<IcMemoryProvider, IcAccessControlList>,
    ) -> WasmDbmsDatabase<'a, IcMemoryProvider, IcAccessControlList> {
        match self.tx() {
            Some(id) => WasmDbmsDatabase::from_transaction(ctx, Self::schema(), id),
            None => WasmDbmsDatabase::oneshot(ctx, Self::schema()),
        }
    }
}
