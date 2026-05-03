//! User repository

use candid::Principal;
use ic_dbms_canister::prelude::{DBMS_CONTEXT, IcAccessControlList, IcMemoryProvider};
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms::prelude::DbmsContext;
use wasm_dbms_api::prelude::*;

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{
    Schema, User, UserCanisterStatus, UserInsertRequest, UserRecord, UserUpdateRequest,
};

/// Repository for user-related database operations.
pub struct UserRepository {
    tx: Option<TransactionId>,
}

impl UserRepository {
    pub const fn oneshot() -> Self {
        Self { tx: None }
    }

    // Reserved for future cross-repo atomic flows that need to splice user
    // reads/writes into an externally-driven transaction. Not yet wired up.
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

    /// Signs up a new user by creating a canister for them and storing their information in the database.
    ///
    /// The user canister is set to Null and the creation state is marked to pending.
    pub fn sign_up(&self, user_principal: Principal, handle: String) -> CanisterResult<()> {
        ic_utils::log!(
            "UserRepository::sign_up: inserting user {user_principal} with handle {handle:?}"
        );

        let insert = UserInsertRequest {
            principal: ic_dbms_canister::prelude::Principal(user_principal),
            handle: handle.into(),
            canister_id: Nullable::Null,
            canister_status: UserCanisterStatus::from(
                did::directory::UserCanisterStatus::CreationPending,
            ),
            created_at: ic_utils::now().into(),
        };

        DBMS_CONTEXT.with(|ctx| self.db(ctx).insert::<User>(insert))?;

        ic_utils::log!("UserRepository::sign_up: user {user_principal} inserted successfully");

        Ok(())
    }

    /// Sets the canister ID for a user after successful canister creation and updates their canister status to active.
    pub fn set_user_canister(
        &self,
        user_principal: Principal,
        canister_id: Principal,
    ) -> CanisterResult<()> {
        ic_utils::log!(
            "UserRepository::set_user_canister: setting canister {canister_id} for user {user_principal}"
        );
        let update = UserUpdateRequest {
            canister_id: Some(Nullable::Value(ic_dbms_canister::prelude::Principal(
                canister_id,
            ))),
            canister_status: Some(UserCanisterStatus::from(
                did::directory::UserCanisterStatus::Active,
            )),
            where_clause: Some(Filter::eq(
                User::primary_key(),
                ic_dbms_canister::prelude::Principal(user_principal).into(),
            )),
            ..Default::default()
        };

        let rows = DBMS_CONTEXT.with(|ctx| self.db(ctx).update::<User>(update))?;

        if rows == 0 {
            ic_utils::log!(
                "UserRepository::set_user_canister: no rows updated for user {user_principal}"
            );
            return Err(CanisterError::SignUpFailed(format!(
                "failed to set user canister id for user {user_principal}"
            )));
        }

        ic_utils::log!(
            "UserRepository::set_user_canister: canister {canister_id} set for user {user_principal}"
        );

        Ok(())
    }

    /// Sets the user canister status to creation failed if the canister creation process fails for a user.
    pub fn set_failed_user_canister_create(&self, user_principal: Principal) -> CanisterResult<()> {
        ic_utils::log!(
            "UserRepository::set_failed_user_canister_create: marking user {user_principal} as failed"
        );
        let update = UserUpdateRequest {
            canister_status: Some(UserCanisterStatus::from(
                did::directory::UserCanisterStatus::CreationFailed,
            )),
            where_clause: Some(Filter::eq(
                User::primary_key(),
                ic_dbms_canister::prelude::Principal(user_principal).into(),
            )),
            ..Default::default()
        };

        let rows = DBMS_CONTEXT.with(|ctx| self.db(ctx).update::<User>(update))?;

        if rows == 0 {
            ic_utils::log!(
                "UserRepository::set_failed_user_canister_create: no rows updated for user {user_principal}"
            );
            return Err(CanisterError::SignUpFailed(format!(
                "failed to set user canister creation failed for user {user_principal}"
            )));
        }

        ic_utils::log!(
            "UserRepository::set_failed_user_canister_create: user {user_principal} marked as failed"
        );

        Ok(())
    }

    /// Sets the user canister status to creation failed if the canister creation process fails for a user.
    pub fn retry_user_canister_creation(&self, user_principal: Principal) -> CanisterResult<()> {
        ic_utils::log!(
            "UserRepository::retry_user_canister_creation: retrying for user {user_principal}"
        );
        let update = UserUpdateRequest {
            canister_status: Some(UserCanisterStatus::from(
                did::directory::UserCanisterStatus::CreationPending,
            )),
            where_clause: Some(Filter::eq(
                User::primary_key(),
                ic_dbms_canister::prelude::Principal(user_principal).into(),
            )),
            ..Default::default()
        };

        let rows = DBMS_CONTEXT.with(|ctx| self.db(ctx).update::<User>(update))?;

        if rows == 0 {
            ic_utils::log!(
                "UserRepository::retry_user_canister_creation: no rows updated for user {user_principal}"
            );
            return Err(CanisterError::SignUpFailed(format!(
                "failed to retry user canister creation for user {user_principal}"
            )));
        }

        ic_utils::log!(
            "UserRepository::retry_user_canister_creation: user {user_principal} status reset to pending"
        );

        Ok(())
    }

    /// Retrieves a user's information from the database by their principal.
    pub fn get_user_by_principal(&self, user_principal: Principal) -> CanisterResult<Option<User>> {
        ic_utils::log!("UserRepository::get_user_by_principal: querying user {user_principal}");
        let rows = DBMS_CONTEXT.with(|ctx| {
            self.db(ctx).select::<User>(
                Query::builder()
                    .all()
                    .limit(1)
                    .and_where(Filter::eq(
                        User::primary_key(),
                        ic_dbms_canister::prelude::Principal(user_principal).into(),
                    ))
                    .build(),
            )
        })?;

        if rows.is_empty() {
            Ok(None)
        } else {
            let user = rows.into_iter().next().expect("row should exist");
            Ok(Some(Self::user_record_to_user(user)))
        }
    }

    /// Retrieves a user's information from the database by their handle.
    pub fn get_user_by_handle(&self, handle: &str) -> CanisterResult<Option<User>> {
        ic_utils::log!("UserRepository::get_user_by_handle: querying handle {handle:?}");
        let rows = DBMS_CONTEXT.with(|ctx| {
            self.db(ctx).select::<User>(
                Query::builder()
                    .all()
                    .limit(1)
                    .and_where(Filter::eq("handle", handle.into()))
                    .build(),
            )
        })?;

        if rows.is_empty() {
            Ok(None)
        } else {
            let user = rows.into_iter().next().expect("row should exist");
            Ok(Some(Self::user_record_to_user(user)))
        }
    }

    /// Marks a user for deletion by setting their canister status to deletion pending.
    /// The actual deletion of the user record and User Canister is handled asynchronously by the state machine.
    pub fn mark_user_for_deletion(&self, user_principal: Principal) -> CanisterResult<()> {
        ic_utils::log!(
            "UserRepository::mark_user_for_deletion: marking user {user_principal} for deletion"
        );
        let update = UserUpdateRequest {
            canister_status: Some(UserCanisterStatus(
                did::directory::UserCanisterStatus::DeletionPending,
            )),
            where_clause: Some(Filter::eq(
                User::primary_key(),
                ic_dbms_canister::prelude::Principal(user_principal).into(),
            )),
            ..Default::default()
        };
        let rows = DBMS_CONTEXT.with(|ctx| self.db(ctx).update::<User>(update))?;

        if rows == 0 {
            return Err(CanisterError::SignUpFailed(format!(
                "failed to mark user {user_principal} for deletion"
            )));
        }

        Ok(())
    }

    /// Removes a user record from the database. Called by the delete_profile state machine
    /// after the user canister has been stopped and deleted.
    pub fn remove_user(&self, user_principal: Principal) -> CanisterResult<()> {
        ic_utils::log!("UserRepository::remove_user: removing user {user_principal}");

        let deleted = DBMS_CONTEXT.with(|ctx| {
            self.db(ctx).delete::<User>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    User::primary_key(),
                    ic_dbms_canister::prelude::Principal(user_principal).into(),
                )),
            )
        })?;

        if deleted == 0 {
            return Err(CanisterError::SignUpFailed(format!(
                "no user record found for {user_principal}"
            )));
        }

        Ok(())
    }

    /// Searches for user profiles based on a query string that matches the handle, with pagination support using offset and limit.
    ///
    /// Search only those which are `Active` and have a canister id.
    pub fn search_profiles(
        &self,
        handle: &str,
        offset: usize,
        limit: usize,
    ) -> CanisterResult<Vec<User>> {
        ic_utils::log!(
            "UserRepository::search_profiles: searching for handle {handle:?} with offset {offset} and limit {limit}"
        );
        let rows = DBMS_CONTEXT.with(|ctx| {
            self.db(ctx).select::<User>(
                Query::builder()
                    .all()
                    .offset(offset)
                    .limit(limit)
                    .and_where(Filter::like("handle", &format!("%{handle}%")))
                    .and_where(Filter::eq(
                        "canister_status",
                        crate::schema::UserCanisterStatus(
                            did::directory::UserCanisterStatus::Active,
                        )
                        .into(),
                    ))
                    .and_where(Filter::not_null("canister_id"))
                    .build(),
            )
        })?;

        Ok(rows.into_iter().map(Self::user_record_to_user).collect())
    }

    fn user_record_to_user(user: UserRecord) -> User {
        User {
            principal: user.principal.expect("principal cannot be empty"),
            handle: user.handle.expect("handle cannot be empty"),
            canister_id: user.canister_id.expect("canister_id cannot be empty"),
            canister_status: user
                .canister_status
                .expect("canister_status cannot be empty"),
            created_at: user.created_at.expect("created_at cannot be empty"),
        }
    }
}

#[cfg(test)]
mod tests {

    use db_utils::transaction::Transaction;

    use super::*;
    use crate::test_utils::{bob, rey_canisteryo, setup};

    #[test]
    fn test_should_sign_up_user() {
        setup();

        UserRepository::oneshot()
            .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())
            .expect("should sign up user");

        let user = UserRepository::oneshot()
            .get_user_by_principal(rey_canisteryo())
            .expect("should query user")
            .expect("user should exist");

        assert_eq!(user.principal.0, rey_canisteryo());
        assert_eq!(user.handle.0, "rey_canisteryo");
        assert!(user.canister_id.is_null());
        assert_eq!(
            did::directory::UserCanisterStatus::from(user.canister_status),
            did::directory::UserCanisterStatus::CreationPending
        );
    }

    #[test]
    fn test_should_reject_duplicate_principal_on_sign_up() {
        setup();

        UserRepository::oneshot()
            .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())
            .expect("should sign up user");

        let result = UserRepository::oneshot().sign_up(rey_canisteryo(), "alice2".to_string());
        assert!(result.is_err(), "duplicate principal should be rejected");
    }

    #[test]
    fn test_should_reject_duplicate_handle_on_sign_up() {
        setup();

        UserRepository::oneshot()
            .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())
            .expect("should sign up user");

        let result = UserRepository::oneshot().sign_up(bob(), "rey_canisteryo".to_string());
        assert!(result.is_err(), "duplicate handle should be rejected");
    }

    #[test]
    fn test_should_set_user_canister() {
        setup();

        UserRepository::oneshot()
            .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())
            .expect("should sign up user");

        let canister_id = bob();
        UserRepository::oneshot()
            .set_user_canister(rey_canisteryo(), canister_id)
            .expect("should set user canister");

        let user = UserRepository::oneshot()
            .get_user_by_principal(rey_canisteryo())
            .expect("should query user")
            .expect("user should exist");

        let Nullable::Value(cid) = user.canister_id else {
            panic!("canister_id should be set");
        };
        assert_eq!(cid.0, canister_id);
        assert_eq!(
            did::directory::UserCanisterStatus::from(user.canister_status),
            did::directory::UserCanisterStatus::Active
        );
    }

    #[test]
    fn test_should_fail_set_user_canister_for_unknown_principal() {
        setup();

        let result = UserRepository::oneshot().set_user_canister(rey_canisteryo(), bob());
        assert!(result.is_err(), "should fail for unknown principal");
    }

    #[test]
    fn test_should_set_failed_user_canister_create() {
        setup();

        UserRepository::oneshot()
            .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())
            .expect("should sign up user");

        UserRepository::oneshot()
            .set_failed_user_canister_create(rey_canisteryo())
            .expect("should set canister creation failed");

        let user = UserRepository::oneshot()
            .get_user_by_principal(rey_canisteryo())
            .expect("should query user")
            .expect("user should exist");

        assert_eq!(
            did::directory::UserCanisterStatus::from(user.canister_status),
            did::directory::UserCanisterStatus::CreationFailed
        );
    }

    #[test]
    fn test_should_fail_set_failed_canister_create_for_unknown_principal() {
        setup();

        let result = UserRepository::oneshot().set_failed_user_canister_create(rey_canisteryo());
        assert!(result.is_err(), "should fail for unknown principal");
    }

    #[test]
    fn test_should_retry_user_canister_creation() {
        setup();

        UserRepository::oneshot()
            .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())
            .expect("should sign up user");

        UserRepository::oneshot()
            .set_failed_user_canister_create(rey_canisteryo())
            .expect("should set canister creation failed");

        UserRepository::oneshot()
            .retry_user_canister_creation(rey_canisteryo())
            .expect("should retry user canister creation");

        let user = UserRepository::oneshot()
            .get_user_by_principal(rey_canisteryo())
            .expect("should query user")
            .expect("user should exist");

        assert_eq!(
            did::directory::UserCanisterStatus::from(user.canister_status),
            did::directory::UserCanisterStatus::CreationPending
        );
    }

    #[test]
    fn test_should_get_user_by_principal() {
        setup();

        UserRepository::oneshot()
            .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())
            .expect("should sign up user");

        let user = UserRepository::oneshot()
            .get_user_by_principal(rey_canisteryo())
            .expect("should query user")
            .expect("user should exist");

        assert_eq!(user.principal.0, rey_canisteryo());
        assert_eq!(user.handle.0, "rey_canisteryo");
    }

    #[test]
    fn test_should_return_none_for_unknown_principal() {
        setup();

        let user = UserRepository::oneshot()
            .get_user_by_principal(rey_canisteryo())
            .expect("should query user");
        assert!(user.is_none(), "should return None for unknown principal");
    }

    #[test]
    fn test_should_get_user_by_handle() {
        setup();

        UserRepository::oneshot()
            .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())
            .expect("should sign up user");

        let user = UserRepository::oneshot()
            .get_user_by_handle("rey_canisteryo")
            .expect("should query user")
            .expect("user should exist");

        assert_eq!(user.principal.0, rey_canisteryo());
        assert_eq!(user.handle.0, "rey_canisteryo");
    }

    #[test]
    fn test_should_return_none_for_unknown_handle() {
        setup();

        let user = UserRepository::oneshot()
            .get_user_by_handle("nonexistent")
            .expect("should query");
        assert!(user.is_none(), "should return None for unknown handle");
    }

    /// Sets the canister status of a user to an arbitrary value (test-only helper).
    fn set_canister_status(principal: Principal, status: did::directory::UserCanisterStatus) {
        let update = UserUpdateRequest {
            canister_status: Some(UserCanisterStatus::from(status)),
            where_clause: Some(Filter::eq(
                User::primary_key(),
                ic_dbms_canister::prelude::Principal(principal).into(),
            )),
            ..Default::default()
        };

        let rows = DBMS_CONTEXT
            .with(|ctx| {
                let dbms = WasmDbmsDatabase::oneshot(ctx, Schema);
                dbms.update::<User>(update)
            })
            .expect("update should succeed");
        assert_eq!(rows, 1, "should update exactly one row");
    }

    fn alice_user() -> Principal {
        Principal::self_authenticating([10u8; 32])
    }

    fn other_user() -> Principal {
        Principal::self_authenticating([11u8; 32])
    }

    fn canister(seed: u8) -> Principal {
        Principal::self_authenticating([seed; 32])
    }

    /// Seeds two Active users named `alice` and `alicia`, both with a canister id.
    /// Used by tests that exercise matching/pagination behavior.
    fn seed_two_active_alikes() {
        let repo = UserRepository::oneshot();
        repo.sign_up(alice_user(), "alice".to_string()).unwrap();
        repo.set_user_canister(alice_user(), canister(100)).unwrap();
        repo.sign_up(other_user(), "alicia".to_string()).unwrap();
        repo.set_user_canister(other_user(), canister(101)).unwrap();
    }

    #[test]
    fn test_search_profiles_should_match_exact_handle() {
        setup();
        seed_two_active_alikes();

        let users = UserRepository::oneshot()
            .search_profiles("alice", 0, 50)
            .expect("search should succeed");
        let handles: Vec<_> = users.iter().map(|u| u.handle.0.as_str()).collect();
        assert!(handles.contains(&"alice"));
    }

    #[test]
    fn test_search_profiles_should_match_prefix_substring() {
        setup();
        seed_two_active_alikes();

        let users = UserRepository::oneshot()
            .search_profiles("ali", 0, 50)
            .expect("search should succeed");
        let handles: Vec<_> = users.iter().map(|u| u.handle.0.as_str()).collect();
        assert!(handles.contains(&"alice"));
        assert!(handles.contains(&"alicia"));
        assert_eq!(handles.len(), 2);
    }

    #[test]
    fn test_search_profiles_should_match_middle_substring() {
        setup();
        seed_two_active_alikes();

        let users = UserRepository::oneshot()
            .search_profiles("lic", 0, 50)
            .expect("search should succeed");
        let handles: Vec<_> = users.iter().map(|u| u.handle.0.as_str()).collect();
        assert!(handles.contains(&"alice"));
        assert!(handles.contains(&"alicia"));
    }

    #[test]
    fn test_search_profiles_empty_query_returns_all_active() {
        setup();
        seed_two_active_alikes();

        let users = UserRepository::oneshot()
            .search_profiles("", 0, 50)
            .expect("search should succeed");
        assert_eq!(users.len(), 2);
    }

    #[test]
    fn test_search_profiles_pagination() {
        setup();
        seed_two_active_alikes();

        let page1 = UserRepository::oneshot()
            .search_profiles("", 0, 1)
            .expect("search should succeed");
        assert_eq!(page1.len(), 1);

        let page2 = UserRepository::oneshot()
            .search_profiles("", 1, 1)
            .expect("search should succeed");
        assert_eq!(page2.len(), 1);
        assert_ne!(page1[0].handle.0, page2[0].handle.0);

        let page3 = UserRepository::oneshot()
            .search_profiles("", 2, 1)
            .expect("search should succeed");
        assert!(page3.is_empty());
    }

    #[test]
    fn test_search_profiles_no_match_returns_empty() {
        setup();
        seed_two_active_alikes();

        let users = UserRepository::oneshot()
            .search_profiles("zorblax", 0, 50)
            .expect("search should succeed");
        assert!(users.is_empty());
    }

    #[test]
    fn test_search_profiles_should_exclude_creation_pending() {
        setup();
        // CreationPending: signed up, no canister set yet.
        UserRepository::oneshot()
            .sign_up(alice_user(), "alice".to_string())
            .unwrap();

        let users = UserRepository::oneshot()
            .search_profiles("alice", 0, 50)
            .expect("search should succeed");
        assert!(users.is_empty(), "CreationPending users must be excluded");
    }

    #[test]
    fn test_search_profiles_should_exclude_creation_failed() {
        setup();
        UserRepository::oneshot()
            .sign_up(alice_user(), "alice".to_string())
            .unwrap();
        UserRepository::oneshot()
            .set_failed_user_canister_create(alice_user())
            .unwrap();

        let users = UserRepository::oneshot()
            .search_profiles("alice", 0, 50)
            .expect("search should succeed");
        assert!(users.is_empty(), "CreationFailed users must be excluded");
    }

    #[test]
    fn test_search_profiles_should_exclude_deletion_pending() {
        setup();
        UserRepository::oneshot()
            .sign_up(alice_user(), "alice".to_string())
            .unwrap();
        UserRepository::oneshot()
            .set_user_canister(alice_user(), canister(100))
            .unwrap();
        UserRepository::oneshot()
            .mark_user_for_deletion(alice_user())
            .unwrap();

        let users = UserRepository::oneshot()
            .search_profiles("alice", 0, 50)
            .expect("search should succeed");
        assert!(users.is_empty(), "DeletionPending users must be excluded");
    }

    #[test]
    fn test_search_profiles_should_exclude_suspended() {
        setup();
        UserRepository::oneshot()
            .sign_up(alice_user(), "alice".to_string())
            .unwrap();
        UserRepository::oneshot()
            .set_user_canister(alice_user(), canister(100))
            .unwrap();
        set_canister_status(alice_user(), did::directory::UserCanisterStatus::Suspended);

        let users = UserRepository::oneshot()
            .search_profiles("alice", 0, 50)
            .expect("search should succeed");
        assert!(users.is_empty(), "Suspended users must be excluded");
    }

    // Transaction-aware tests: validate that callers can splice repository
    // operations into an externally-driven transaction (commit + rollback).

    #[test]
    fn test_should_sign_up_user_in_transaction_and_commit() {
        setup();

        Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
            UserRepository::with_transaction(tx)
                .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())
        })
        .expect("transaction should commit");

        let user = UserRepository::oneshot()
            .get_user_by_principal(rey_canisteryo())
            .expect("should query user")
            .expect("user should exist after commit");
        assert_eq!(user.handle.0, "rey_canisteryo");
    }

    #[test]
    fn test_should_rollback_sign_up_when_transaction_errors() {
        setup();

        let result: Result<(), CanisterError> = Transaction::run(Schema, |tx| {
            UserRepository::with_transaction(tx)
                .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())?;
            Err(CanisterError::Internal("boom".to_string()))
        });
        assert!(result.is_err());

        let user = UserRepository::oneshot()
            .get_user_by_principal(rey_canisteryo())
            .expect("should query user");
        assert!(
            user.is_none(),
            "user must not persist when transaction rolls back"
        );
    }

    #[test]
    fn test_should_atomically_sign_up_and_set_canister_in_one_transaction() {
        setup();

        Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
            let repo = UserRepository::with_transaction(tx);
            repo.sign_up(rey_canisteryo(), "rey_canisteryo".to_string())?;
            repo.set_user_canister(rey_canisteryo(), bob())
        })
        .expect("transaction should commit");

        let user = UserRepository::oneshot()
            .get_user_by_principal(rey_canisteryo())
            .expect("should query user")
            .expect("user should exist after commit");

        let Nullable::Value(cid) = user.canister_id else {
            panic!("canister_id should be set");
        };
        assert_eq!(cid.0, bob());
        assert_eq!(
            did::directory::UserCanisterStatus::from(user.canister_status),
            did::directory::UserCanisterStatus::Active
        );
    }

    #[test]
    fn test_should_rollback_combined_writes_when_second_step_errors() {
        setup();

        // First do a valid sign_up.
        UserRepository::oneshot()
            .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())
            .expect("should sign up user");

        // Now try a tx that updates canister id and then errors. The update
        // must not be visible after rollback.
        let result: Result<(), CanisterError> = Transaction::run(Schema, |tx| {
            UserRepository::with_transaction(tx).set_user_canister(rey_canisteryo(), bob())?;
            Err(CanisterError::Internal("boom".to_string()))
        });
        assert!(result.is_err());

        let user = UserRepository::oneshot()
            .get_user_by_principal(rey_canisteryo())
            .expect("should query user")
            .expect("user must still exist");
        assert!(
            user.canister_id.is_null(),
            "canister update must roll back on tx error"
        );
        assert_eq!(
            did::directory::UserCanisterStatus::from(user.canister_status),
            did::directory::UserCanisterStatus::CreationPending,
            "status must roll back on tx error"
        );
    }

    #[test]
    fn test_should_remove_user_in_transaction() {
        setup();

        UserRepository::oneshot()
            .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())
            .expect("should sign up user");

        Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
            UserRepository::with_transaction(tx).remove_user(rey_canisteryo())
        })
        .expect("transaction should commit");

        let user = UserRepository::oneshot()
            .get_user_by_principal(rey_canisteryo())
            .expect("should query user");
        assert!(user.is_none(), "user must be removed after commit");
    }

    #[test]
    fn test_should_rollback_remove_user_when_transaction_errors() {
        setup();

        UserRepository::oneshot()
            .sign_up(rey_canisteryo(), "rey_canisteryo".to_string())
            .expect("should sign up user");

        let result: Result<(), CanisterError> = Transaction::run(Schema, |tx| {
            UserRepository::with_transaction(tx).remove_user(rey_canisteryo())?;
            Err(CanisterError::Internal("boom".to_string()))
        });
        assert!(result.is_err());

        let user = UserRepository::oneshot()
            .get_user_by_principal(rey_canisteryo())
            .expect("should query user");
        assert!(user.is_some(), "user must persist when tx rolls back");
    }
}
