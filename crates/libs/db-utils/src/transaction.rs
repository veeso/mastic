//! Transaction utilities.

use ic_dbms_canister::prelude::{DBMS_CONTEXT, IcAccessControlList, IcMemoryProvider};
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms::prelude::DatabaseSchema;
use wasm_dbms_api::error::DbmsError;
use wasm_dbms_api::prelude::{Database, TransactionId};

/// Build a transaction caller ID from a timestamp.
///
/// Converts the given `u64` timestamp into its 8-byte big-endian
/// representation, suitable for use as the caller argument to
/// `DbmsContext::begin_transaction`.
pub fn transaction_caller(timestamp: u64) -> Vec<u8> {
    timestamp.to_be_bytes().to_vec()
}

/// Transaction lifecycle primitive operating on the canister-local
/// `DBMS_CONTEXT`. Repositories never commit themselves — callers drive
/// lifecycle through `Transaction::begin`/`commit`/`rollback` or the
/// `Transaction::run` sugar.
pub struct Transaction;

impl Transaction {
    /// Open a new transaction owned by the current `ic_utils::now()`
    /// timestamp.
    pub fn begin() -> TransactionId {
        DBMS_CONTEXT.with(|ctx| ctx.begin_transaction(transaction_caller(ic_utils::now())))
    }

    /// Commit the transaction `tx` against `schema`.
    pub fn commit<S>(schema: S, tx: TransactionId) -> Result<(), DbmsError>
    where
        S: DatabaseSchema<IcMemoryProvider, IcAccessControlList>,
    {
        DBMS_CONTEXT.with(|ctx| {
            let mut db = WasmDbmsDatabase::from_transaction(ctx, schema, tx);
            db.commit()
        })
    }

    /// Roll back the transaction `tx` against `schema`.
    pub fn rollback<S>(schema: S, tx: TransactionId) -> Result<(), DbmsError>
    where
        S: DatabaseSchema<IcMemoryProvider, IcAccessControlList>,
    {
        DBMS_CONTEXT.with(|ctx| {
            let mut db = WasmDbmsDatabase::from_transaction(ctx, schema, tx);
            db.rollback()
        })
    }

    /// Run `f` inside a transaction, committing on `Ok` and rolling back on
    /// `Err`. The closure receives the live `TransactionId`, which it must
    /// pass to `XRepository::with_transaction(tx)` for every repo call inside.
    pub fn run<S, F, T, E>(schema: S, f: F) -> Result<T, E>
    where
        S: DatabaseSchema<IcMemoryProvider, IcAccessControlList> + Copy,
        F: FnOnce(TransactionId) -> Result<T, E>,
        E: From<DbmsError>,
    {
        let tx = Self::begin();
        match f(tx) {
            Ok(value) => {
                Self::commit(schema, tx)?;
                Ok(value)
            }
            Err(err) => {
                // best-effort rollback; the caller's error wins
                let _ = Self::rollback(schema, tx);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::cell::Cell;

    use wasm_dbms_api::prelude::{
        DatabaseSchema as DatabaseSchemaDerive, Filter, Query, Table, Uint64, Value,
    };

    use super::*;

    /// Minimal single-table fixture used to exercise commit/rollback against
    /// the canister-local `DBMS_CONTEXT`.
    #[derive(Debug, Table, Clone, PartialEq, Eq)]
    #[table = "items"]
    pub struct Item {
        #[primary_key]
        pub id: Uint64,
    }

    #[derive(Clone, Copy, DatabaseSchemaDerive)]
    #[tables(Item = "items")]
    pub struct TestSchema;

    thread_local! {
        static REGISTERED: Cell<bool> = const { Cell::new(false) };
    }

    fn register_test_schema() {
        REGISTERED.with(|flag| {
            if !flag.get() {
                DBMS_CONTEXT.with(|ctx| {
                    TestSchema::register_tables(ctx).expect("register tables");
                });
                flag.set(true);
            }
        });
    }

    #[test]
    fn test_transaction_caller_returns_8_bytes() {
        let caller = transaction_caller(1_000_000);
        assert_eq!(caller.len(), 8);
    }

    #[test]
    fn test_transaction_caller_is_big_endian() {
        let caller = transaction_caller(0x0102_0304_0506_0708);
        assert_eq!(caller, vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
    }

    #[test]
    fn test_transaction_begin_returns_distinct_ids() {
        let a = Transaction::begin();
        let b = Transaction::begin();
        assert_ne!(a, b);
    }

    #[test]
    fn test_transaction_commit_persists_inserts() {
        register_test_schema();

        let tx = Transaction::begin();
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::from_transaction(ctx, TestSchema, tx);
            db.insert::<Item>(ItemInsertRequest { id: 1u64.into() })
                .expect("insert");
        });
        Transaction::commit(TestSchema, tx).expect("commit");

        let rows = DBMS_CONTEXT.with(|ctx| {
            WasmDbmsDatabase::oneshot(ctx, TestSchema)
                .select::<Item>(
                    Query::builder()
                        .and_where(Filter::eq("id", Value::from(1u64)))
                        .build(),
                )
                .expect("select")
        });
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn test_transaction_run_commits_on_ok() {
        register_test_schema();

        Transaction::run::<_, _, _, DbmsError>(TestSchema, |tx| {
            DBMS_CONTEXT.with(|ctx| {
                let db = WasmDbmsDatabase::from_transaction(ctx, TestSchema, tx);
                db.insert::<Item>(ItemInsertRequest { id: 7u64.into() })
            })
        })
        .expect("run");

        let rows = DBMS_CONTEXT.with(|ctx| {
            WasmDbmsDatabase::oneshot(ctx, TestSchema)
                .select::<Item>(
                    Query::builder()
                        .and_where(Filter::eq("id", Value::from(7u64)))
                        .build(),
                )
                .expect("select")
        });
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn test_transaction_run_rolls_back_on_err() {
        register_test_schema();

        let result: Result<(), DbmsError> = Transaction::run(TestSchema, |tx| {
            DBMS_CONTEXT.with(|ctx| {
                let db = WasmDbmsDatabase::from_transaction(ctx, TestSchema, tx);
                db.insert::<Item>(ItemInsertRequest { id: 9u64.into() })?;
                Err(DbmsError::Transaction(
                    wasm_dbms_api::prelude::TransactionError::NoActiveTransaction,
                ))
            })
        });
        assert!(result.is_err());

        let rows = DBMS_CONTEXT.with(|ctx| {
            WasmDbmsDatabase::oneshot(ctx, TestSchema)
                .select::<Item>(
                    Query::builder()
                        .and_where(Filter::eq("id", Value::from(9u64)))
                        .build(),
                )
                .expect("select")
        });
        assert!(rows.is_empty(), "rollback should drop the staged insert");
    }
}
