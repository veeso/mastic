//! Database schema for the Directory canister.

mod user_canister_status;

use db_utils::handle::{HandleSanitizer, HandleValidator};
use db_utils::settings::*;
use ic_dbms_canister::prelude::*;

pub use self::user_canister_status::UserCanisterStatus;

/// Represents a moderator in the Directory canister.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "moderators"]
pub struct Moderator {
    /// The principal of the moderator.
    #[primary_key]
    #[custom_type]
    pub principal: Principal,
    /// The date and time when the moderator was added.
    pub created_at: Uint64,
}

/// User registered in the Directory canister.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "users"]
pub struct User {
    /// The principal of the user.
    #[primary_key]
    #[custom_type]
    pub principal: Principal,
    /// User's unique handle.
    #[unique]
    #[sanitizer(HandleSanitizer)]
    #[validate(HandleValidator)]
    pub handle: Text,
    /// User canister ID.
    #[unique]
    #[custom_type]
    pub canister_id: Nullable<Principal>,
    /// User canister status.
    #[custom_type]
    pub canister_status: UserCanisterStatus,
    /// The date and time when the user was added.
    pub created_at: Uint64,
}

#[derive(DatabaseSchema)]
#[tables(Settings = "settings", Moderator = "moderators", User = "users")]
pub struct Schema;

#[cfg(test)]
mod tests {

    use ic_dbms_canister::prelude::*;
    use wasm_dbms::WasmDbmsDatabase;
    use wasm_dbms_api::prelude::{Database, DeleteBehavior, Filter, Query};

    use super::*;
    use crate::test_utils::{bob, rey_canisteryo, setup};

    #[test]
    fn test_should_insert_and_query_user() {
        setup();

        let principal = Principal(rey_canisteryo());
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<User>(UserInsertRequest {
                principal: principal.clone(),
                handle: "alice".into(),
                canister_id: Nullable::Value(Principal(bob())),
                canister_status: UserCanisterStatus::default(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert user");

            let rows = db
                .select::<User>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "principal",
                            wasm_dbms_api::prelude::Value::from(principal),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select user");

            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].handle.as_ref().expect("handle").0, "alice");
        });
    }

    #[test]
    fn test_should_enforce_unique_handle() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<User>(UserInsertRequest {
                principal: Principal(rey_canisteryo()),
                handle: "alice".into(),
                canister_id: Nullable::Null,
                canister_status: UserCanisterStatus::default(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert first user");

            let result = db.insert::<User>(UserInsertRequest {
                principal: Principal(bob()),
                handle: "alice".into(),
                canister_id: Nullable::Null,
                canister_status: UserCanisterStatus::default(),
                created_at: ic_utils::now().into(),
            });

            assert!(result.is_err(), "duplicate handle should be rejected");
        });
    }

    #[test]
    fn test_should_reject_invalid_handle() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<User>(UserInsertRequest {
                principal: Principal(rey_canisteryo()),
                handle: "INVALID!".into(),
                canister_id: Nullable::Null,
                canister_status: UserCanisterStatus::default(),
                created_at: ic_utils::now().into(),
            });

            assert!(result.is_err(), "invalid handle should be rejected");
        });
    }

    #[test]
    fn test_should_reject_reserved_handle() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<User>(UserInsertRequest {
                principal: Principal(rey_canisteryo()),
                handle: "admin".into(),
                canister_id: Nullable::Null,
                canister_status: UserCanisterStatus::default(),
                created_at: ic_utils::now().into(),
            });

            assert!(result.is_err(), "reserved handle should be rejected");
        });
    }

    #[test]
    fn test_should_sanitize_handle_on_insert() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<User>(UserInsertRequest {
                principal: Principal(rey_canisteryo()),
                handle: "  @Alice  ".into(),
                canister_id: Nullable::Null,
                canister_status: UserCanisterStatus::default(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert user with sanitized handle");

            let rows = db
                .select::<User>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "handle",
                            wasm_dbms_api::prelude::Value::from("alice".to_string()),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select user by sanitized handle");

            assert_eq!(rows.len(), 1, "user should be found by sanitized handle");
        });
    }

    #[test]
    fn test_should_insert_user_with_null_canister_id() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<User>(UserInsertRequest {
                principal: Principal(rey_canisteryo()),
                handle: "alice".into(),
                canister_id: Nullable::Null,
                canister_status: UserCanisterStatus::default(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert user with null canister_id");

            let rows = db
                .select::<User>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "principal",
                            wasm_dbms_api::prelude::Value::from(Principal(rey_canisteryo())),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select user");

            assert_eq!(rows.len(), 1);
            assert!(
                rows[0].canister_id.as_ref().expect("canister_id").is_null(),
                "canister_id should be null"
            );
        });
    }

    #[test]
    fn test_should_delete_user() {
        setup();

        let principal = Principal(rey_canisteryo());
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<User>(UserInsertRequest {
                principal: principal.clone(),
                handle: "alice".into(),
                canister_id: Nullable::Null,
                canister_status: UserCanisterStatus::default(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert user");

            db.delete::<User>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    "principal",
                    wasm_dbms_api::prelude::Value::from(principal.clone()),
                )),
            )
            .expect("should delete user");

            let rows = db
                .select::<User>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "principal",
                            wasm_dbms_api::prelude::Value::from(principal),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select user");

            assert!(rows.is_empty(), "user should be deleted");
        });
    }
}
