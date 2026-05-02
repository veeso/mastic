//! Database schema for the user canister.

mod activity;
mod feed_source;
mod follow_status;
mod status;
mod visibility;

use db_utils::bounded_text::{BoundedTextValidator, TrimSanitizer};
use db_utils::handle::{HandleSanitizer, HandleValidator};
use db_utils::hashtag::{HashtagSanitizer, HashtagValidator};
use db_utils::media::{BlurhashValidator, MimeValidator};
use db_utils::settings::*;
use db_utils::url::NullableUrlValidator;
use ic_dbms_canister::prelude::Principal;
use wasm_dbms_api::prelude::*;

/// Maximum length of a `spoiler_text` value (shared between `statuses`
/// and `edit_history`). Matches the Mastodon default.
pub const MAX_SPOILER_LENGTH: usize = 500;

/// Maximum length of a media alt-text description. Matches the Mastodon default.
pub const MAX_MEDIA_DESCRIPTION_LENGTH: usize = 1500;

/// Maximum length of a profile metadata field name or value.
pub const MAX_PROFILE_METADATA_LENGTH: usize = 255;

pub use self::activity::ActivityType;
pub use self::feed_source::FeedSource;
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
    /// Cached count of `Like` activities for this status.
    pub like_count: Uint64,
    /// Cached count of `Announce` (boost) activities for this status.
    pub boost_count: Uint64,
    /// URI of the status this one replies to, if any.
    /// Indexed for thread lookups.
    #[index]
    #[validate(NullableUrlValidator)]
    pub in_reply_to_uri: Nullable<Text>,
    /// Optional content warning / spoiler text shown before content.
    #[sanitizer(TrimSanitizer)]
    #[validate(BoundedTextValidator(MAX_SPOILER_LENGTH))]
    pub spoiler_text: Nullable<Text>,
    /// Whether the status should be hidden behind a content warning by clients.
    pub sensitive: Boolean,
    /// Timestamp of the last edit. `Null` when never edited.
    pub edited_at: Nullable<Uint64>,
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
    /// `true` when this inbox entry represents a remote `Announce` (boost)
    /// activity. Paired with `original_status_uri` to locate the boosted status.
    pub is_boost: Boolean,
    /// URI of the boosted status when `is_boost` is `true`.
    #[validate(NullableUrlValidator)]
    pub original_status_uri: Nullable<Text>,
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

/// A denormalized feed entry that indexes both outbox (own statuses) and
/// inbox (received `Create` activities) under a single sorted timeline.
///
/// The `source_id` references the primary key of either the `statuses` or
/// `inbox` table depending on the [`FeedSource`] discriminator.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "feed"]
pub struct FeedEntry {
    /// Snowflake ID shared with the source record.
    #[primary_key]
    pub id: Uint64,
    /// Whether this entry comes from the outbox or inbox.
    #[custom_type]
    pub source: FeedSource,
    /// Created at timestamp, duplicated from the source record for
    /// efficient `ORDER BY` + `LIMIT` + `OFFSET` without joining.
    #[index]
    pub created_at: Uint64,
}

/// A status the user has liked.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "liked"]
pub struct Liked {
    /// URI of the liked status.
    #[primary_key]
    #[validate(UrlValidator)]
    pub status_uri: Text,
    /// Created at timestamp.
    pub created_at: Uint64,
}

/// A remote actor blocked by the user.
///
/// Blocked actors cannot see the user's statuses and the user does
/// not receive any activity from them.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "blocks"]
pub struct Block {
    /// URI of the blocked actor.
    #[primary_key]
    #[validate(UrlValidator)]
    pub actor_uri: Text,
    /// Created at timestamp.
    pub created_at: Uint64,
}

/// A remote actor muted by the user.
///
/// Muted actors do not appear in the user's timeline but may still
/// follow and interact with the account.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "mutes"]
pub struct Mute {
    /// URI of the muted actor.
    #[primary_key]
    #[validate(UrlValidator)]
    pub actor_uri: Text,
    /// Created at timestamp.
    pub created_at: Uint64,
}

/// A status bookmarked by the user.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "bookmarks"]
pub struct Bookmark {
    /// URI of the bookmarked status.
    #[primary_key]
    #[validate(UrlValidator)]
    pub status_uri: Text,
    /// Created at timestamp.
    pub created_at: Uint64,
}

/// A boost (`Announce`) emitted by the user.
///
/// Each boost is linked to the user's own wrapper status row in the
/// `statuses` table via `status_id`. `original_status_uri` points to
/// the boosted remote status.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "boosts"]
pub struct Boost {
    /// Snowflake ID of the boost.
    #[primary_key]
    pub id: Uint64,
    /// Foreign key to the owner's wrapper status row.
    #[foreign_key(entity = "Status", table = "statuses", column = "id")]
    pub status_id: Uint64,
    /// URI of the boosted status.
    #[validate(UrlValidator)]
    pub original_status_uri: Text,
    /// Created at timestamp.
    /// Indexed for efficient retrieval of recent boosts.
    #[index]
    pub created_at: Uint64,
}

/// Media attachment associated with a status.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "media"]
pub struct Media {
    /// Snowflake ID of the media attachment.
    #[primary_key]
    pub id: Uint64,
    /// Foreign key to the parent status.
    #[index]
    #[foreign_key(entity = "Status", table = "statuses", column = "id")]
    pub status_id: Uint64,
    /// MIME-like media type (e.g. `image/png`).
    #[validate(MimeValidator)]
    pub media_type: Text,
    /// Alt-text description of the media.
    #[sanitizer(TrimSanitizer)]
    #[validate(BoundedTextValidator(MAX_MEDIA_DESCRIPTION_LENGTH))]
    pub description: Nullable<Text>,
    /// Blurhash preview for the media.
    #[validate(BlurhashValidator)]
    pub blurhash: Nullable<Text>,
    /// Raw media bytes.
    pub bytes: Blob,
    /// Created at timestamp.
    pub created_at: Uint64,
}

/// Previous versions of a status, appended on every edit.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "edit_history"]
pub struct EditHistory {
    /// Snowflake ID of the history entry.
    #[primary_key]
    pub id: Uint64,
    /// Foreign key to the status the entry belongs to.
    #[index]
    #[foreign_key(entity = "Status", table = "statuses", column = "id")]
    pub status_id: Uint64,
    /// Content of the status before the edit.
    pub previous_content: Text,
    /// Spoiler text of the status before the edit.
    #[sanitizer(TrimSanitizer)]
    #[validate(BoundedTextValidator(MAX_SPOILER_LENGTH))]
    pub previous_spoiler_text: Nullable<Text>,
    /// Timestamp of the edit, indexed for ordered retrieval.
    #[index]
    pub edited_at: Uint64,
}

/// Hashtags referenced by the user's own statuses (local index).
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "hashtags"]
pub struct Hashtag {
    /// Snowflake ID of the hashtag row.
    #[primary_key]
    pub id: Uint64,
    /// Sanitized, lowercase tag (without leading `#`).
    #[unique]
    #[sanitizer(HashtagSanitizer)]
    #[validate(HashtagValidator)]
    pub tag: Text,
    /// Created at timestamp.
    pub created_at: Uint64,
}

/// Join table between statuses and hashtags.
///
/// A surrogate `id` is used as primary key because the underlying
/// storage layer does not support composite primary keys; application
/// logic ensures uniqueness of the `(status_id, hashtag_id)` pair.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "status_hashtags"]
pub struct StatusHashtag {
    /// Snowflake ID of the join row.
    #[primary_key]
    pub id: Uint64,
    /// Foreign key to the status.
    #[index]
    #[foreign_key(entity = "Status", table = "statuses", column = "id")]
    pub status_id: Uint64,
    /// Foreign key to the hashtag.
    #[index]
    #[foreign_key(entity = "Hashtag", table = "hashtags", column = "id")]
    pub hashtag_id: Uint64,
}

/// Hashtags featured on the user's profile (max 4).
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "featured_tags"]
pub struct FeaturedTag {
    /// Sanitized, lowercase tag.
    #[primary_key]
    #[sanitizer(HashtagSanitizer)]
    #[validate(HashtagValidator)]
    pub tag: Text,
    /// Display position within the featured list (`0`..=`3`).
    #[unique]
    pub position: Uint8,
    /// Created at timestamp.
    pub created_at: Uint64,
}

/// Statuses pinned on the user's profile (max 5).
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "pinned_statuses"]
pub struct PinnedStatus {
    /// Foreign key to the pinned status.
    #[primary_key]
    #[foreign_key(entity = "Status", table = "statuses", column = "id")]
    pub status_id: Uint64,
    /// Display position (`0`..=`4`).
    #[unique]
    pub position: Uint8,
    /// Timestamp when the status was pinned.
    pub pinned_at: Uint64,
}

/// Custom profile metadata rows (max 4).
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "profile_metadata"]
pub struct ProfileMetadata {
    /// Position within the metadata list (`0`..=`3`).
    #[primary_key]
    pub position: Uint8,
    /// Field name.
    #[sanitizer(TrimSanitizer)]
    #[validate(BoundedTextValidator(MAX_PROFILE_METADATA_LENGTH))]
    pub name: Text,
    /// Field value.
    #[sanitizer(TrimSanitizer)]
    #[validate(BoundedTextValidator(MAX_PROFILE_METADATA_LENGTH))]
    pub value: Text,
}

#[derive(DatabaseSchema, Clone, Copy)]
#[tables(
    Settings = "settings",
    Profile = "profiles",
    Status = "statuses",
    InboxActivity = "inbox",
    Follower = "followers",
    Following = "following",
    FollowRequest = "follow_requests",
    FeedEntry = "feed",
    Liked = "liked",
    Block = "blocks",
    Mute = "mutes",
    Bookmark = "bookmarks",
    Boost = "boosts",
    Media = "media",
    EditHistory = "edit_history",
    Hashtag = "hashtags",
    StatusHashtag = "status_hashtags",
    FeaturedTag = "featured_tags",
    PinnedStatus = "pinned_statuses",
    ProfileMetadata = "profile_metadata"
)]
pub struct Schema;

#[cfg(test)]
mod tests {

    use ic_dbms_canister::prelude::*;
    use wasm_dbms::WasmDbmsDatabase;
    use wasm_dbms_api::prelude::{Database, DeleteBehavior, Filter, Query};

    use super::*;
    use crate::test_utils::{alice, bob, setup};

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

    #[test]
    fn test_should_insert_and_query_status() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Status>(StatusInsertRequest {
                id: 1u64.into(),
                content: "Hello, world!".into(),
                visibility: Visibility::from(did::common::Visibility::Public),
                like_count: 0u64.into(),
                boost_count: 0u64.into(),
                in_reply_to_uri: Nullable::Null,
                spoiler_text: Nullable::Null,
                sensitive: false.into(),
                edited_at: Nullable::Null,
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
                like_count: 0u64.into(),
                boost_count: 0u64.into(),
                in_reply_to_uri: Nullable::Null,
                spoiler_text: Nullable::Null,
                sensitive: false.into(),
                edited_at: Nullable::Null,
                created_at: ic_utils::now().into(),
            })
            .expect("should insert first status");

            let result = db.insert::<Status>(StatusInsertRequest {
                id: 1u64.into(),
                content: "Duplicate".into(),
                visibility: Visibility::from(did::common::Visibility::Public),
                like_count: 0u64.into(),
                boost_count: 0u64.into(),
                in_reply_to_uri: Nullable::Null,
                spoiler_text: Nullable::Null,
                sensitive: false.into(),
                edited_at: Nullable::Null,
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
                like_count: 0u64.into(),
                boost_count: 0u64.into(),
                in_reply_to_uri: Nullable::Null,
                spoiler_text: Nullable::Null,
                sensitive: false.into(),
                edited_at: Nullable::Null,
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
                    like_count: 0u64.into(),
                    boost_count: 0u64.into(),
                    in_reply_to_uri: Nullable::Null,
                    spoiler_text: Nullable::Null,
                    sensitive: false.into(),
                    edited_at: Nullable::Null,
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
                is_boost: false.into(),
                original_status_uri: Nullable::Null,
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
                is_boost: false.into(),
                original_status_uri: Nullable::Null,
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
                is_boost: false.into(),
                original_status_uri: Nullable::Null,
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

            assert_eq!(rows.len(), 2);
        });
    }

    #[test]
    fn test_should_insert_and_query_feed_entry() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<FeedEntry>(FeedEntryInsertRequest {
                id: 1u64.into(),
                source: FeedSource::Outbox,
                created_at: ic_utils::now().into(),
            })
            .expect("should insert feed entry");

            let rows = db
                .select::<FeedEntry>(Query::builder().build())
                .expect("should select feed entries");

            assert_eq!(rows.len(), 1);
        });
    }

    #[test]
    fn test_should_insert_multiple_feed_entries() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);

            db.insert::<FeedEntry>(FeedEntryInsertRequest {
                id: 1u64.into(),
                source: FeedSource::Outbox,
                created_at: 1000u64.into(),
            })
            .expect("should insert first feed entry");

            db.insert::<FeedEntry>(FeedEntryInsertRequest {
                id: 2u64.into(),
                source: FeedSource::Inbox,
                created_at: 2000u64.into(),
            })
            .expect("should insert second feed entry");

            db.commit().expect("should commit");
        });

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let rows = db
                .select::<FeedEntry>(Query::builder().build())
                .expect("should select feed entries");

            assert_eq!(rows.len(), 2);
        });
    }

    fn insert_status_row<M, A>(db: &mut WasmDbmsDatabase<'_, M, A>, id: u64)
    where
        M: wasm_dbms_memory::MemoryProvider,
        A: wasm_dbms_memory::AccessControl,
    {
        db.insert::<Status>(StatusInsertRequest {
            id: id.into(),
            content: format!("Status {id}").into(),
            visibility: Visibility::from(did::common::Visibility::Public),
            like_count: 0u64.into(),
            boost_count: 0u64.into(),
            in_reply_to_uri: Nullable::Null,
            spoiler_text: Nullable::Null,
            sensitive: false.into(),
            edited_at: Nullable::Null,
            created_at: ic_utils::now().into(),
        })
        .expect("should insert status");
    }

    #[test]
    fn test_should_insert_and_delete_liked() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Liked>(LikedInsertRequest {
                status_uri: "https://example.com/statuses/1".into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert liked");

            let rows = db
                .select::<Liked>(Query::builder().build())
                .expect("should select liked");
            assert_eq!(rows.len(), 1);

            db.delete::<Liked>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    "status_uri",
                    wasm_dbms_api::prelude::Value::from(
                        "https://example.com/statuses/1".to_string(),
                    ),
                )),
            )
            .expect("should delete liked");

            let rows = db
                .select::<Liked>(Query::builder().build())
                .expect("should select liked");
            assert!(rows.is_empty());
        });
    }

    #[test]
    fn test_should_reject_invalid_liked_uri() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<Liked>(LikedInsertRequest {
                status_uri: "not-a-url".into(),
                created_at: ic_utils::now().into(),
            });
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_should_insert_and_delete_block() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Block>(BlockInsertRequest {
                actor_uri: "https://example.com/users/bob".into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert block");

            let rows = db
                .select::<Block>(Query::builder().build())
                .expect("should select block");
            assert_eq!(rows.len(), 1);

            db.delete::<Block>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    "actor_uri",
                    wasm_dbms_api::prelude::Value::from(
                        "https://example.com/users/bob".to_string(),
                    ),
                )),
            )
            .expect("should delete block");

            let rows = db
                .select::<Block>(Query::builder().build())
                .expect("should select block");
            assert!(rows.is_empty());
        });
    }

    #[test]
    fn test_should_insert_and_delete_mute() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Mute>(MuteInsertRequest {
                actor_uri: "https://example.com/users/bob".into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert mute");

            let rows = db
                .select::<Mute>(Query::builder().build())
                .expect("should select mute");
            assert_eq!(rows.len(), 1);

            db.delete::<Mute>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    "actor_uri",
                    wasm_dbms_api::prelude::Value::from(
                        "https://example.com/users/bob".to_string(),
                    ),
                )),
            )
            .expect("should delete mute");

            let rows = db
                .select::<Mute>(Query::builder().build())
                .expect("should select mute");
            assert!(rows.is_empty());
        });
    }

    #[test]
    fn test_should_insert_and_delete_bookmark() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Bookmark>(BookmarkInsertRequest {
                status_uri: "https://example.com/statuses/1".into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert bookmark");

            let rows = db
                .select::<Bookmark>(Query::builder().build())
                .expect("should select bookmark");
            assert_eq!(rows.len(), 1);

            db.delete::<Bookmark>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    "status_uri",
                    wasm_dbms_api::prelude::Value::from(
                        "https://example.com/statuses/1".to_string(),
                    ),
                )),
            )
            .expect("should delete bookmark");

            let rows = db
                .select::<Bookmark>(Query::builder().build())
                .expect("should select bookmark");
            assert!(rows.is_empty());
        });
    }

    #[test]
    fn test_should_insert_and_delete_boost() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);
            insert_status_row(&mut db, 10);

            db.insert::<Boost>(BoostInsertRequest {
                id: 100u64.into(),
                status_id: 10u64.into(),
                original_status_uri: "https://example.com/statuses/99".into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert boost");
            db.commit().expect("should commit");
        });

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let rows = db
                .select::<Boost>(Query::builder().build())
                .expect("should select boost");
            assert_eq!(rows.len(), 1);

            db.delete::<Boost>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    "id",
                    wasm_dbms_api::prelude::Value::from(100u64),
                )),
            )
            .expect("should delete boost");

            let rows = db
                .select::<Boost>(Query::builder().build())
                .expect("should select boost");
            assert!(rows.is_empty());
        });
    }

    #[test]
    fn test_should_insert_and_delete_media() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);
            insert_status_row(&mut db, 20);

            db.insert::<Media>(MediaInsertRequest {
                id: 200u64.into(),
                status_id: 20u64.into(),
                media_type: "image/png".into(),
                description: Nullable::Value("alt".into()),
                blurhash: Nullable::Null,
                bytes: vec![1u8, 2, 3].into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert media");
            db.commit().expect("should commit");
        });

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let rows = db
                .select::<Media>(Query::builder().build())
                .expect("should select media");
            assert_eq!(rows.len(), 1);

            db.delete::<Media>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    "id",
                    wasm_dbms_api::prelude::Value::from(200u64),
                )),
            )
            .expect("should delete media");
        });
    }

    #[test]
    fn test_should_insert_and_delete_edit_history() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);
            insert_status_row(&mut db, 30);

            db.insert::<EditHistory>(EditHistoryInsertRequest {
                id: 300u64.into(),
                status_id: 30u64.into(),
                previous_content: "old".into(),
                previous_spoiler_text: Nullable::Null,
                edited_at: ic_utils::now().into(),
            })
            .expect("should insert edit history");
            db.commit().expect("should commit");
        });

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let rows = db
                .select::<EditHistory>(Query::builder().build())
                .expect("should select edit history");
            assert_eq!(rows.len(), 1);
        });
    }

    #[test]
    fn test_should_insert_hashtag_and_join() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);
            insert_status_row(&mut db, 40);

            db.insert::<Hashtag>(HashtagInsertRequest {
                id: 400u64.into(),
                tag: "rust".into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert hashtag");

            db.insert::<StatusHashtag>(StatusHashtagInsertRequest {
                id: 401u64.into(),
                status_id: 40u64.into(),
                hashtag_id: 400u64.into(),
            })
            .expect("should insert status_hashtag");
            db.commit().expect("should commit");
        });

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            assert_eq!(
                db.select::<Hashtag>(Query::builder().build())
                    .expect("select hashtags")
                    .len(),
                1
            );
            assert_eq!(
                db.select::<StatusHashtag>(Query::builder().build())
                    .expect("select status_hashtags")
                    .len(),
                1
            );
        });
    }

    #[test]
    fn test_should_sanitize_hashtag_tag_on_insert() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Hashtag>(HashtagInsertRequest {
                id: 1u64.into(),
                tag: "  #Rust  ".into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert hashtag with sanitized tag");

            let rows = db
                .select::<Hashtag>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "tag",
                            wasm_dbms_api::prelude::Value::from("rust".to_string()),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select hashtag by sanitized tag");

            assert_eq!(rows.len(), 1);
        });
    }

    #[test]
    fn test_should_reject_invalid_hashtag_tag() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<Hashtag>(HashtagInsertRequest {
                id: 1u64.into(),
                tag: "rust-lang".into(),
                created_at: ic_utils::now().into(),
            });
            assert!(result.is_err(), "invalid hashtag tag should be rejected");
        });
    }

    #[test]
    fn test_should_sanitize_featured_tag_on_insert() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<FeaturedTag>(FeaturedTagInsertRequest {
                tag: "  #Rust  ".into(),
                position: 0u8.into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert featured tag with sanitized tag");

            let rows = db
                .select::<FeaturedTag>(
                    Query::builder()
                        .and_where(Filter::eq(
                            "tag",
                            wasm_dbms_api::prelude::Value::from("rust".to_string()),
                        ))
                        .limit(1)
                        .build(),
                )
                .expect("should select featured tag by sanitized tag");

            assert_eq!(rows.len(), 1);
        });
    }

    #[test]
    fn test_should_reject_invalid_featured_tag() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<FeaturedTag>(FeaturedTagInsertRequest {
                tag: "rust-lang".into(),
                position: 0u8.into(),
                created_at: ic_utils::now().into(),
            });
            assert!(result.is_err(), "invalid featured tag should be rejected");
        });
    }

    #[test]
    fn test_should_enforce_unique_hashtag_tag() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Hashtag>(HashtagInsertRequest {
                id: 1u64.into(),
                tag: "rust".into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert first hashtag");

            let result = db.insert::<Hashtag>(HashtagInsertRequest {
                id: 2u64.into(),
                tag: "rust".into(),
                created_at: ic_utils::now().into(),
            });
            assert!(result.is_err(), "duplicate hashtag tag rejected");
        });
    }

    #[test]
    fn test_should_insert_and_delete_featured_tag() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<FeaturedTag>(FeaturedTagInsertRequest {
                tag: "rust".into(),
                position: 0u8.into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert featured tag");

            let rows = db
                .select::<FeaturedTag>(Query::builder().build())
                .expect("should select featured tags");
            assert_eq!(rows.len(), 1);

            db.delete::<FeaturedTag>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    "tag",
                    wasm_dbms_api::prelude::Value::from("rust".to_string()),
                )),
            )
            .expect("should delete featured tag");
        });
    }

    #[test]
    fn test_should_enforce_unique_featured_tag_position() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<FeaturedTag>(FeaturedTagInsertRequest {
                tag: "rust".into(),
                position: 0u8.into(),
                created_at: ic_utils::now().into(),
            })
            .expect("should insert first featured tag");

            let result = db.insert::<FeaturedTag>(FeaturedTagInsertRequest {
                tag: "ic".into(),
                position: 0u8.into(),
                created_at: ic_utils::now().into(),
            });
            assert!(result.is_err(), "duplicate featured tag position rejected");
        });
    }

    #[test]
    fn test_should_insert_pinned_status() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);
            insert_status_row(&mut db, 50);

            db.insert::<PinnedStatus>(PinnedStatusInsertRequest {
                status_id: 50u64.into(),
                position: 0u8.into(),
                pinned_at: ic_utils::now().into(),
            })
            .expect("should insert pinned status");
            db.commit().expect("should commit");
        });

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let rows = db
                .select::<PinnedStatus>(Query::builder().build())
                .expect("should select pinned statuses");
            assert_eq!(rows.len(), 1);
        });
    }

    #[test]
    fn test_should_reject_spoiler_text_over_limit() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<Status>(StatusInsertRequest {
                id: 99u64.into(),
                content: "hi".into(),
                visibility: Visibility::from(did::common::Visibility::Public),
                like_count: 0u64.into(),
                boost_count: 0u64.into(),
                in_reply_to_uri: Nullable::Null,
                spoiler_text: Nullable::Value("a".repeat(MAX_SPOILER_LENGTH + 1).into()),
                sensitive: false.into(),
                edited_at: Nullable::Null,
                created_at: ic_utils::now().into(),
            });
            assert!(result.is_err(), "spoiler over limit should be rejected");
        });
    }

    #[test]
    fn test_should_trim_spoiler_text() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<Status>(StatusInsertRequest {
                id: 98u64.into(),
                content: "hi".into(),
                visibility: Visibility::from(did::common::Visibility::Public),
                like_count: 0u64.into(),
                boost_count: 0u64.into(),
                in_reply_to_uri: Nullable::Null,
                spoiler_text: Nullable::Value("  spoiler  ".into()),
                sensitive: false.into(),
                edited_at: Nullable::Null,
                created_at: ic_utils::now().into(),
            })
            .expect("should insert with trimmed spoiler");

            let rows = db
                .select::<Status>(
                    Query::builder()
                        .and_where(Filter::eq("id", wasm_dbms_api::prelude::Value::from(98u64)))
                        .limit(1)
                        .build(),
                )
                .expect("should select status");
            let spoiler = rows[0]
                .spoiler_text
                .as_ref()
                .expect("spoiler_text")
                .clone()
                .into_opt()
                .expect("spoiler value");
            assert_eq!(spoiler.0, "spoiler");
        });
    }

    #[test]
    fn test_should_reject_invalid_in_reply_to_uri() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<Status>(StatusInsertRequest {
                id: 97u64.into(),
                content: "hi".into(),
                visibility: Visibility::from(did::common::Visibility::Public),
                like_count: 0u64.into(),
                boost_count: 0u64.into(),
                in_reply_to_uri: Nullable::Value("not-a-url".into()),
                spoiler_text: Nullable::Null,
                sensitive: false.into(),
                edited_at: Nullable::Null,
                created_at: ic_utils::now().into(),
            });
            assert!(
                result.is_err(),
                "invalid in_reply_to_uri should be rejected"
            );
        });
    }

    #[test]
    fn test_should_reject_invalid_inbox_original_status_uri() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<InboxActivity>(InboxActivityInsertRequest {
                id: 500u64.into(),
                activity_type: ActivityType::from(activitypub::ActivityType::Announce),
                actor_uri: "https://example.com/users/bob".into(),
                object_data: serde_json::json!({"type": "Announce"}).into(),
                is_boost: true.into(),
                original_status_uri: Nullable::Value("not-a-url".into()),
                created_at: ic_utils::now().into(),
            });
            assert!(
                result.is_err(),
                "invalid original_status_uri should be rejected"
            );
        });
    }

    #[test]
    fn test_should_reject_invalid_media_type() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);
            insert_status_row(&mut db, 21);

            let result = db.insert::<Media>(MediaInsertRequest {
                id: 210u64.into(),
                status_id: 21u64.into(),
                media_type: "imagepng".into(),
                description: Nullable::Null,
                blurhash: Nullable::Null,
                bytes: vec![].into(),
                created_at: ic_utils::now().into(),
            });
            assert!(result.is_err(), "invalid media_type should be rejected");
        });
    }

    #[test]
    fn test_should_reject_invalid_blurhash() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);
            insert_status_row(&mut db, 22);

            let result = db.insert::<Media>(MediaInsertRequest {
                id: 220u64.into(),
                status_id: 22u64.into(),
                media_type: "image/png".into(),
                description: Nullable::Null,
                blurhash: Nullable::Value("abc".into()),
                bytes: vec![].into(),
                created_at: ic_utils::now().into(),
            });
            assert!(result.is_err(), "blurhash too short should be rejected");
        });
    }

    #[test]
    fn test_should_reject_profile_metadata_over_limit() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            let result = db.insert::<ProfileMetadata>(ProfileMetadataInsertRequest {
                position: 0u8.into(),
                name: "a".repeat(MAX_PROFILE_METADATA_LENGTH + 1).into(),
                value: "v".into(),
            });
            assert!(result.is_err(), "metadata name over limit rejected");
        });
    }

    #[test]
    fn test_should_insert_and_update_profile_metadata() {
        setup();

        DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, Schema);
            db.insert::<ProfileMetadata>(ProfileMetadataInsertRequest {
                position: 0u8.into(),
                name: "Website".into(),
                value: "https://mastic.social".into(),
            })
            .expect("should insert profile metadata");

            let rows = db
                .select::<ProfileMetadata>(Query::builder().build())
                .expect("should select profile metadata");
            assert_eq!(rows.len(), 1);

            db.delete::<ProfileMetadata>(
                DeleteBehavior::Restrict,
                Some(Filter::eq(
                    "position",
                    wasm_dbms_api::prelude::Value::from(0u8),
                )),
            )
            .expect("should delete profile metadata");
        });
    }
}
