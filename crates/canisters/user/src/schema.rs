//! Database schema for the user canister.

mod activity;
mod follow_status;
mod status;
mod visibility;

use db_utils::handle::{HandleSanitizer, HandleValidator};
use db_utils::settings::*;
use ic_dbms_canister::prelude::Principal;
use wasm_dbms_api::prelude::*;

pub use self::activity::ActivityType;
pub use self::follow_status::FollowStatus;
pub use self::status::{StatusContentSanitizer, StatusContentValidator};
pub use self::visibility::Visibility;

/// Profile of the user in the canister.
///
/// This is a single row table, because we only have one user per canister.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "profiles"]
pub struct Profile {
    /// The principal of the user.
    #[primary_key]
    #[custom_type]
    pub principal: Principal,
    /// User's unique handle.
    #[unique]
    #[sanitizer(HandleSanitizer)]
    #[validate(HandleValidator)]
    pub handle: Text,
    /// Display name of the user.
    pub display_name: Nullable<Text>,
    /// Bio of the user.
    pub bio: Nullable<Text>,
    /// Avatar data of the user.
    pub avatar_data: Nullable<Blob>,
    /// Header data of the user. (banner)
    pub header_data: Nullable<Blob>,
    /// Created at timestamp.
    pub created_at: Uint64,
    /// Updated at timestamp.
    pub updated_at: Uint64,
}

/// A status posted by the user.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "statuses"]
pub struct Status {
    /// Snowflake ID of the status.
    /// See Snowflake.md for more details.
    #[primary_key]
    pub id: Uint64,
    /// Status content.
    #[sanitizer(StatusContentSanitizer)]
    #[validate(StatusContentValidator)]
    pub content: Text,
    /// Visibility of the status.
    /// See [`Visibility`](did::common::Visibility) enum for more details.
    #[custom_type]
    pub visibility: Visibility,
    /// Created at timestamp.
    /// Index for efficient retrieval of recent statuses.
    #[index]
    pub created_at: Uint64,
}

/// Inbox of the user in the canister.
/// Stores the activities that the user received from other users.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "inbox"]
pub struct InboxActivity {
    /// Snowflake ID of the activity.
    /// See Snowflake.md for more details.
    #[primary_key]
    pub id: Uint64,
    /// Activity type, such as `Create`, `Follow`, etc.
    #[custom_type]
    pub activity_type: ActivityType,
    /// The actor URI.
    #[validate(UrlValidator)]
    pub actor_uri: Text,
    /// The object data as JSON.
    pub object_data: Json,
    /// Created at timestamp.
    /// Index for efficient retrieval of recent activities.
    #[index]
    pub created_at: Uint64,
}

/// A follower of the user.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "followers"]
pub struct Follower {
    /// The follower's actor URI.
    #[primary_key]
    #[validate(UrlValidator)]
    pub actor_uri: Text,
    /// Created at timestamp.
    pub created_at: Uint64,
}

/// An account the user is following.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "following"]
pub struct Following {
    /// The followed actor's URI.
    #[primary_key]
    #[validate(UrlValidator)]
    pub actor_uri: Text,
    /// Status of the follow request.
    #[custom_type]
    pub status: FollowStatus,
    /// Created at timestamp.
    pub created_at: Uint64,
}

/// Follow requests
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "follow_requests"]
pub struct FollowRequest {
    /// The followed actor's URI.
    #[primary_key]
    #[validate(UrlValidator)]
    pub actor_uri: Text,
    /// Created at timestamp.
    pub created_at: Uint64,
}

#[derive(DatabaseSchema)]
#[tables(
    Settings = "settings",
    Profile = "profiles",
    Status = "statuses",
    InboxActivity = "inbox",
    Follower = "followers",
    Following = "following",
    FollowRequest = "follow_requests"
)]
pub struct Schema;

#[cfg(test)]
mod tests {

    use ic_dbms_canister::prelude::*;
    use wasm_dbms::WasmDbmsDatabase;
    use wasm_dbms_api::prelude::{Database, DeleteBehavior, Filter, Query};

    use super::*;
    use crate::test_utils::{alice, bob, setup};

    // ── Profile ──────────────────────────────────────────────────────

    #[test]
    fn test_should_insert_and_query_profile() {
        setup();

        let principal = Principal(alice());
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Profile>(ProfileInsertRequest {
                principal: principal.clone(),
                handle: "alice".into(),
                display_name: Nullable::Value("Alice".into()),
                bio: Nullable::Value("Hello!".into()),
                avatar_data: Nullable::Null,
                header_data: Nullable::Null,
                created_at: ic_utils::now().into(),
                updated_at: ic_utils::now().into(),
            })
            .expect("should insert profile");

            let rows = db
                .select::<Profile>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "principal",
                            wasm_dbms_api::prelude::Value::from(principal),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select profile");

            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].handle.as_ref().expect("handle").0, "alice");
            assert_eq!(
                rows[0]
                    .display_name
                    .as_ref()
                    .expect("display_name")
                    .clone()
                    .into_opt()
                    .expect("display_name value")
                    .0,
                "Alice"
            );
            assert_eq!(
                rows[0]
                    .bio
                    .as_ref()
                    .expect("bio")
                    .clone()
                    .into_opt()
                    .expect("bio value")
                    .0,
                "Hello!"
            );
        });
    }

    #[test]
    fn test_should_insert_profile_with_null_optional_fields() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Profile>(ProfileInsertRequest {
                principal: Principal(alice()),
                handle: "alice".into(),
                display_name: Nullable::Null,
                bio: Nullable::Null,
                avatar_data: Nullable::Null,
                header_data: Nullable::Null,
                created_at: ic_utils::now().into(),
                updated_at: ic_utils::now().into(),
            })
            .expect("should insert profile with null optional fields");

            let rows = db
                .select::<Profile>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "principal",
                            wasm_dbms_api::prelude::Value::from(Principal(alice())),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select profile");

            assert_eq!(rows.len(), 1);
            assert!(
                rows[0]
                    .display_name
                    .as_ref()
                    .expect("display_name")
                    .is_null(),
                "display_name should be null"
            );
            assert!(
                rows[0].bio.as_ref().expect("bio").is_null(),
                "bio should be null"
            );
        });
    }

    #[test]
    fn test_should_enforce_unique_handle_on_profile() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Profile>(ProfileInsertRequest {
                principal: Principal(alice()),
                handle: "alice".into(),
                display_name: Nullable::Null,
                bio: Nullable::Null,
                avatar_data: Nullable::Null,
                header_data: Nullable::Null,
                created_at: ic_utils::now().into(),
                updated_at: ic_utils::now().into(),
            })
            .expect("should insert first profile");

            let result = db.insert::<Profile>(ProfileInsertRequest {
                principal: Principal(bob()),
                handle: "alice".into(),
                display_name: Nullable::Null,
                bio: Nullable::Null,
                avatar_data: Nullable::Null,
                header_data: Nullable::Null,
                created_at: ic_utils::now().into(),
                updated_at: ic_utils::now().into(),
            });

            assert!(result.is_err(), "duplicate handle should be rejected");
        });
    }

    #[test]
    fn test_should_reject_invalid_handle_on_profile() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<Profile>(ProfileInsertRequest {
                principal: Principal(alice()),
                handle: "INVALID!".into(),
                display_name: Nullable::Null,
                bio: Nullable::Null,
                avatar_data: Nullable::Null,
                header_data: Nullable::Null,
                created_at: ic_utils::now().into(),
                updated_at: ic_utils::now().into(),
            });

            assert!(result.is_err(), "invalid handle should be rejected");
        });
    }

    #[test]
    fn test_should_reject_reserved_handle_on_profile() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<Profile>(ProfileInsertRequest {
                principal: Principal(alice()),
                handle: "admin".into(),
                display_name: Nullable::Null,
                bio: Nullable::Null,
                avatar_data: Nullable::Null,
                header_data: Nullable::Null,
                created_at: ic_utils::now().into(),
                updated_at: ic_utils::now().into(),
            });

            assert!(result.is_err(), "reserved handle should be rejected");
        });
    }

    #[test]
    fn test_should_sanitize_handle_on_profile_insert() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Profile>(ProfileInsertRequest {
                principal: Principal(alice()),
                handle: "  @Alice  ".into(),
                display_name: Nullable::Null,
                bio: Nullable::Null,
                avatar_data: Nullable::Null,
                header_data: Nullable::Null,
                created_at: ic_utils::now().into(),
                updated_at: ic_utils::now().into(),
            })
            .expect("should insert profile with sanitized handle");

            let rows = db
                .select::<Profile>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "handle",
                            wasm_dbms_api::prelude::Value::from("alice".to_string()),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select profile by sanitized handle");

            assert_eq!(rows.len(), 1, "profile should be found by sanitized handle");
        });
    }

    #[test]
    fn test_should_delete_profile() {
        setup();

        let principal = Principal(alice());
        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Profile>(ProfileInsertRequest {
                principal: principal.clone(),
                handle: "alice".into(),
                display_name: Nullable::Null,
                bio: Nullable::Null,
                avatar_data: Nullable::Null,
                header_data: Nullable::Null,
                created_at: ic_utils::now().into(),
                updated_at: ic_utils::now().into(),
            })
            .expect("should insert profile");

            db.delete::<Profile>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    "principal",
                    wasm_dbms_api::prelude::Value::from(principal.clone()),
                )),
            )
            .expect("should delete profile");

            let rows = db
                .select::<Profile>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "principal",
                            wasm_dbms_api::prelude::Value::from(principal),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select profile");

            assert!(rows.is_empty(), "profile should be deleted");
        });
    }

    // ── Status ───────────────────────────────────────────────────────

    #[test]
    fn test_should_insert_and_query_status() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Status>(StatusInsertRequest {
                id: 1u64.into(),
                content: "Hello, world!".into(),
                visibility: Visibility::from(did::common::Visibility::Public),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert status");

            let rows = db
                .select::<Status>(
                    Query::builder()
                        .and_where(Filter::eq("id", wasm_dbms_api::prelude::Value::from(1u64)))
                        .limit(1)
                        .build(),
                )
                .expect("should select status");

            assert_eq!(rows.len(), 1);
            assert_eq!(
                rows[0].content.as_ref().expect("content").0,
                "Hello, world!"
            );
        });
    }

    #[test]
    fn test_should_enforce_unique_status_id() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Status>(StatusInsertRequest {
                id: 1u64.into(),
                content: "First".into(),
                visibility: Visibility::from(did::common::Visibility::Public),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert first status");

            let result = db.insert::<Status>(StatusInsertRequest {
                id: 1u64.into(),
                content: "Duplicate".into(),
                visibility: Visibility::from(did::common::Visibility::Public),
                created_at: ic_utils::now().into(),
            });

            assert!(result.is_err(), "duplicate status id should be rejected");
        });
    }

    #[test]
    fn test_should_delete_status() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Status>(StatusInsertRequest {
                id: 1u64.into(),
                content: "To be deleted".into(),
                visibility: Visibility::from(did::common::Visibility::Public),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert status");

            db.delete::<Status>(
                DeleteBehavior::Restrict,
                Some(Filter::eq("id", wasm_dbms_api::prelude::Value::from(1u64))),
            )
            .expect("should delete status");

            let rows = db
                .select::<Status>(
                    Query::builder()
                        .and_where(Filter::eq("id", wasm_dbms_api::prelude::Value::from(1u64)))
                        .limit(1)
                        .build(),
                )
                .expect("should select status");

            assert!(rows.is_empty(), "status should be deleted");
        });
    }

    #[test]
    fn test_should_insert_status_with_different_visibilities() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            let visibilities = [
                (1u64, did::common::Visibility::Public),
                (2u64, did::common::Visibility::Unlisted),
                (3u64, did::common::Visibility::FollowersOnly),
                (4u64, did::common::Visibility::Direct),
            ];

            for (id, vis) in visibilities {
                db.insert::<Status>(StatusInsertRequest {
                    id: id.into(),
                    content: format!("Status {id}").into(),
                    visibility: Visibility::from(vis),
                    created_at: ic_utils::now().into(),
                })
                .unwrap_or_else(|_| panic!("should insert status {id}"));
            }

            let rows = db
                .select::<Status>(Query::builder().build())
                .expect("should select all statuses");

            assert_eq!(rows.len(), 4);
        });
    }

    // ── InboxActivity ────────────────────────────────────────────────

    #[test]
    fn test_should_insert_and_query_inbox_activity() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<InboxActivity>(InboxActivityInsertRequest {
                id: 100u64.into(),
                activity_type: ActivityType::from(activitypub::ActivityType::Follow),
                actor_uri: "https://example.com/users/bob".into(),
                object_data: serde_json::json!({"type": "Follow"}).into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert inbox activity");

            let rows = db
                .select::<InboxActivity>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "id",
                            wasm_dbms_api::prelude::Value::from(100u64),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select inbox activity");

            assert_eq!(rows.len(), 1);
            assert_eq!(
                rows[0].actor_uri.as_ref().expect("actor_uri").0,
                "https://example.com/users/bob"
            );
        });
    }

    #[test]
    fn test_should_reject_invalid_actor_uri_on_inbox_activity() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<InboxActivity>(InboxActivityInsertRequest {
                id: 100u64.into(),
                activity_type: ActivityType::from(activitypub::ActivityType::Follow),
                actor_uri: "not-a-url".into(),
                object_data: serde_json::json!({"type": "Follow"}).into(),
                created_at: ic_utils::now().into(),
            });

            assert!(result.is_err(), "invalid actor_uri should be rejected");
        });
    }

    #[test]
    fn test_should_delete_inbox_activity() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<InboxActivity>(InboxActivityInsertRequest {
                id: 100u64.into(),
                activity_type: ActivityType::from(activitypub::ActivityType::Create),
                actor_uri: "https://example.com/users/bob".into(),
                object_data: serde_json::json!({"type": "Create"}).into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert inbox activity");

            db.delete::<InboxActivity>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    "id",
                    wasm_dbms_api::prelude::Value::from(100u64),
                )),
            )
            .expect("should delete inbox activity");

            let rows = db
                .select::<InboxActivity>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "id",
                            wasm_dbms_api::prelude::Value::from(100u64),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select inbox activity");

            assert!(rows.is_empty(), "inbox activity should be deleted");
        });
    }

    // ── Follower ─────────────────────────────────────────────────────

    #[test]
    fn test_should_insert_and_query_follower() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Follower>(FollowerInsertRequest {
                actor_uri: "https://example.com/users/bob".into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert follower");

            let rows = db
                .select::<Follower>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "actor_uri",
                            wasm_dbms_api::prelude::Value::from(
                                "https://example.com/users/bob".to_string(),
                            ),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select follower");

            assert_eq!(rows.len(), 1);
        });
    }

    #[test]
    fn test_should_reject_invalid_follower_uri() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<Follower>(FollowerInsertRequest {
                actor_uri: "not-a-url".into(),
                created_at: ic_utils::now().into(),
            });

            assert!(result.is_err(), "invalid actor_uri should be rejected");
        });
    }

    #[test]
    fn test_should_enforce_unique_follower_uri() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Follower>(FollowerInsertRequest {
                actor_uri: "https://example.com/users/bob".into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert first follower");

            let result = db.insert::<Follower>(FollowerInsertRequest {
                actor_uri: "https://example.com/users/bob".into(),
                created_at: ic_utils::now().into(),
            });

            assert!(
                result.is_err(),
                "duplicate follower actor_uri should be rejected"
            );
        });
    }

    #[test]
    fn test_should_delete_follower() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Follower>(FollowerInsertRequest {
                actor_uri: "https://example.com/users/bob".into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert follower");

            db.delete::<Follower>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    "actor_uri",
                    wasm_dbms_api::prelude::Value::from(
                        "https://example.com/users/bob".to_string(),
                    ),
                )),
            )
            .expect("should delete follower");

            let rows = db
                .select::<Follower>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "actor_uri",
                            wasm_dbms_api::prelude::Value::from(
                                "https://example.com/users/bob".to_string(),
                            ),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select follower");

            assert!(rows.is_empty(), "follower should be deleted");
        });
    }

    // ── Following ────────────────────────────────────────────────────

    #[test]
    fn test_should_insert_and_query_following() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Following>(FollowingInsertRequest {
                actor_uri: "https://example.com/users/carol".into(),
                status: FollowStatus::default(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert following");

            let rows = db
                .select::<Following>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "actor_uri",
                            wasm_dbms_api::prelude::Value::from(
                                "https://example.com/users/carol".to_string(),
                            ),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select following");

            assert_eq!(rows.len(), 1);
        });
    }

    #[test]
    fn test_should_reject_invalid_following_uri() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<Following>(FollowingInsertRequest {
                actor_uri: "not-a-url".into(),
                status: FollowStatus::default(),
                created_at: ic_utils::now().into(),
            });

            assert!(result.is_err(), "invalid actor_uri should be rejected");
        });
    }

    #[test]
    fn test_should_enforce_unique_following_uri() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Following>(FollowingInsertRequest {
                actor_uri: "https://example.com/users/carol".into(),
                status: FollowStatus::default(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert first following");

            let result = db.insert::<Following>(FollowingInsertRequest {
                actor_uri: "https://example.com/users/carol".into(),
                status: FollowStatus::Accepted,
                created_at: ic_utils::now().into(),
            });

            assert!(
                result.is_err(),
                "duplicate following actor_uri should be rejected"
            );
        });
    }

    #[test]
    fn test_should_delete_following() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Following>(FollowingInsertRequest {
                actor_uri: "https://example.com/users/carol".into(),
                status: FollowStatus::default(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert following");

            db.delete::<Following>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    "actor_uri",
                    wasm_dbms_api::prelude::Value::from(
                        "https://example.com/users/carol".to_string(),
                    ),
                )),
            )
            .expect("should delete following");

            let rows = db
                .select::<Following>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "actor_uri",
                            wasm_dbms_api::prelude::Value::from(
                                "https://example.com/users/carol".to_string(),
                            ),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select following");

            assert!(rows.is_empty(), "following should be deleted");
        });
    }

    #[test]
    fn test_should_insert_following_with_different_statuses() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);

            let entries = [
                ("https://example.com/users/a", FollowStatus::Pending),
                ("https://example.com/users/b", FollowStatus::Accepted),
                ("https://example.com/users/c", FollowStatus::Rejected),
            ];

            for (uri, status) in entries {
                db.insert::<Following>(FollowingInsertRequest {
                    actor_uri: uri.into(),
                    status,
                    created_at: ic_utils::now().into(),
                })
                .unwrap_or_else(|_| panic!("should insert following {uri}"));
            }

            let rows = db
                .select::<Following>(Query::builder().build())
                .expect("should select all following");

            assert_eq!(rows.len(), 3);
        });
    }
}
