//! Database schema for the Directory canister.

mod report_state;
mod user_canister_status;

use db_utils::bounded_text::{BoundedTextValidator, TrimSanitizer};
use db_utils::handle::{HandleSanitizer, HandleValidator};
use db_utils::settings::*;
use db_utils::url::NullableUrlValidator;
use ic_dbms_canister::prelude::*;

/// Maximum length of a `reports.reason` value.
pub const MAX_REPORT_REASON_LENGTH: usize = 1000;

pub use self::report_state::ReportState;
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

/// Tombstone for a deleted user handle.
///
/// Retained to prevent immediate re-registration of a handle after
/// deletion and to keep an audit trail of deleted accounts.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "tombstones"]
pub struct Tombstone {
    /// The deleted user's handle.
    #[primary_key]
    #[sanitizer(HandleSanitizer)]
    #[validate(HandleValidator)]
    pub handle: Text,
    /// The deleted user's principal.
    #[custom_type]
    pub principal: Principal,
    /// The date and time when the account was deleted.
    pub deleted_at: Uint64,
}

/// A user report submitted to the Directory for moderator review.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "reports"]
pub struct Report {
    /// Snowflake ID of the report.
    /// See Snowflake.md for more details.
    #[primary_key]
    pub id: Uint64,
    /// Principal of the reporter.
    #[custom_type]
    pub reporter: Principal,
    /// Principal of the reported user's canister.
    #[custom_type]
    pub target_canister: Principal,
    /// Optional URI of the specific status being reported.
    #[validate(NullableUrlValidator)]
    pub target_status_uri: Nullable<Text>,
    /// Free-form reason provided by the reporter.
    #[sanitizer(TrimSanitizer)]
    #[validate(BoundedTextValidator(MAX_REPORT_REASON_LENGTH))]
    pub reason: Text,
    /// Lifecycle state of the report.
    #[custom_type]
    pub state: ReportState,
    /// Created at timestamp.
    /// Indexed for efficient retrieval of recent reports.
    #[index]
    pub created_at: Uint64,
    /// Timestamp when the report was resolved or dismissed.
    pub resolved_at: Nullable<Uint64>,
    /// Principal of the moderator who resolved the report.
    #[custom_type]
    pub resolved_by: Nullable<Principal>,
}

#[derive(DatabaseSchema, Clone)]
#[tables(
    Settings = "settings",
    Moderator = "moderators",
    User = "users",
    Tombstone = "tombstones",
    Report = "reports"
)]
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
    fn test_should_insert_and_query_tombstone() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Tombstone>(TombstoneInsertRequest {
                handle: "alice".into(),
                principal: Principal(rey_canisteryo()),
                deleted_at: ic_utils::now().into(),
            })
            .expect("should insert tombstone");

            let rows = db
                .select::<Tombstone>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "handle",
                            wasm_dbms_api::prelude::Value::from("alice".to_string()),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select tombstone");

            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].handle.as_ref().expect("handle").0, "alice");
        });
    }

    #[test]
    fn test_should_enforce_unique_tombstone_handle() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Tombstone>(TombstoneInsertRequest {
                handle: "alice".into(),
                principal: Principal(rey_canisteryo()),
                deleted_at: ic_utils::now().into(),
            })
            .expect("should insert first tombstone");

            let result = db.insert::<Tombstone>(TombstoneInsertRequest {
                handle: "alice".into(),
                principal: Principal(bob()),
                deleted_at: ic_utils::now().into(),
            });

            assert!(result.is_err(), "duplicate tombstone handle rejected");
        });
    }

    #[test]
    fn test_should_delete_tombstone() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Tombstone>(TombstoneInsertRequest {
                handle: "alice".into(),
                principal: Principal(rey_canisteryo()),
                deleted_at: ic_utils::now().into(),
            })
            .expect("should insert tombstone");

            db.delete::<Tombstone>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    "handle",
                    wasm_dbms_api::prelude::Value::from("alice".to_string()),
                )),
            )
            .expect("should delete tombstone");

            let rows = db
                .select::<Tombstone>(Query::builder().build())
                .expect("should select tombstones");

            assert!(rows.is_empty(), "tombstone should be deleted");
        });
    }

    #[test]
    fn test_should_insert_and_query_report() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Report>(ReportInsertRequest {
                id: 1u64.into(),
                reporter: Principal(rey_canisteryo()),
                target_canister: Principal(bob()),
                target_status_uri: Nullable::Value("https://example.com/statuses/1".into()),
                reason: "spam".into(),
                state: ReportState::default(),
                created_at: ic_utils::now().into(),
                resolved_at: Nullable::Null,
                resolved_by: Nullable::Null,
            })
            .expect("should insert report");

            let rows = db
                .select::<Report>(
                    Query::builder()
                        .and_where(Filter::eq("id", wasm_dbms_api::prelude::Value::from(1u64)))
                        .limit(1)
                        .build(),
                )
                .expect("should select report");

            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].reason.as_ref().expect("reason").0, "spam");
        });
    }

    #[test]
    fn test_should_insert_resolved_report() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Report>(ReportInsertRequest {
                id: 2u64.into(),
                reporter: Principal(rey_canisteryo()),
                target_canister: Principal(bob()),
                target_status_uri: Nullable::Null,
                reason: "abuse".into(),
                state: ReportState::Resolved,
                created_at: ic_utils::now().into(),
                resolved_at: Nullable::Value(ic_utils::now().into()),
                resolved_by: Nullable::Value(Principal(rey_canisteryo())),
            })
            .expect("should insert resolved report");

            let rows = db
                .select::<Report>(
                    Query::builder()
                        .and_where(Filter::eq("id", wasm_dbms_api::prelude::Value::from(2u64)))
                        .limit(1)
                        .build(),
                )
                .expect("should select report");

            assert_eq!(rows.len(), 1);
            assert!(!rows[0].resolved_at.as_ref().expect("resolved_at").is_null());
        });
    }

    #[test]
    fn test_should_reject_report_reason_over_limit() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<Report>(ReportInsertRequest {
                id: 10u64.into(),
                reporter: Principal(rey_canisteryo()),
                target_canister: Principal(bob()),
                target_status_uri: Nullable::Null,
                reason: "x".repeat(MAX_REPORT_REASON_LENGTH + 1).into(),
                state: ReportState::default(),
                created_at: ic_utils::now().into(),
                resolved_at: Nullable::Null,
                resolved_by: Nullable::Null,
            });
            assert!(result.is_err(), "reason over limit should be rejected");
        });
    }

    #[test]
    fn test_should_reject_invalid_report_target_status_uri() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<Report>(ReportInsertRequest {
                id: 11u64.into(),
                reporter: Principal(rey_canisteryo()),
                target_canister: Principal(bob()),
                target_status_uri: Nullable::Value("not-a-url".into()),
                reason: "x".into(),
                state: ReportState::default(),
                created_at: ic_utils::now().into(),
                resolved_at: Nullable::Null,
                resolved_by: Nullable::Null,
            });
            assert!(result.is_err(), "invalid target_status_uri rejected");
        });
    }

    #[test]
    fn test_should_sanitize_tombstone_handle() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Tombstone>(TombstoneInsertRequest {
                handle: "  @Alice  ".into(),
                principal: Principal(rey_canisteryo()),
                deleted_at: ic_utils::now().into(),
            })
            .expect("should insert tombstone with sanitized handle");

            let rows = db
                .select::<Tombstone>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "handle",
                            wasm_dbms_api::prelude::Value::from("alice".to_string()),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select tombstone by sanitized handle");
            assert_eq!(rows.len(), 1);
        });
    }

    #[test]
    fn test_should_reject_invalid_tombstone_handle() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<Tombstone>(TombstoneInsertRequest {
                handle: "INVALID!".into(),
                principal: Principal(rey_canisteryo()),
                deleted_at: ic_utils::now().into(),
            });
            assert!(result.is_err(), "invalid tombstone handle rejected");
        });
    }

    #[test]
    fn test_should_delete_report() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Report>(ReportInsertRequest {
                id: 3u64.into(),
                reporter: Principal(rey_canisteryo()),
                target_canister: Principal(bob()),
                target_status_uri: Nullable::Null,
                reason: "delete me".into(),
                state: ReportState::default(),
                created_at: ic_utils::now().into(),
                resolved_at: Nullable::Null,
                resolved_by: Nullable::Null,
            })
            .expect("should insert report");

            db.delete::<Report>(
                DeleteBehavior::Restrict,
                Some(Filter::eq("id", wasm_dbms_api::prelude::Value::from(3u64))),
            )
            .expect("should delete report");

            let rows = db
                .select::<Report>(Query::builder().build())
                .expect("should select reports");

            assert!(rows.is_empty(), "report should be deleted");
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
