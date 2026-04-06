//! Status repository

use did::common::Visibility;
use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::*;

use crate::domain::snowflake::Snowflake;
use crate::error::CanisterResult;
use crate::schema::{
    FeedEntry, FeedEntryInsertRequest, FeedSource, Schema, Status, StatusInsertRequest,
    StatusRecord,
};

/// Interface for the [`Status`] repository.
pub struct StatusRepository;

impl StatusRepository {
    /// Create a new [`Status`] with the given content, timestamp and visibility.
    ///
    /// A new [`Snowflake`] ID is generated for the status, and the current timestamp is used as the creation time.
    ///
    /// Both the `statuses` row and the corresponding `feed` entry are
    /// inserted inside a single transaction.
    ///
    /// In case of success the [`Snowflake`] ID of the newly created status is returned.
    pub fn create(
        content: String,
        visibility: Visibility,
        created_at: u64,
    ) -> CanisterResult<Snowflake> {
        let snowflake_id = Snowflake::new();

        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);

            db.insert::<Status>(StatusInsertRequest {
                id: snowflake_id.into(),
                content: content.into(),
                visibility: visibility.into(),
                created_at: created_at.into(),
            })?;

            db.insert::<FeedEntry>(FeedEntryInsertRequest {
                id: snowflake_id.into(),
                source: FeedSource::Outbox,
                created_at: created_at.into(),
            })?;

            db.commit()
        })?;

        Ok(snowflake_id)
    }

    /// Get a paginated list of [`Status`]es with the given visibility, ordered from the most recent to the oldest.
    ///
    /// This function is used to implement the `get_statuses` API, and the returned statuses are filtered based on the relationship of the caller with the user (owner, follower, or other).
    pub fn get_paginated_by_visibility(
        visibility: &[Visibility],
        offset: usize,
        limit: usize,
    ) -> CanisterResult<Vec<Status>> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            let mut query = Query::builder()
                .all()
                .limit(limit)
                .offset(offset)
                .order_by_desc("created_at");
            for v in visibility {
                let visibility = crate::schema::Visibility::from(*v);
                query = query.or_where(Filter::eq("visibility", visibility.into()));
            }

            let results = db.select::<Status>(query.build())?;
            Ok(results.into_iter().map(Self::record_to_status).collect())
        })
    }

    fn record_to_status(record: StatusRecord) -> Status {
        Status {
            id: record.id.expect("must have field"),
            content: record.content.expect("must have field"),
            visibility: record.visibility.expect("must have field"),
            created_at: record.created_at.expect("must have field"),
        }
    }
}

#[cfg(test)]
mod tests {

    use did::common::Visibility;

    use super::StatusRepository;
    use crate::test_utils::{insert_status, setup};

    #[test]
    fn test_should_return_empty_when_no_statuses() {
        setup();

        let result = StatusRepository::get_paginated_by_visibility(&[Visibility::Public], 0, 10)
            .expect("should query");

        assert!(result.is_empty());
    }

    #[test]
    fn test_should_return_statuses_ordered_by_created_at_desc() {
        setup();
        insert_status(1, "Old", Visibility::Public, 1000);
        insert_status(2, "Mid", Visibility::Public, 2000);
        insert_status(3, "New", Visibility::Public, 3000);

        let result = StatusRepository::get_paginated_by_visibility(&[Visibility::Public], 0, 10)
            .expect("should query");

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].created_at.0, 3000);
        assert_eq!(result[1].created_at.0, 2000);
        assert_eq!(result[2].created_at.0, 1000);
    }

    #[test]
    fn test_should_paginate_with_offset_and_limit() {
        setup();
        for i in 1..=5 {
            insert_status(i, &format!("Status {i}"), Visibility::Public, i * 1000);
        }

        // skip 1, take 2 → newest-first: 5,4,3,2,1 → skip 1 → 4,3
        let result = StatusRepository::get_paginated_by_visibility(&[Visibility::Public], 1, 2)
            .expect("should query");

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].created_at.0, 4000);
        assert_eq!(result[1].created_at.0, 3000);
    }

    #[test]
    fn test_should_filter_by_single_visibility() {
        setup();
        insert_status(1, "Public", Visibility::Public, 1000);
        insert_status(2, "Unlisted", Visibility::Unlisted, 2000);
        insert_status(3, "FollowersOnly", Visibility::FollowersOnly, 3000);
        insert_status(4, "Direct", Visibility::Direct, 4000);

        let result = StatusRepository::get_paginated_by_visibility(&[Visibility::Public], 0, 10)
            .expect("should query");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content.0, "Public");
    }

    #[test]
    fn test_should_filter_by_multiple_visibilities() {
        setup();
        insert_status(1, "Public", Visibility::Public, 1000);
        insert_status(2, "Unlisted", Visibility::Unlisted, 2000);
        insert_status(3, "FollowersOnly", Visibility::FollowersOnly, 3000);
        insert_status(4, "Direct", Visibility::Direct, 4000);

        let result = StatusRepository::get_paginated_by_visibility(
            &[
                Visibility::Public,
                Visibility::Unlisted,
                Visibility::FollowersOnly,
            ],
            0,
            10,
        )
        .expect("should query");

        assert_eq!(result.len(), 3);
        // ordered newest-first
        assert_eq!(result[0].content.0, "FollowersOnly");
        assert_eq!(result[1].content.0, "Unlisted");
        assert_eq!(result[2].content.0, "Public");
    }

    #[test]
    fn test_should_filter_all_visibilities() {
        setup();
        insert_status(1, "Public", Visibility::Public, 1000);
        insert_status(2, "Unlisted", Visibility::Unlisted, 2000);
        insert_status(3, "FollowersOnly", Visibility::FollowersOnly, 3000);
        insert_status(4, "Direct", Visibility::Direct, 4000);

        let result = StatusRepository::get_paginated_by_visibility(
            &[
                Visibility::Public,
                Visibility::Unlisted,
                Visibility::FollowersOnly,
                Visibility::Direct,
            ],
            0,
            10,
        )
        .expect("should query");

        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_should_return_empty_page_beyond_data() {
        setup();
        insert_status(1, "Only one", Visibility::Public, 1000);

        let result = StatusRepository::get_paginated_by_visibility(&[Visibility::Public], 100, 10)
            .expect("should query");

        assert!(result.is_empty());
    }

    #[test]
    fn test_create_should_insert_status_and_feed_entry() {
        setup();

        let snowflake =
            StatusRepository::create("Hello world".to_string(), Visibility::Public, 42_000)
                .expect("should create");

        // verify the status was inserted
        let statuses = StatusRepository::get_paginated_by_visibility(&[Visibility::Public], 0, 10)
            .expect("should query");

        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].id, snowflake.into());
        assert_eq!(statuses[0].content.0, "Hello world");
        assert_eq!(statuses[0].created_at.0, 42_000);
    }
}
