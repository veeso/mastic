//! Read feed flow logic

use activitypub::Activity;
use activitypub::activity::{ActivityObject, ActivityType as ApActivityType};
use activitypub::object::{BaseObject, ObjectType, OneOrMany};
use did::common::{FeedItem, Status, Visibility};
use did::user::{ReadFeedArgs, ReadFeedError, ReadFeedResponse};
use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms::WasmDbmsDatabase;
use wasm_dbms_api::prelude::{ColumnDef, Database, Filter, Query, Value};

use crate::domain::boost::BoostRepository;
use crate::domain::liked::LikedRepository;
use crate::error::CanisterResult;
use crate::schema::{FeedSource, Schema, Visibility as DbVisibility};

/// The ActivityStreams public addressing constant.
const AS_PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";

/// Tuple of fields extracted from a `Create(Note)` activity:
/// `(content, visibility, spoiler_text, sensitive, note_id)`.
type ExtractedNote = (String, Visibility, Option<String>, bool, Option<String>);

/// Reads the user's feed, which includes status updates from followed users.
///
/// The user's feed is read from the denormalized `feed` table which indexes
/// both outbox and inbox entries under a single sorted timeline.
pub fn read_feed(ReadFeedArgs { limit, offset }: ReadFeedArgs) -> ReadFeedResponse {
    ic_utils::log!("Reading user's feed with limit {limit} and offset {offset}");

    if limit > crate::domain::MAX_PAGE_LIMIT {
        ic_utils::log!(
            "Requested feed page limit {limit} exceeds maximum of {MAX}",
            MAX = crate::domain::MAX_PAGE_LIMIT
        );
        return ReadFeedResponse::Err(ReadFeedError::LimitExceeded);
    }

    match read_feed_inner(limit, offset) {
        Ok(items) => ReadFeedResponse::Ok(items),
        Err(e) => {
            ic_utils::log!("Error reading feed: {e}");
            ReadFeedResponse::Err(ReadFeedError::Internal(e.to_string()))
        }
    }
}

/// Internal helper that queries the aggregated `feed` table and hydrates
/// each entry into a [`FeedItem`] by joining back to `statuses` or `inbox`.
///
/// Sorting, offset and limit are fully handled at the database level,
/// keeping memory usage bounded regardless of feed size.
fn read_feed_inner(limit: u64, offset: u64) -> CanisterResult<Vec<FeedItem>> {
    let own_profile = crate::domain::profile::ProfileRepository::get_profile()?;
    let owner_actor_uri = crate::domain::urls::actor_uri(&own_profile.handle.0)?;

    DBMS_CONTEXT.with(|ctx| {
        let db = WasmDbmsDatabase::oneshot(ctx, Schema);

        // Single query on the aggregated feed table
        let feed_query = Query::builder()
            .all()
            .order_by_desc("created_at")
            .limit(limit as usize)
            .offset(offset as usize)
            .build();
        let feed_rows = db.select_raw("feed", feed_query)?;

        let mut items = Vec::with_capacity(feed_rows.len());

        for row in &feed_rows {
            let Some(id) = find_value(row, "id")
                .and_then(|v| v.as_uint64())
                .map(|u| u.0)
            else {
                continue;
            };
            let Some(source) =
                find_value(row, "source").and_then(|v| v.as_custom_type::<FeedSource>())
            else {
                continue;
            };

            let item = match source {
                FeedSource::Outbox => hydrate_outbox(&db, id, &owner_actor_uri),
                FeedSource::Inbox => hydrate_inbox(&db, id, &owner_actor_uri),
            };

            if let Some(feed_item) = item {
                items.push(feed_item);
            }
        }

        Ok(items)
    })
}

/// Loads a status from the `statuses` table and wraps it as a [`FeedItem`].
fn hydrate_outbox(db: &impl Database, id: u64, owner_actor_uri: &str) -> Option<FeedItem> {
    let query = Query::builder()
        .all()
        .and_where(Filter::eq("id", Value::from(id)))
        .limit(1)
        .build();
    let rows = db.select_raw("statuses", query).ok()?;
    let row = rows.first()?;

    let content = find_value(row, "content")?.as_text()?.0.clone();
    let db_vis: DbVisibility = find_value(row, "visibility")?.as_custom_type()?;
    let created_at = find_value(row, "created_at")?.as_uint64()?.0;

    let like_count = find_value(row, "like_count")?.as_uint64()?.0;
    let boost_count = find_value(row, "boost_count")?.as_uint64()?.0;
    let spoiler_text = find_value(row, "spoiler_text")
        .and_then(|v| v.as_text())
        .map(|t| t.0.clone());
    let sensitive = find_value(row, "sensitive")?.as_boolean()?.0;

    // If a `Boost` row exists for this status id, this is a wrapper status for
    // a re-share: rewrite id/author from the boosted original and surface the
    // owner as `boosted_by`.
    let boost_row = db
        .select::<crate::schema::Boost>(
            Query::builder()
                .all()
                .and_where(Filter::eq("status_id", Value::from(id)))
                .limit(1)
                .build(),
        )
        .ok()
        .and_then(|rows| rows.into_iter().next());

    let (final_id, author, boosted_by, lookup_uri) = match boost_row {
        Some(row) => {
            let original_uri = row
                .original_status_uri
                .expect("boosts.original_status_uri is non-null")
                .0;
            let original_id = crate::domain::urls::parse_status_id(&original_uri).unwrap_or(id);
            let author = crate::domain::urls::actor_uri_from_status_uri(&original_uri)
                .unwrap_or_else(|| owner_actor_uri.to_string());
            (
                original_id,
                author,
                Some(owner_actor_uri.to_string()),
                original_uri,
            )
        }
        None => (
            id,
            owner_actor_uri.to_string(),
            None,
            format!("{owner_actor_uri}/statuses/{id}"),
        ),
    };

    let (liked, boosted) = viewer_flags(&lookup_uri);

    Some(FeedItem {
        status: Status {
            id: final_id,
            content,
            author,
            created_at,
            visibility: db_vis.into(),
            like_count,
            boost_count,
            spoiler_text,
            sensitive,
        },
        boosted_by,
        liked,
        boosted,
    })
}

/// Loads an inbox activity and extracts its `Create(Note)` content as a [`FeedItem`].
///
/// Visibility filtering:
/// - `Public` / `Unlisted` / `FollowersOnly`: always shown (the user is a
///   follower by definition since the item landed in their inbox).
/// - `Direct`: only shown if the owner's actor URI appears in `to` or `cc`.
fn hydrate_inbox(db: &impl Database, id: u64, owner_actor_uri: &str) -> Option<FeedItem> {
    let query = Query::builder()
        .all()
        .and_where(Filter::eq("id", Value::from(id)))
        .limit(1)
        .build();
    let rows = db.select_raw("inbox", query).ok()?;
    let row = rows.first()?;

    let created_at = find_value(row, "created_at")?.as_uint64()?.0;
    let is_boost = find_value(row, "is_boost")
        .and_then(|v| v.as_boolean())
        .map(|b| b.0)
        .unwrap_or(false);

    if is_boost {
        let original_uri = find_value(row, "original_status_uri")?.as_text()?.0.clone();
        let booster = find_value(row, "actor_uri")?.as_text()?.0.clone();
        let original_id = crate::domain::urls::parse_status_id(&original_uri).unwrap_or(0);
        let author =
            crate::domain::urls::actor_uri_from_status_uri(&original_uri).unwrap_or_default();

        let (content, visibility, spoiler_text, sensitive) =
            resolve_boost_original(db, &original_uri, original_id);

        let (liked, boosted) = viewer_flags(&original_uri);

        return Some(FeedItem {
            status: Status {
                id: original_id,
                content,
                author,
                created_at,
                visibility,
                like_count: 0,
                boost_count: 0,
                spoiler_text,
                sensitive,
            },
            boosted_by: Some(booster),
            liked,
            boosted,
        });
    }

    let json_val = find_value(row, "object_data")?.as_json()?;
    let activity: Activity = serde_json::from_value(json_val.value().clone()).ok()?;
    let (content, visibility, spoiler_text, sensitive, note_id) =
        extract_note_from_activity(&activity)?;

    // Direct messages must only appear when the owner is explicitly addressed.
    if visibility == Visibility::Direct && !is_addressed_to(&activity.base, owner_actor_uri) {
        return None;
    }

    let author_uri = activity.actor.clone().unwrap_or_default();
    let lookup_uri = note_id.and_then(|nid| canonical_status_uri(&nid, &author_uri));
    let (liked, boosted) = lookup_uri
        .as_deref()
        .map(viewer_flags)
        .unwrap_or((false, false));

    Some(FeedItem {
        status: Status {
            id,
            content,
            author: author_uri,
            created_at,
            visibility,
            like_count: 0,
            boost_count: 0,
            spoiler_text,
            sensitive,
        },
        boosted_by: None,
        liked,
        boosted,
    })
}

/// Returns `(liked, boosted)` for the given status URI from the viewer's
/// own `liked` and `boosts` tables. Lookup failures degrade to `false` so
/// hydration never blocks the feed render.
fn viewer_flags(status_uri: &str) -> (bool, bool) {
    let liked = LikedRepository::oneshot()
        .is_liked(status_uri)
        .unwrap_or(false);
    let boosted = BoostRepository::is_boosted(status_uri).unwrap_or(false);
    (liked, boosted)
}

/// Resolve a note's `id` field into the canonical status URI used by the
/// `liked` and `boosts` tables. When the note id is already an absolute
/// URI it is returned unchanged; when it is a bare snowflake (as emitted
/// today by [`make_activity`] in the publish pipeline) it is rebuilt as
/// `{actor}/statuses/{id}`. Returns [`None`] when no actor is available
/// to anchor a relative id.
fn canonical_status_uri(note_id: &str, author_uri: &str) -> Option<String> {
    if note_id.starts_with("http://") || note_id.starts_with("https://") {
        Some(note_id.to_string())
    } else if author_uri.is_empty() {
        None
    } else {
        Some(format!("{author_uri}/statuses/{note_id}"))
    }
}

/// Resolve the original (boosted) status for an inbox boost render.
///
/// Tries:
/// 1. Local statuses table (if the URI is hosted on this instance and the
///    `find_by_id` lookup succeeds).
/// 2. Cached inbox `Create(Note)` row whose `object_data.object.id` matches
///    `original_uri`.
/// 3. Fallback: empty content, `Visibility::Public`, no spoiler, not sensitive.
fn resolve_boost_original(
    db: &impl Database,
    original_uri: &str,
    original_id: u64,
) -> (String, Visibility, Option<String>, bool) {
    if let Ok(Some((_, _))) = crate::domain::urls::parse_local_status_uri(original_uri)
        && let Ok(Some(row)) = crate::domain::status::StatusRepository::find_by_id(original_id)
    {
        let content = row.content.0.to_string();
        let visibility: Visibility = row.visibility.into();
        let spoiler_text = row.spoiler_text.into_opt().map(|t| t.0);
        let sensitive = row.sensitive.0;
        return (content, visibility, spoiler_text, sensitive);
    }

    if let Some((content, visibility, spoiler_text, sensitive, _)) =
        find_cached_inbox_note(db, original_uri)
    {
        return (content, visibility, spoiler_text, sensitive);
    }

    (String::new(), Visibility::Public, None, false)
}

/// Scan the `inbox` table for a `Create(Note)` whose embedded note's `id`
/// matches `original_uri`, returning the extracted note fields.
fn find_cached_inbox_note(db: &impl Database, original_uri: &str) -> Option<ExtractedNote> {
    let query = Query::builder().all().build();
    let rows = db.select_raw("inbox", query).ok()?;

    for row in &rows {
        let Some(json) = find_value(row, "object_data").and_then(|v| v.as_json()) else {
            continue;
        };
        let Ok(activity) = serde_json::from_value::<Activity>(json.value().clone()) else {
            continue;
        };
        if activity.base.kind != ApActivityType::Create {
            continue;
        }
        let Some(ActivityObject::Object(note)) = activity.object.as_ref() else {
            continue;
        };
        if note.id.as_deref() == Some(original_uri) {
            return extract_note_from_activity(&activity);
        }
    }
    None
}

/// Extracts the text content, inferred [`Visibility`], optional spoiler text,
/// `sensitive` flag, and the note's canonical id (when present) from a
/// `Create(Note)` activity.
fn extract_note_from_activity(activity: &Activity) -> Option<ExtractedNote> {
    let ActivityObject::Object(note) = activity.object.as_ref()? else {
        return None;
    };

    if note.kind != ObjectType::Note {
        return None;
    }

    let content = note.content.clone()?;
    let visibility = infer_visibility(&activity.base);
    let spoiler_text = note.summary.clone();
    let sensitive = note.sensitive.unwrap_or(false);
    let note_id = note.id.clone();

    Some((content, visibility, spoiler_text, sensitive, note_id))
}

/// Infers [`Visibility`] from ActivityPub `to`/`cc` addressing conventions
/// (same rules used by Mastodon):
///
/// - `to` contains `as:Public` → `Public`
/// - `cc` contains `as:Public` → `Unlisted`
/// - `to`/`cc` contains a `/followers` collection URL → `FollowersOnly`
/// - Otherwise → `Direct`
fn infer_visibility(base: &BaseObject<ApActivityType>) -> Visibility {
    let has_public_to = base
        .to
        .as_ref()
        .is_some_and(|t| one_or_many_contains(t, AS_PUBLIC));
    let has_public_cc = base
        .cc
        .as_ref()
        .is_some_and(|c| one_or_many_contains(c, AS_PUBLIC));

    if has_public_to {
        Visibility::Public
    } else if has_public_cc {
        Visibility::Unlisted
    } else if has_followers_collection(base) {
        Visibility::FollowersOnly
    } else {
        Visibility::Direct
    }
}

/// Returns `true` if any `to` or `cc` entry looks like a followers collection
/// URL (ends with `/followers`).
fn has_followers_collection(base: &BaseObject<ApActivityType>) -> bool {
    let check = |col: &Option<OneOrMany<String>>| {
        col.as_ref().is_some_and(|c| match c {
            OneOrMany::One(s) => s.ends_with("/followers"),
            OneOrMany::Many(v) => v.iter().any(|s| s.ends_with("/followers")),
        })
    };

    check(&base.to) || check(&base.cc)
}

/// Returns `true` if the given `actor_uri` appears in the activity's `to` or
/// `cc` fields.
fn is_addressed_to(base: &BaseObject<ApActivityType>, actor_uri: &str) -> bool {
    let in_to = base
        .to
        .as_ref()
        .is_some_and(|t| one_or_many_contains(t, actor_uri));
    let in_cc = base
        .cc
        .as_ref()
        .is_some_and(|c| one_or_many_contains(c, actor_uri));

    in_to || in_cc
}

/// Returns `true` if a [`OneOrMany`] contains the given value.
fn one_or_many_contains(collection: &OneOrMany<String>, value: &str) -> bool {
    match collection {
        OneOrMany::One(s) => s == value,
        OneOrMany::Many(v) => v.iter().any(|s| s == value),
    }
}

/// Finds the first non-null column with the given `name` in a raw row.
fn find_value<'a>(row: &'a [(ColumnDef, Value)], name: &str) -> Option<&'a Value> {
    row.iter()
        .find(|(col, _)| col.name == name)
        .map(|(_, v)| v)
        .filter(|v| !matches!(v, Value::Null))
}

#[cfg(test)]
mod tests {

    use activitypub::activity::{Activity, ActivityObject, ActivityType as ApActivityType};
    use activitypub::context::ACTIVITY_STREAMS_CONTEXT;
    use activitypub::object::{BaseObject, ObjectType, OneOrMany};
    use did::common::{FeedItem, Visibility};
    use did::user::{ReadFeedArgs, ReadFeedError, ReadFeedResponse};
    use ic_dbms_canister::prelude::DBMS_CONTEXT;
    use wasm_dbms::WasmDbmsDatabase;
    use wasm_dbms_api::prelude::{Database, Nullable};

    use super::read_feed;
    use crate::schema::{
        ActivityType as DbActivityType, Boost, BoostInsertRequest, FeedEntry,
        FeedEntryInsertRequest, FeedSource, InboxActivity, InboxActivityInsertRequest, Schema,
        Status, StatusInsertRequest, Visibility as DbVisibility,
    };
    use crate::test_utils::setup;

    fn insert_status(id: u64, content: &str, visibility: Visibility, created_at: u64) {
        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);
            db.insert::<Status>(StatusInsertRequest {
                id: id.into(),
                content: content.into(),
                visibility: DbVisibility::from(visibility),
                like_count: 0u64.into(),
                boost_count: 0u64.into(),
                in_reply_to_uri: Nullable::Null,
                spoiler_text: Nullable::Null,
                sensitive: false.into(),
                edited_at: Nullable::Null,
                created_at: created_at.into(),
            })
            .expect("should insert status");
            db.insert::<FeedEntry>(FeedEntryInsertRequest {
                id: id.into(),
                source: FeedSource::Outbox,
                created_at: created_at.into(),
            })
            .expect("should insert feed entry");
            db.commit().expect("should commit");
        });
    }

    fn make_create_note_json(content: &str, visibility: Visibility) -> serde_json::Value {
        make_create_note_json_addressed(content, visibility, &[])
    }

    fn make_create_note_json_addressed(
        content: &str,
        visibility: Visibility,
        extra_to: &[&str],
    ) -> serde_json::Value {
        let (mut to, cc) = match visibility {
            Visibility::Public => (
                Some(OneOrMany::One(
                    "https://www.w3.org/ns/activitystreams#Public".to_string(),
                )),
                None,
            ),
            Visibility::Unlisted => (
                None,
                Some(OneOrMany::One(
                    "https://www.w3.org/ns/activitystreams#Public".to_string(),
                )),
            ),
            Visibility::FollowersOnly => (
                Some(OneOrMany::One(
                    "https://remote.example/users/bob/followers".to_string(),
                )),
                None,
            ),
            Visibility::Direct => (None, None),
        };

        // Append extra recipients to the `to` field.
        if !extra_to.is_empty() {
            let mut recipients: Vec<String> = match to.take() {
                Some(OneOrMany::One(s)) => vec![s],
                Some(OneOrMany::Many(v)) => v,
                None => Vec::new(),
            };
            recipients.extend(extra_to.iter().map(|s| s.to_string()));
            to = Some(OneOrMany::Many(recipients));
        }

        let note = BaseObject {
            kind: ObjectType::Note,
            content: Some(content.to_string()),
            to: to.clone(),
            cc: cc.clone(),
            ..Default::default()
        };

        let activity = Activity {
            base: BaseObject {
                context: Some(activitypub::context::Context::Uri(
                    ACTIVITY_STREAMS_CONTEXT.to_string(),
                )),
                kind: ApActivityType::Create,
                to,
                cc,
                ..Default::default()
            },
            actor: Some("https://remote.example/users/bob".to_string()),
            object: Some(ActivityObject::Object(Box::new(note))),
            target: None,
            result: None,
            origin: None,
            instrument: None,
        };

        serde_json::to_value(&activity).expect("activity serialization must not fail")
    }

    fn insert_inbox_create_addressed(
        id: u64,
        content: &str,
        visibility: Visibility,
        created_at: u64,
        extra_to: &[&str],
    ) {
        let object_data = make_create_note_json_addressed(content, visibility, extra_to);
        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);
            db.insert::<InboxActivity>(InboxActivityInsertRequest {
                id: id.into(),
                activity_type: DbActivityType::from(ApActivityType::Create),
                actor_uri: "https://remote.example/users/bob".into(),
                object_data: object_data.into(),
                is_boost: false.into(),
                original_status_uri: Nullable::Null,
                created_at: created_at.into(),
            })
            .expect("should insert inbox activity");
            db.insert::<FeedEntry>(FeedEntryInsertRequest {
                id: id.into(),
                source: FeedSource::Inbox,
                created_at: created_at.into(),
            })
            .expect("should insert feed entry");
            db.commit().expect("should commit");
        });
    }

    fn insert_inbox_create(id: u64, content: &str, visibility: Visibility, created_at: u64) {
        let object_data = make_create_note_json(content, visibility);
        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);
            db.insert::<InboxActivity>(InboxActivityInsertRequest {
                id: id.into(),
                activity_type: DbActivityType::from(ApActivityType::Create),
                actor_uri: "https://remote.example/users/bob".into(),
                object_data: object_data.into(),
                is_boost: false.into(),
                original_status_uri: Nullable::Null,
                created_at: created_at.into(),
            })
            .expect("should insert inbox activity");
            db.insert::<FeedEntry>(FeedEntryInsertRequest {
                id: id.into(),
                source: FeedSource::Inbox,
                created_at: created_at.into(),
            })
            .expect("should insert feed entry");
            db.commit().expect("should commit");
        });
    }

    fn unwrap_ok(response: ReadFeedResponse) -> Vec<FeedItem> {
        let ReadFeedResponse::Ok(items) = response else {
            panic!("expected Ok, got {response:?}");
        };
        items
    }

    #[test]
    fn test_should_return_empty_feed_when_no_items() {
        setup();

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert!(items.is_empty());
    }

    #[test]
    fn test_should_return_outbox_statuses() {
        setup();
        insert_status(1, "Hello world", Visibility::Public, 1000);
        insert_status(2, "Second post", Visibility::Public, 2000);

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 2);
        // newest first
        assert_eq!(items[0].status.content, "Second post");
        assert_eq!(items[1].status.content, "Hello world");
        assert!(items[0].boosted_by.is_none());
    }

    #[test]
    fn test_should_return_inbox_create_activities() {
        setup();
        insert_inbox_create(100, "Remote status", Visibility::Public, 3000);

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status.content, "Remote status");
        assert_eq!(items[0].status.author, "https://remote.example/users/bob");
    }

    #[test]
    fn test_should_merge_and_sort_outbox_and_inbox() {
        setup();
        insert_status(1, "Own status early", Visibility::Public, 1000);
        insert_inbox_create(100, "Remote status mid", Visibility::Public, 2000);
        insert_status(2, "Own status late", Visibility::Public, 3000);

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].status.content, "Own status late");
        assert_eq!(items[1].status.content, "Remote status mid");
        assert_eq!(items[2].status.content, "Own status early");
    }

    #[test]
    fn test_should_paginate_with_offset_and_limit() {
        setup();
        for i in 1..=5 {
            insert_status(i, &format!("Status {i}"), Visibility::Public, i * 1000);
        }

        // page 1: items at indices 1..3 (after sorting newest-first: 5,4,3,2,1)
        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 2,
            offset: 1,
        }));

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].status.content, "Status 4");
        assert_eq!(items[1].status.content, "Status 3");
    }

    #[test]
    fn test_should_return_limit_exceeded_error() {
        setup();

        let response = read_feed(ReadFeedArgs {
            limit: crate::domain::MAX_PAGE_LIMIT + 1,
            offset: 0,
        });

        assert_eq!(
            response,
            ReadFeedResponse::Err(ReadFeedError::LimitExceeded)
        );
    }

    #[test]
    fn test_should_infer_visibility_from_addressing() {
        setup();
        insert_inbox_create(100, "Public post", Visibility::Public, 1000);
        insert_inbox_create(101, "Unlisted post", Visibility::Unlisted, 2000);
        insert_inbox_create(102, "Followers only", Visibility::FollowersOnly, 3000);

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 3);
        // sorted newest first
        assert_eq!(items[0].status.visibility, Visibility::FollowersOnly);
        assert_eq!(items[1].status.visibility, Visibility::Unlisted);
        assert_eq!(items[2].status.visibility, Visibility::Public);
    }

    #[test]
    fn test_should_return_empty_page_beyond_data() {
        setup();
        insert_status(1, "Only status", Visibility::Public, 1000);

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 100,
        }));

        assert!(items.is_empty());
    }

    #[test]
    fn test_should_set_owner_as_author_for_outbox_items() {
        setup();
        insert_status(1, "My post", Visibility::Public, 1000);

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].status.author,
            "https://mastic.social/users/rey_canisteryo"
        );
    }

    #[test]
    fn test_should_show_direct_message_when_owner_is_addressed() {
        setup();
        let owner_uri = crate::domain::urls::actor_uri("rey_canisteryo").unwrap();
        insert_inbox_create_addressed(100, "DM for you", Visibility::Direct, 1000, &[&owner_uri]);

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status.content, "DM for you");
        assert_eq!(items[0].status.visibility, Visibility::Direct);
    }

    fn insert_boost_wrapper(snowflake: u64, original_uri: &str, content: &str, created_at: u64) {
        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);
            db.insert::<Status>(StatusInsertRequest {
                id: snowflake.into(),
                content: content.into(),
                visibility: DbVisibility::from(Visibility::Public),
                like_count: 0u64.into(),
                boost_count: 0u64.into(),
                in_reply_to_uri: Nullable::Null,
                spoiler_text: Nullable::Null,
                sensitive: false.into(),
                edited_at: Nullable::Null,
                created_at: created_at.into(),
            })
            .expect("should insert wrapper status");
            db.insert::<Boost>(BoostInsertRequest {
                id: snowflake.into(),
                status_id: snowflake.into(),
                original_status_uri: original_uri.into(),
                created_at: created_at.into(),
            })
            .expect("should insert boost");
            db.insert::<FeedEntry>(FeedEntryInsertRequest {
                id: snowflake.into(),
                source: FeedSource::Outbox,
                created_at: created_at.into(),
            })
            .expect("should insert feed entry");
            db.commit().expect("should commit");
        });
    }

    #[test]
    fn test_outbox_boost_renders_boosted_by_self_and_original_author() {
        setup();
        let original = "https://remote.example/users/bob/statuses/99";
        insert_boost_wrapper(7, original, "boosted text", 5_000);

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].boosted_by.as_deref(),
            Some("https://mastic.social/users/rey_canisteryo")
        );
        assert_eq!(items[0].status.author, "https://remote.example/users/bob");
        assert_eq!(items[0].status.id, 99);
        assert_eq!(items[0].status.content, "boosted text");
    }

    #[test]
    fn test_outbox_non_boost_renders_owner_as_author_with_no_boosted_by() {
        setup();
        insert_status(8, "Mine", Visibility::Public, 1_000);

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 1);
        assert!(items[0].boosted_by.is_none());
        assert_eq!(
            items[0].status.author,
            "https://mastic.social/users/rey_canisteryo"
        );
        assert_eq!(items[0].status.id, 8);
        assert_eq!(items[0].status.content, "Mine");
    }

    #[test]
    fn test_should_hide_direct_message_when_owner_is_not_addressed() {
        setup();
        insert_inbox_create_addressed(
            100,
            "DM not for you",
            Visibility::Direct,
            1000,
            &["https://remote.example/users/charlie"],
        );

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert!(items.is_empty());
    }

    fn insert_inbox_announce(id: u64, booster: &str, target_uri: &str, created_at: u64) {
        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);
            db.insert::<InboxActivity>(InboxActivityInsertRequest {
                id: id.into(),
                activity_type: DbActivityType::from(ApActivityType::Announce),
                actor_uri: booster.into(),
                object_data: serde_json::json!({"type": "Announce"}).into(),
                is_boost: true.into(),
                original_status_uri: Nullable::Value(target_uri.into()),
                created_at: created_at.into(),
            })
            .expect("insert inbox row");
            db.insert::<FeedEntry>(FeedEntryInsertRequest {
                id: id.into(),
                source: FeedSource::Inbox,
                created_at: created_at.into(),
            })
            .expect("insert feed entry");
            db.commit().expect("commit");
        });
    }

    #[test]
    fn test_inbox_boost_renders_boosted_by_booster_and_local_original_author() {
        setup();
        insert_status(99, "Bob's post", Visibility::Public, 1_000);
        insert_inbox_announce(
            500,
            "https://remote.example/users/alice",
            "https://mastic.social/users/rey_canisteryo/statuses/99",
            2_000,
        );

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 2);
        let boost = &items[0]; // newer first
        assert_eq!(
            boost.boosted_by.as_deref(),
            Some("https://remote.example/users/alice")
        );
        assert_eq!(
            boost.status.author,
            "https://mastic.social/users/rey_canisteryo"
        );
        assert_eq!(boost.status.id, 99);
        assert_eq!(boost.status.content, "Bob's post");
    }

    #[test]
    fn test_inbox_boost_falls_back_to_empty_when_original_not_cached() {
        setup();
        insert_inbox_announce(
            500,
            "https://remote.example/users/alice",
            "https://other.example/users/bob/statuses/99",
            2_000,
        );

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].boosted_by.as_deref(),
            Some("https://remote.example/users/alice")
        );
        assert_eq!(items[0].status.author, "https://other.example/users/bob");
        assert_eq!(items[0].status.id, 99);
        assert_eq!(items[0].status.content, "");
    }

    #[test]
    fn test_outbox_status_flags_default_false() {
        setup();
        insert_status(1, "Plain", Visibility::Public, 1_000);

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 1);
        assert!(!items[0].liked);
        assert!(!items[0].boosted);
    }

    #[test]
    fn test_outbox_status_liked_flag_set_when_user_liked_own_status() {
        setup();
        insert_status(1, "Mine", Visibility::Public, 1_000);
        let owner_uri = crate::domain::urls::actor_uri("rey_canisteryo").unwrap();
        let status_uri = format!("{owner_uri}/statuses/1");
        crate::domain::liked::LikedRepository::oneshot()
            .like_status(&status_uri)
            .expect("like");

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 1);
        assert!(items[0].liked);
        assert!(!items[0].boosted);
    }

    #[test]
    fn test_outbox_self_boost_marks_wrapper_boosted_true() {
        setup();
        let owner_uri = crate::domain::urls::actor_uri("rey_canisteryo").unwrap();
        // Original status authored by the owner, then boosted by the owner.
        insert_status(1, "Mine", Visibility::Public, 1_000);
        let original_uri = format!("{owner_uri}/statuses/1");
        insert_boost_wrapper(7, &original_uri, "boosted text", 5_000);

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 2);
        // Newest first: boost wrapper, then the original.
        let wrapper = &items[0];
        assert!(wrapper.boosted_by.is_some());
        assert!(wrapper.boosted, "wrapper resolves boosted=true");

        let original = &items[1];
        assert!(original.boosted_by.is_none());
        assert!(
            original.boosted,
            "original status is also boosted by the viewer"
        );
    }

    #[test]
    fn test_inbox_boost_sets_liked_when_viewer_liked_original() {
        setup();
        // Viewer's local status acts as the boosted target.
        insert_status(99, "Bob's post", Visibility::Public, 1_000);
        let original_uri = "https://mastic.social/users/rey_canisteryo/statuses/99";
        crate::domain::liked::LikedRepository::oneshot()
            .like_status(original_uri)
            .expect("like");

        insert_inbox_announce(
            500,
            "https://remote.example/users/alice",
            original_uri,
            2_000,
        );

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 2);
        let boost_item = items
            .iter()
            .find(|i| i.boosted_by.is_some())
            .expect("boost item present");
        assert!(boost_item.liked);
        assert!(!boost_item.boosted);
    }

    #[test]
    fn test_inbox_create_note_uses_note_id_for_flag_lookup() {
        setup();
        // Build an inbox Create(Note) whose note carries an explicit `id`,
        // and like that id from the viewer's perspective.
        let note_uri = "https://remote.example/users/bob/statuses/42";
        crate::domain::liked::LikedRepository::oneshot()
            .like_status(note_uri)
            .expect("like");

        let object_data = make_create_note_with_id(note_uri, "Remote post", Visibility::Public);
        DBMS_CONTEXT.with(|ctx| {
            let tx_id =
                ctx.begin_transaction(db_utils::transaction::transaction_caller(ic_utils::now()));
            let mut db = WasmDbmsDatabase::from_transaction(ctx, Schema, tx_id);
            db.insert::<InboxActivity>(InboxActivityInsertRequest {
                id: 200u64.into(),
                activity_type: DbActivityType::from(ApActivityType::Create),
                actor_uri: "https://remote.example/users/bob".into(),
                object_data: object_data.into(),
                is_boost: false.into(),
                original_status_uri: Nullable::Null,
                created_at: 3_000u64.into(),
            })
            .expect("insert inbox row");
            db.insert::<FeedEntry>(FeedEntryInsertRequest {
                id: 200u64.into(),
                source: FeedSource::Inbox,
                created_at: 3_000u64.into(),
            })
            .expect("insert feed entry");
            db.commit().expect("commit");
        });

        let items = unwrap_ok(read_feed(ReadFeedArgs {
            limit: 10,
            offset: 0,
        }));

        assert_eq!(items.len(), 1);
        assert!(items[0].liked);
        assert!(!items[0].boosted);
    }

    fn make_create_note_with_id(
        note_id: &str,
        content: &str,
        visibility: Visibility,
    ) -> serde_json::Value {
        let (to, cc) = match visibility {
            Visibility::Public => (
                Some(OneOrMany::One(
                    "https://www.w3.org/ns/activitystreams#Public".to_string(),
                )),
                None,
            ),
            _ => (None, None),
        };

        let note = BaseObject {
            kind: ObjectType::Note,
            id: Some(note_id.to_string()),
            content: Some(content.to_string()),
            to: to.clone(),
            cc: cc.clone(),
            ..Default::default()
        };

        let activity = Activity {
            base: BaseObject {
                context: Some(activitypub::context::Context::Uri(
                    ACTIVITY_STREAMS_CONTEXT.to_string(),
                )),
                kind: ApActivityType::Create,
                to,
                cc,
                ..Default::default()
            },
            actor: Some("https://remote.example/users/bob".to_string()),
            object: Some(ActivityObject::Object(Box::new(note))),
            target: None,
            result: None,
            origin: None,
            instrument: None,
        };

        serde_json::to_value(&activity).expect("serialize")
    }
}
