//! Migration utilities for the database.

use std::thread::LocalKey;

use ic_dbms_canister::prelude::{
    DatabaseSchema, DbmsContext, IcAccessControlList, IcMemoryProvider,
};
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::{Database, MigrationPolicy};

/// Run a post-upgrade migration to update the database schema after a canister upgrade.
///
/// This function doesn't fail with an error, but can trap if it fails to check whether there is a migration to run.
///
/// Since Internet Computer wouldn't allow the user to run the migration inside a single call, we need to run the migration in
/// a separate task, so this means we don't have control over whether the migration failed or not, but we can at least log the error if it fails.
pub fn run_post_upgrade_migration<S>(
    ctx: &'static LocalKey<DbmsContext<IcMemoryProvider, IcAccessControlList>>,
    schema: S,
) where
    S: DatabaseSchema<IcMemoryProvider, IcAccessControlList> + Clone + 'static,
{
    let has_drift = match ctx.with(|c| {
        let db = WasmDbmsDatabase::oneshot(c, schema.clone());
        db.has_drift()
    }) {
        Ok(v) => v,
        Err(err) => {
            ic_utils::trap!("Failed to check database drift: {err}");
        }
    };

    if !has_drift {
        ic_utils::log!("No database drift detected, skipping migration");
        return;
    }

    ic_utils::log!("Database drift detected, running migration");
    ic_cdk::futures::spawn_migratory(run_migration(ctx, schema));
}

/// Run a migration to update the database schema after a canister upgrade.
///
/// This function is intended to be used in a post-upgrade hook,
/// and it will run the migration in a separate task, so it won't block the upgrade process.
/// However, it can trap if it fails to check whether there is a migration to run.
async fn run_migration<S>(
    ctx: &'static LocalKey<DbmsContext<IcMemoryProvider, IcAccessControlList>>,
    schema: S,
) where
    S: DatabaseSchema<IcMemoryProvider, IcAccessControlList> + Clone + 'static,
{
    ctx.with(|c| {
        let mut db = WasmDbmsDatabase::oneshot(c, schema);

        let pending_migrations = match db.pending_migrations() {
            Ok(migrations) => migrations,
            Err(err) => {
                ic_utils::log!("Failed to check pending migrations: {err}");
                return;
            }
        };
        for migration in pending_migrations {
            ic_utils::log!("Pending migration: {migration:?}");
        }

        if let Err(err) = db.migrate(MigrationPolicy {
            allow_destructive: true,
        }) {
            ic_utils::log!("Failed to run database migrations: {err}");
        } else {
            ic_utils::log!("Database migrations completed successfully");
        }
    });
}
