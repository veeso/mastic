//! Status domain

use db_utils::repository::Repository;
mod delete_status;
mod get_local_status;
mod get_statuses;
mod publish;

pub use delete_status::delete_status;
use did::common::Visibility;
use wasm_dbms_api::prelude::TransactionId;

pub use self::get_local_status::get_local_status;
pub use self::get_statuses::get_statuses;
pub use self::publish::publish_status;
use crate::error::CanisterResult;
use crate::repository::feed::FeedRepository;
use crate::repository::status::StatusRepository;

/// Maximum allowed length for the status content.
pub const MAX_STATUS_LENGTH: usize = 500;

/// Insert a status row plus its outbox feed entry inside the given
/// transaction. Caller must drive the transaction lifecycle, typically
/// via [`db_utils::transaction::Transaction::run`].
pub(crate) fn create_status_with_feed(
    tx: TransactionId,
    snowflake_id: u64,
    content: String,
    visibility: Visibility,
    created_at: u64,
) -> CanisterResult<()> {
    StatusRepository::with_transaction(tx).insert(snowflake_id, content, visibility, created_at)?;
    FeedRepository::with_transaction(tx).insert_outbox(snowflake_id, created_at)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use db_utils::transaction::Transaction;
    use did::common::Visibility;
    use ic_dbms_canister::prelude::DBMS_CONTEXT;
    use wasm_dbms::WasmDbmsDatabase;
    use wasm_dbms_api::prelude::{Database, Query};

    use super::*;
    use crate::domain::snowflake::Snowflake;
    use crate::error::CanisterError;
    use crate::schema::{FeedEntry, Schema};
    use crate::test_utils::setup;

    fn count_feed_entries() -> usize {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.select::<FeedEntry>(Query::builder().all().build())
                .expect("select feed")
                .len()
        })
    }

    #[test]
    fn test_create_status_with_feed_should_insert_status_and_feed_entry() {
        setup();

        let snowflake: u64 = Snowflake::new().into();
        Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
            create_status_with_feed(
                tx,
                snowflake,
                "Hello world".to_string(),
                Visibility::Public,
                42_000,
            )
        })
        .expect("should create");

        // verify the status was inserted
        let statuses = StatusRepository::oneshot()
            .get_paginated_by_visibility(&[Visibility::Public], 0, 10)
            .expect("should query");

        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].id.0, snowflake);
        assert_eq!(statuses[0].content.0, "Hello world");
        assert_eq!(statuses[0].created_at.0, 42_000);

        // verify the feed entry was inserted alongside
        assert_eq!(count_feed_entries(), 1);
    }

    #[test]
    fn test_create_status_with_feed_should_rollback_on_error() {
        setup();

        // First create a status with a known ID via the helper
        let snowflake: u64 = Snowflake::new().into();
        Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
            create_status_with_feed(
                tx,
                snowflake,
                "first".to_string(),
                Visibility::Public,
                1_000,
            )
        })
        .expect("first create");

        let initial_count = StatusRepository::oneshot()
            .get_paginated_by_visibility(&[Visibility::Public], 0, 100)
            .expect("query")
            .len();
        assert_eq!(initial_count, 1);

        // Run a transaction that errors out — neither the status nor the
        // feed entry must be persisted.
        let rollback_id: u64 = Snowflake::new().into();
        let result: Result<(), CanisterError> = Transaction::run(Schema, |tx| {
            create_status_with_feed(
                tx,
                rollback_id,
                "rollback".to_string(),
                Visibility::Public,
                2_000,
            )?;
            Err(CanisterError::Internal("boom".to_string()))
        });
        assert!(result.is_err());

        let after_count = StatusRepository::oneshot()
            .get_paginated_by_visibility(&[Visibility::Public], 0, 100)
            .expect("query")
            .len();
        assert_eq!(after_count, 1, "errored tx must not persist its insert");

        // The rollback status must not be present
        assert!(
            StatusRepository::oneshot()
                .find_by_id(rollback_id)
                .expect("query")
                .is_none()
        );

        // The feed entry from the rolled-back transaction must not be persisted.
        assert_eq!(
            count_feed_entries(),
            1,
            "errored tx must not persist its feed entry"
        );
    }
}
