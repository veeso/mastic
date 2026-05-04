//! Repository for the `inbox` table.

use db_utils::repository::Repository;
use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms_api::prelude::{
    Database, DeleteBehavior, Filter, JsonFilter, Nullable, Query, TableSchema, TransactionId,
    Value,
};

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{ActivityType, InboxActivity, InboxActivityInsertRequest, Schema};

/// Repository over the `inbox` table.
pub struct InboxActivityRepository {
    tx: Option<TransactionId>,
}

impl InboxActivityRepository {
    /// Insert an inbox activity row.
    ///
    /// The caller mints `snowflake_id` and is responsible for any
    /// cross-table orchestration (e.g. the matching `feed` entry) via
    /// [`db_utils::transaction::Transaction::run`].
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "will be used by receive_activity refactor")
    )]
    #[expect(
        clippy::too_many_arguments,
        reason = "schema row collapses to a single insert"
    )]
    pub fn insert(
        &self,
        snowflake_id: u64,
        activity_type: ActivityType,
        actor_uri: &str,
        object_data: serde_json::Value,
        is_boost: bool,
        original_status_uri: Option<&str>,
        created_at: u64,
    ) -> CanisterResult<()> {
        let original_status_uri = match original_status_uri {
            Some(s) => Nullable::Value(s.into()),
            None => Nullable::Null,
        };
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx)
                .insert::<InboxActivity>(InboxActivityInsertRequest {
                    id: snowflake_id.into(),
                    activity_type,
                    actor_uri: actor_uri.into(),
                    object_data: object_data.into(),
                    is_boost: is_boost.into(),
                    original_status_uri,
                    created_at: created_at.into(),
                })?;
            Ok(())
        })
    }

    /// Return the snowflake ids of every inbox boost row whose
    /// `original_status_uri` matches `uri`.
    pub fn find_boost_ids_by_original_uri(&self, uri: &str) -> CanisterResult<Vec<u64>> {
        self.find_ids(
            Query::builder()
                .all()
                .and_where(Filter::eq("is_boost", Value::from(true)))
                .and_where(Filter::eq("original_status_uri", Value::from(uri)))
                .build(),
        )
    }

    /// Return the snowflake ids of every non-boost inbox row whose cached
    /// `object_data` activity has `object.id == object_id`. When `actor`
    /// is supplied, the activity's `actor` URI must also match.
    pub fn find_create_ids_with_object_id(
        &self,
        object_id: &str,
        actor: Option<&str>,
    ) -> CanisterResult<Vec<u64>> {
        let mut query = Query::builder()
            .all()
            .and_where(Filter::eq("is_boost", Value::from(false)))
            .and_where(Filter::json(
                "object_data",
                JsonFilter::extract_eq("object.id", Value::from(object_id)),
            ));
        if let Some(actor) = actor {
            query = query.and_where(Filter::json(
                "object_data",
                JsonFilter::extract_eq("actor", Value::from(actor)),
            ));
        }
        self.find_ids(query.build())
    }

    /// Delete the [`InboxActivity`] row whose primary key matches
    /// `snowflake_id`. Restrict-mode delete: callers must drop the matching
    /// `feed` row separately.
    pub fn delete_by_id(&self, snowflake_id: u64) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx).delete::<InboxActivity>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    InboxActivity::primary_key(),
                    Value::from(snowflake_id),
                )),
            )?;
            Ok(())
        })
    }

    /// Run a typed select for [`InboxActivity`] and project the rows down
    /// to their primary keys.
    fn find_ids(&self, query: Query) -> CanisterResult<Vec<u64>> {
        DBMS_CONTEXT
            .with(|ctx| self.db(ctx).select::<InboxActivity>(query))
            .map(|rows| rows.into_iter().filter_map(|r| r.id.map(|v| v.0)).collect())
            .map_err(CanisterError::from)
    }
}

impl Repository for InboxActivityRepository {
    type Schema = Schema;

    fn schema() -> Self::Schema {
        Schema
    }

    fn oneshot() -> Self {
        Self { tx: None }
    }

    fn with_transaction(tx: TransactionId) -> Self {
        Self { tx: Some(tx) }
    }

    fn tx(&self) -> Option<TransactionId> {
        self.tx
    }
}

#[cfg(test)]
mod tests {
    use db_utils::transaction::Transaction;

    use super::*;
    use crate::error::CanisterError;
    use crate::test_utils::setup;

    fn announce_type() -> ActivityType {
        ActivityType::from(activitypub::ActivityType::Announce)
    }

    fn create_type() -> ActivityType {
        ActivityType::from(activitypub::ActivityType::Create)
    }

    #[test]
    fn test_should_find_boost_ids_by_original_uri() {
        setup();
        let uri = "https://remote.example/users/x/statuses/9";
        let repo = InboxActivityRepository::oneshot();
        repo.insert(
            1,
            announce_type(),
            "https://remote.example/users/a",
            serde_json::json!({"type":"Announce"}),
            true,
            Some(uri),
            1_000,
        )
        .expect("insert");
        repo.insert(
            2,
            announce_type(),
            "https://remote.example/users/b",
            serde_json::json!({"type":"Announce"}),
            true,
            Some("https://other.example/users/x/statuses/100"),
            1_000,
        )
        .expect("insert");

        let ids = repo.find_boost_ids_by_original_uri(uri).expect("query");
        assert_eq!(ids, vec![1]);
    }

    #[test]
    fn test_should_find_create_ids_by_full_uri_object_id() {
        setup();
        let uri = "https://remote.example/users/x/statuses/9";
        let repo = InboxActivityRepository::oneshot();
        repo.insert(
            1,
            create_type(),
            "https://remote.example/users/a",
            serde_json::json!({
                "type": "Create",
                "actor": "https://remote.example/users/a",
                "object": { "type": "Note", "id": uri, "content": "hi" },
            }),
            false,
            None,
            1_000,
        )
        .expect("insert");

        let ids = repo
            .find_create_ids_with_object_id(uri, None)
            .expect("query");
        assert_eq!(ids, vec![1]);
    }

    #[test]
    fn test_should_find_create_ids_with_bare_id_anchored_to_actor() {
        setup();
        let actor = "https://mastic.social/users/alice";
        let repo = InboxActivityRepository::oneshot();
        repo.insert(
            7,
            create_type(),
            actor,
            serde_json::json!({
                "type": "Create",
                "actor": actor,
                "object": { "type": "Note", "id": "42", "content": "hi" },
            }),
            false,
            None,
            1_000,
        )
        .expect("insert");

        let ids = repo
            .find_create_ids_with_object_id("42", Some(actor))
            .expect("query");
        assert_eq!(ids, vec![7]);
    }

    #[test]
    fn test_should_not_match_bare_id_for_wrong_actor() {
        setup();
        let alice = "https://mastic.social/users/alice";
        let bob = "https://mastic.social/users/bob";
        let repo = InboxActivityRepository::oneshot();
        repo.insert(
            7,
            create_type(),
            alice,
            serde_json::json!({
                "type": "Create",
                "actor": alice,
                "object": { "type": "Note", "id": "42", "content": "hi" },
            }),
            false,
            None,
            1_000,
        )
        .expect("insert");

        let ids = repo
            .find_create_ids_with_object_id("42", Some(bob))
            .expect("query");
        assert!(ids.is_empty());
    }

    #[test]
    fn test_should_skip_create_rows_pointing_at_other_object_id() {
        setup();
        let repo = InboxActivityRepository::oneshot();
        repo.insert(
            1,
            create_type(),
            "https://remote.example/users/a",
            serde_json::json!({
                "type": "Create",
                "actor": "https://remote.example/users/a",
                "object": {
                    "type": "Note",
                    "id": "https://remote.example/users/x/statuses/100",
                    "content": "hi",
                },
            }),
            false,
            None,
            1_000,
        )
        .expect("insert");

        let ids = repo
            .find_create_ids_with_object_id("https://remote.example/users/x/statuses/9", None)
            .expect("query");
        assert!(ids.is_empty());
    }

    #[test]
    fn test_delete_by_id_should_remove_inbox_row() {
        setup();
        let repo = InboxActivityRepository::oneshot();
        repo.insert(
            1,
            create_type(),
            "https://remote.example/users/a",
            serde_json::json!({"type": "Create"}),
            false,
            None,
            1_000,
        )
        .expect("insert");

        Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
            InboxActivityRepository::with_transaction(tx).delete_by_id(1)
        })
        .expect("delete");

        let remaining = InboxActivityRepository::oneshot()
            .find_create_ids_with_object_id("anything", None)
            .expect("query");
        assert!(remaining.is_empty());
    }
}
