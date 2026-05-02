//! Follow request repository for managing incoming follow requests.

use ic_dbms_canister::prelude::{DBMS_CONTEXT, IcAccessControlList, IcMemoryProvider};
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms::prelude::DbmsContext;
use wasm_dbms_api::prelude::*;

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{FollowRequest, FollowRequestInsertRequest, FollowRequestRecord, Schema};

/// Interface to access [`FollowRequest`] data.
pub struct FollowRequestRepository {
    tx: Option<TransactionId>,
}

impl FollowRequestRepository {
    pub const fn oneshot() -> Self {
        Self { tx: None }
    }

    // Reserved for future cross-repo atomic flows that need to splice follow
    // request reads/writes into an externally-driven transaction. Not yet wired
    // up.
    #[allow(dead_code)]
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

    /// Insert a new follow request for the given actor URI.
    pub fn insert(&self, actor_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx)
                .insert::<FollowRequest>(FollowRequestInsertRequest {
                    actor_uri: actor_uri.into(),
                    created_at: ic_utils::now().into(),
                })
                .map_err(CanisterError::from)
        })
    }

    /// Find a follow request by actor URI.
    pub fn find_by_actor_uri(&self, actor_uri: &str) -> CanisterResult<Option<FollowRequest>> {
        DBMS_CONTEXT.with(|ctx| {
            let records = self
                .db(ctx)
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
    pub fn get_paginated(&self, offset: usize, limit: usize) -> CanisterResult<Vec<FollowRequest>> {
        DBMS_CONTEXT.with(|ctx| {
            let records = self
                .db(ctx)
                .select::<FollowRequest>(Query::builder().all().offset(offset).limit(limit).build())
                .map_err(CanisterError::from)?;

            Ok(records
                .into_iter()
                .map(Self::record_to_follow_request)
                .collect())
        })
    }

    /// Delete a follow request by actor URI.
    pub fn delete_by_actor_uri(&self, actor_uri: &str) -> CanisterResult<()> {
        DBMS_CONTEXT.with(|ctx| {
            self.db(ctx)
                .delete::<FollowRequest>(
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

        FollowRequestRepository::oneshot()
            .insert("https://mastic.social/users/alice")
            .expect("should insert");

        let found = FollowRequestRepository::oneshot()
            .find_by_actor_uri("https://mastic.social/users/alice")
            .expect("should query")
            .expect("should find follow request");

        assert_eq!(found.actor_uri.0, "https://mastic.social/users/alice");
    }

    #[test]
    fn test_should_return_none_for_missing_follow_request() {
        setup();

        let found = FollowRequestRepository::oneshot()
            .find_by_actor_uri("https://mastic.social/users/nobody")
            .expect("should query");

        assert!(found.is_none());
    }

    #[test]
    fn test_should_delete_follow_request() {
        setup();

        FollowRequestRepository::oneshot()
            .insert("https://mastic.social/users/alice")
            .expect("should insert");

        FollowRequestRepository::oneshot()
            .delete_by_actor_uri("https://mastic.social/users/alice")
            .expect("should delete");

        let found = FollowRequestRepository::oneshot()
            .find_by_actor_uri("https://mastic.social/users/alice")
            .expect("should query");

        assert!(found.is_none());
    }
}
