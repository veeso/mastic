//! Follow request repository for managing incoming follow requests.

use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::*;

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{FollowRequest, FollowRequestInsertRequest, FollowRequestRecord, Schema};

/// Interface to access [`FollowRequest`] data.
pub struct FollowRequestRepository;

impl FollowRequestRepository {
    /// Insert a new follow request for the given actor URI.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "will be used by receive_activity handler")
    )]
    pub fn insert(actor_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            db.insert::<FollowRequest>(FollowRequestInsertRequest {
                actor_uri: actor_uri.into(),
                created_at: ic_utils::now().into(),
            })
            .map_err(CanisterError::from)
        })
    }

    /// Find a follow request by actor URI.
    pub fn find_by_actor_uri(actor_uri: &str) -> CanisterResult<Option<FollowRequest>> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            let records = db
                .select::<FollowRequest>(
                    Query::builder()
                        .all()
                        .and_where(Filter::eq("actor_uri", Value::from(actor_uri)))
                        .build(),
                )
                .map_err(CanisterError::from)?;

            Ok(records
                .into_iter()
                .next()
                .map(Self::record_to_follow_request))
        })
    }

    /// Get a paginated list of [`FollowRequest`]s.
    pub fn get_paginated(offset: usize, limit: usize) -> CanisterResult<Vec<FollowRequest>> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            let records = db
                .select::<FollowRequest>(Query::builder().all().offset(offset).limit(limit).build())
                .map_err(CanisterError::from)?;

            Ok(records
                .into_iter()
                .map(Self::record_to_follow_request)
                .collect())
        })
    }

    /// Delete a follow request by actor URI.
    pub fn delete_by_actor_uri(actor_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            db.delete::<FollowRequest>(
                DeleteBehavior::Restrict,
                Some(Filter::eq("actor_uri", Value::from(actor_uri.to_string()))),
            )
            .map(|_| ())
            .map_err(CanisterError::from)
        })
    }

    fn record_to_follow_request(record: FollowRequestRecord) -> FollowRequest {
        FollowRequest {
            actor_uri: record.actor_uri.expect("must have field"),
            created_at: record.created_at.expect("must have field"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::setup;

    #[test]
    fn test_should_insert_and_find_follow_request() {
        setup();

        FollowRequestRepository::insert("https://mastic.social/users/alice")
            .expect("should insert");

        let found = FollowRequestRepository::find_by_actor_uri("https://mastic.social/users/alice")
            .expect("should query")
            .expect("should find follow request");

        assert_eq!(found.actor_uri.0, "https://mastic.social/users/alice");
    }

    #[test]
    fn test_should_return_none_for_missing_follow_request() {
        setup();

        let found =
            FollowRequestRepository::find_by_actor_uri("https://mastic.social/users/nobody")
                .expect("should query");

        assert!(found.is_none());
    }

    #[test]
    fn test_should_delete_follow_request() {
        setup();

        FollowRequestRepository::insert("https://mastic.social/users/alice")
            .expect("should insert");

        FollowRequestRepository::delete_by_actor_uri("https://mastic.social/users/alice")
            .expect("should delete");

        let found = FollowRequestRepository::find_by_actor_uri("https://mastic.social/users/alice")
            .expect("should query");

        assert!(found.is_none());
    }
}
