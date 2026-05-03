//! Status repository

use did::common::Visibility;
use ic_dbms_canister::prelude::{DBMS_CONTEXT, IcAccessControlList, IcMemoryProvider};
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms::prelude::DbmsContext;
use wasm_dbms_api::prelude::*;

use crate::error::CanisterResult;
use crate::schema::{Schema, Status, StatusInsertRequest, StatusRecord, StatusUpdateRequest};

/// Interface for the [`Status`] repository.
pub struct StatusRepository {
    tx: Option<TransactionId>,
}

impl StatusRepository {
    pub const fn oneshot() -> Self {
        Self { tx: None }
    }

    pub const fn with_transaction(tx: TransactionId) -> Self {
        Self { tx: Some(tx) }
    }

    fn db<'a>(
        &self,
        ctx: &'a DbmsContext<IcMemoryProvider, IcAccessControlList>,
    ) -> WasmDbmsDatabase<'a, IcMemoryProvider, IcAccessControlList> {
        match self.tx {
            Some(id) => WasmDbmsDatabase::from_transaction(ctx, Schema, id),
            None => WasmDbmsDatabase::oneshot(ctx, Schema),
        }
    }

    /// Insert a row into the `statuses` table with default flags
    /// (`like_count = 0`, `boost_count = 0`, no reply, no spoiler, not
    /// sensitive, never edited).
    ///
    /// The caller mints `snowflake_id` and is responsible for any cross-table
    /// orchestration (e.g. the matching `feed` entry) via `Transaction::run`.
    pub fn insert(
        &self,
        snowflake_id: u64,
        content: String,
        visibility: Visibility,
        created_at: u64,
    ) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx).insert::<Status>(StatusInsertRequest {
                id: snowflake_id.into(),
                content: content.into(),
                visibility: visibility.into(),
                like_count: 0u64.into(),
                boost_count: 0u64.into(),
                in_reply_to_uri: Nullable::Null,
                spoiler_text: Nullable::Null,
                sensitive: false.into(),
                edited_at: Nullable::Null,
                created_at: created_at.into(),
            })?;
            Ok(())
        })
    }

    /// Insert a wrapper-style `statuses` row used for boost wrappers, where
    /// `spoiler_text` and `sensitive` may be set from the boosted status.
    ///
    /// The caller mints `snowflake_id` and is responsible for any cross-table
    /// orchestration (e.g. matching `boosts` and `feed` rows) via
    /// `Transaction::run`.
    //
    // Wired in by the boost refactor; covered by the in-module test suite
    // until then.
    #[allow(dead_code)]
    pub fn insert_wrapper(
        &self,
        snowflake_id: u64,
        content: &str,
        visibility: Visibility,
        spoiler_text: Option<&str>,
        sensitive: bool,
        created_at: u64,
    ) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx).insert::<Status>(StatusInsertRequest {
                id: snowflake_id.into(),
                content: content.into(),
                visibility: visibility.into(),
                like_count: 0u64.into(),
                boost_count: 0u64.into(),
                in_reply_to_uri: Nullable::Null,
                spoiler_text: spoiler_text.map_or(Nullable::Null, |s| Nullable::Value(s.into())),
                sensitive: sensitive.into(),
                edited_at: Nullable::Null,
                created_at: created_at.into(),
            })?;
            Ok(())
        })
    }

    /// Delete the [`Status`] row whose primary key matches `snowflake_id`.
    ///
    /// Uses [`DeleteBehavior::Restrict`] — callers must drop dependent rows
    /// (`boosts`, `feed`) in the right order so referential integrity is
    /// preserved.
    //
    // Wired in by the boost refactor (undo_boost flow); covered by the
    // in-module test suite until then.
    #[allow(dead_code)]
    pub fn delete_by_id(&self, snowflake_id: u64) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx).delete::<Status>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(Status::primary_key(), Value::from(snowflake_id))),
            )?;
            Ok(())
        })
    }

    /// Get a paginated list of [`Status`]es with the given visibility, ordered from the most recent to the oldest.
    ///
    /// This function is used to implement the `get_statuses` API, and the returned statuses are filtered based on the relationship of the caller with the user (owner, follower, or other).
    pub fn get_paginated_by_visibility(
        &self,
        visibility: &[Visibility],
        offset: usize,
        limit: usize,
    ) -> CanisterResult<Vec<Status>> {
        DBMS_CONTEXT.with(|ctx| {
            let db = self.db(ctx);

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

    /// Look up a single [`Status`] by its [`Snowflake`] id, returning [`None`]
    /// if no row matches.
    pub fn find_by_id(&self, id: u64) -> CanisterResult<Option<Status>> {
        DBMS_CONTEXT.with(|ctx| {
            let rows = self.db(ctx).select::<Status>(
                Query::builder()
                    .all()
                    .and_where(Filter::eq(Status::primary_key(), Value::from(id)))
                    .limit(1)
                    .build(),
            )?;
            Ok(rows.into_iter().next().map(Self::record_to_status))
        })
    }

    /// Increment the cached `like_count` of the [`Status`] with the given id.
    ///
    /// Returns `Ok(true)` when the row exists and was updated, `Ok(false)`
    /// when no row matched (silently ignored: a missing target means we are
    /// not the author of that status, so the counter does not concern us).
    pub fn increment_like_count(&self, id: u64) -> CanisterResult<bool> {
        self.adjust_like_count(id, 1, true)
    }

    /// Saturating-decrement the cached `like_count` of the [`Status`] with the
    /// given id. Decrementing from `0` is a no-op rather than an underflow.
    ///
    /// Returns `Ok(true)` when the row exists, `Ok(false)` when no row matched.
    pub fn decrement_like_count(&self, id: u64) -> CanisterResult<bool> {
        self.adjust_like_count(id, 1, false)
    }

    fn adjust_like_count(&self, id: u64, delta: u64, increment: bool) -> CanisterResult<bool> {
        let Some(status) = self.find_by_id(id)? else {
            return Ok(false);
        };
        let current = status.like_count.0;
        let next = if increment {
            current.saturating_add(delta)
        } else {
            current.saturating_sub(delta)
        };
        if next == current {
            return Ok(true);
        }

        let patch = StatusUpdateRequest {
            like_count: Some(next.into()),
            where_clause: Some(Filter::eq(Status::primary_key(), Value::from(id))),
            ..Default::default()
        };

        DBMS_CONTEXT.with(|ctx| self.db(ctx).update::<Status>(patch))?;
        Ok(true)
    }

    /// Increment the cached `boost_count` of the [`Status`] with the given id.
    ///
    /// Returns `Ok(true)` when the row exists and was updated, `Ok(false)`
    /// when no row matched (silently ignored: a missing target means we are
    /// not the author of that status, so the counter does not concern us).
    pub fn increment_boost_count(&self, id: u64) -> CanisterResult<bool> {
        self.adjust_boost_count(id, 1, true)
    }

    /// Saturating-decrement the cached `boost_count` of the [`Status`] with the
    /// given id. Decrementing from `0` is a no-op rather than an underflow.
    ///
    /// Returns `Ok(true)` when the row exists, `Ok(false)` when no row matched.
    pub fn decrement_boost_count(&self, id: u64) -> CanisterResult<bool> {
        self.adjust_boost_count(id, 1, false)
    }

    fn adjust_boost_count(&self, id: u64, delta: u64, increment: bool) -> CanisterResult<bool> {
        let Some(status) = self.find_by_id(id)? else {
            return Ok(false);
        };
        let current = status.boost_count.0;
        let next = if increment {
            current.saturating_add(delta)
        } else {
            current.saturating_sub(delta)
        };
        if next == current {
            return Ok(true);
        }
        let patch = StatusUpdateRequest {
            boost_count: Some(next.into()),
            where_clause: Some(Filter::eq(Status::primary_key(), Value::from(id))),
            ..Default::default()
        };
        DBMS_CONTEXT.with(|ctx| self.db(ctx).update::<Status>(patch))?;
        Ok(true)
    }

    fn record_to_status(record: StatusRecord) -> Status {
        Status {
            id: record.id.expect("must have field"),
            content: record.content.expect("must have field"),
            visibility: record.visibility.expect("must have field"),
            like_count: record.like_count.expect("must have field"),
            boost_count: record.boost_count.expect("must have field"),
            in_reply_to_uri: record.in_reply_to_uri.expect("must have field"),
            spoiler_text: record.spoiler_text.expect("must have field"),
            sensitive: record.sensitive.expect("must have field"),
            edited_at: record.edited_at.expect("must have field"),
            created_at: record.created_at.expect("must have field"),
        }
    }
}

#[cfg(test)]
mod tests {

    use did::common::Visibility;

    use super::*;
    use crate::test_utils::{insert_status, setup};

    #[test]
    fn test_should_return_empty_when_no_statuses() {
        setup();

        let result = StatusRepository::oneshot()
            .get_paginated_by_visibility(&[Visibility::Public], 0, 10)
            .expect("should query");

        assert!(result.is_empty());
    }

    #[test]
    fn test_should_return_statuses_ordered_by_created_at_desc() {
        setup();
        insert_status(1, "Old", Visibility::Public, 1000);
        insert_status(2, "Mid", Visibility::Public, 2000);
        insert_status(3, "New", Visibility::Public, 3000);

        let result = StatusRepository::oneshot()
            .get_paginated_by_visibility(&[Visibility::Public], 0, 10)
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
        let result = StatusRepository::oneshot()
            .get_paginated_by_visibility(&[Visibility::Public], 1, 2)
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

        let result = StatusRepository::oneshot()
            .get_paginated_by_visibility(&[Visibility::Public], 0, 10)
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

        let result = StatusRepository::oneshot()
            .get_paginated_by_visibility(
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

        let result = StatusRepository::oneshot()
            .get_paginated_by_visibility(
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

        let result = StatusRepository::oneshot()
            .get_paginated_by_visibility(&[Visibility::Public], 100, 10)
            .expect("should query");

        assert!(result.is_empty());
    }

    #[test]
    fn test_delete_by_id_should_remove_status_row() {
        setup();
        insert_status(11, "to delete", Visibility::Public, 1_000);

        StatusRepository::oneshot()
            .delete_by_id(11)
            .expect("should delete");

        assert!(
            StatusRepository::oneshot()
                .find_by_id(11)
                .expect("query")
                .is_none()
        );
    }

    #[test]
    fn test_delete_by_id_should_be_noop_when_missing() {
        setup();

        StatusRepository::oneshot()
            .delete_by_id(999)
            .expect("delete missing must succeed");
    }

    #[test]
    fn test_insert_should_persist_default_flags() {
        setup();

        StatusRepository::oneshot()
            .insert(123, "hello".to_string(), Visibility::Public, 7_000)
            .expect("should insert");

        let row = StatusRepository::oneshot()
            .find_by_id(123)
            .expect("query")
            .expect("row exists");
        assert_eq!(row.content.0, "hello");
        assert_eq!(row.created_at.0, 7_000);
        assert_eq!(row.like_count.0, 0);
        assert_eq!(row.boost_count.0, 0);
        assert!(!row.sensitive.0);
        assert!(row.spoiler_text.clone().into_opt().is_none());
        assert!(row.in_reply_to_uri.clone().into_opt().is_none());
        assert!(row.edited_at.into_opt().is_none());
    }

    #[test]
    fn test_insert_wrapper_should_persist_spoiler_and_sensitive() {
        setup();

        StatusRepository::oneshot()
            .insert_wrapper(321, "wrapper", Visibility::Public, Some("cw"), true, 8_000)
            .expect("should insert wrapper");

        let row = StatusRepository::oneshot()
            .find_by_id(321)
            .expect("query")
            .expect("row exists");
        assert_eq!(row.content.0, "wrapper");
        assert_eq!(
            row.spoiler_text.clone().into_opt().expect("spoiler").0,
            "cw"
        );
        assert!(row.sensitive.0);
    }

    #[test]
    fn test_should_increment_boost_count() {
        setup();
        insert_status(7, "Hi", Visibility::Public, 1_000);

        assert!(
            StatusRepository::oneshot()
                .increment_boost_count(7)
                .expect("should adjust")
        );

        let row = StatusRepository::oneshot().find_by_id(7).unwrap().unwrap();
        assert_eq!(row.boost_count.0, 1);
    }

    #[test]
    fn test_increment_boost_count_returns_false_when_missing() {
        setup();
        assert!(
            !StatusRepository::oneshot()
                .increment_boost_count(99)
                .expect("should adjust")
        );
    }

    #[test]
    fn test_should_decrement_boost_count_saturating() {
        setup();
        insert_status(7, "Hi", Visibility::Public, 1_000);
        StatusRepository::oneshot()
            .increment_boost_count(7)
            .expect("inc");
        StatusRepository::oneshot()
            .decrement_boost_count(7)
            .expect("dec");
        assert_eq!(
            StatusRepository::oneshot()
                .find_by_id(7)
                .unwrap()
                .unwrap()
                .boost_count
                .0,
            0
        );

        StatusRepository::oneshot()
            .decrement_boost_count(7)
            .expect("dec at zero");
        assert_eq!(
            StatusRepository::oneshot()
                .find_by_id(7)
                .unwrap()
                .unwrap()
                .boost_count
                .0,
            0
        );
    }
}
