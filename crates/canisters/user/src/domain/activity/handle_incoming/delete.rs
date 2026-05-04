//! Handle `Delete` activity.

use activitypub::Activity;
use db_utils::repository::Repository;
use db_utils::transaction::Transaction;
use did::user::ReceiveActivityError;

use crate::domain::activity::InboxActivityRepository;
use crate::domain::feed::FeedRepository;
use crate::error::{CanisterError, CanisterResult};
use crate::schema::Schema;

/// Handle an incoming `Delete` activity.
///
/// Purges every inbox row referencing the deleted status URI plus its
/// matching `feed` entry, in a single transaction:
///
/// - Boost rows: matched by `original_status_uri`.
/// - `Create(Note)` rows: matched by parsing the cached `object_data`
///   activity JSON and comparing the embedded note's `id`.
///
/// The handler is idempotent — when no rows match (already purged or
/// never received) it returns `Ok(())` without modifying state.
pub(super) fn handle_delete(activity: &Activity) -> Result<(), ReceiveActivityError> {
    let actor_uri = activity
        .actor
        .as_deref()
        .ok_or(ReceiveActivityError::ProcessingFailed)?;
    let Some(target_uri) = super::extract_object_uri(activity) else {
        ic_utils::log!("handle_incoming: Delete missing object URI; ignoring");
        return Ok(());
    };
    ic_utils::log!("handle_incoming: Delete from {actor_uri} on {target_uri}");
    let ids_to_delete = collect_delete_targets(&target_uri).map_err(|e| {
        ic_utils::log!("handle_incoming: failed to scan inbox for Delete: {e}");
        ReceiveActivityError::Internal(e.to_string())
    })?;

    if ids_to_delete.is_empty() {
        ic_utils::log!("handle_incoming: Delete found no matching inbox entries, ignoring");
        return Ok(());
    }

    Transaction::run::<_, _, _, CanisterError>(Schema, |tx| {
        let inbox_tx = InboxActivityRepository::with_transaction(tx);
        let feed_tx = FeedRepository::with_transaction(tx);
        for id in &ids_to_delete {
            feed_tx.delete_by_id(*id)?;
            inbox_tx.delete_by_id(*id)?;
        }
        Ok(())
    })
    .map_err(|e| {
        ic_utils::log!("handle_incoming: failed to apply Delete: {e}");
        ReceiveActivityError::Internal(e.to_string())
    })
}

/// Collect every inbox row id whose contents reference `target_uri`.
///
/// Combines three independent matches:
///
/// 1. Boost rows whose `original_status_uri` is the target.
/// 2. Create rows whose embedded note id is the full target URI.
/// 3. Create rows whose embedded note id is the bare snowflake string,
///    paired with the activity actor matching the URI's parent actor.
///    Mastic's publish pipeline currently mints note ids as bare
///    snowflakes, so this branch is what catches locally-published notes.
fn collect_delete_targets(target_uri: &str) -> CanisterResult<Vec<u64>> {
    let inbox = InboxActivityRepository::oneshot();
    let mut ids = inbox.find_boost_ids_by_original_uri(target_uri)?;
    ids.extend(inbox.find_create_ids_with_object_id(target_uri, None)?);

    if let Some(bare_id) = crate::domain::urls::parse_status_id(target_uri)
        && let Some(parent_actor) = crate::domain::urls::actor_uri_from_status_uri(target_uri)
    {
        for id in inbox.find_create_ids_with_object_id(&bare_id.to_string(), Some(&parent_actor))? {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    Ok(ids)
}

#[cfg(test)]
mod tests {

    use did::user::{ReceiveActivityArgs, ReceiveActivityResponse};

    use super::super::handle_incoming;
    use super::super::test_helpers::{
        REMOTE_AUTHOR, REMOTE_BOOSTER, REMOTE_NOTE_URI, count_feed, count_inbox,
        make_announce_json, make_create_note_json, make_delete_json,
    };
    use crate::test_utils::setup;

    #[test]
    fn test_should_purge_inbox_and_feed_on_delete_of_create_note() {
        setup();

        // ingest a remote Create(Note)
        let create = make_create_note_json(REMOTE_AUTHOR, REMOTE_NOTE_URI, "hi");
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: create
            }),
            ReceiveActivityResponse::Ok
        );
        assert_eq!(count_inbox(), 1);
        assert_eq!(count_feed(), 1);

        // delete it
        let delete = make_delete_json(REMOTE_AUTHOR, REMOTE_NOTE_URI);
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: delete
            }),
            ReceiveActivityResponse::Ok
        );
        assert_eq!(count_inbox(), 0);
        assert_eq!(count_feed(), 0);
    }

    #[test]
    fn test_should_purge_inbox_boost_referencing_deleted_uri() {
        setup();

        let announce = make_announce_json(REMOTE_BOOSTER, REMOTE_NOTE_URI);
        handle_incoming(ReceiveActivityArgs {
            activity_json: announce,
        });
        assert_eq!(count_inbox(), 1);
        assert_eq!(count_feed(), 1);

        let delete = make_delete_json(REMOTE_AUTHOR, REMOTE_NOTE_URI);
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: delete
            }),
            ReceiveActivityResponse::Ok
        );
        assert_eq!(count_inbox(), 0);
        assert_eq!(count_feed(), 0);
    }

    #[test]
    fn test_should_be_idempotent_on_repeat_delete() {
        setup();

        // ingest then delete twice
        handle_incoming(ReceiveActivityArgs {
            activity_json: make_create_note_json(REMOTE_AUTHOR, REMOTE_NOTE_URI, "hi"),
        });
        let delete = make_delete_json(REMOTE_AUTHOR, REMOTE_NOTE_URI);
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: delete.clone()
            }),
            ReceiveActivityResponse::Ok
        );
        // second delete on same URI must succeed silently
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: delete
            }),
            ReceiveActivityResponse::Ok
        );
    }

    #[test]
    fn test_should_succeed_when_delete_targets_unknown_uri() {
        setup();

        // No prior Create / Announce — must still succeed.
        let delete = make_delete_json(REMOTE_AUTHOR, REMOTE_NOTE_URI);
        assert_eq!(
            handle_incoming(ReceiveActivityArgs {
                activity_json: delete
            }),
            ReceiveActivityResponse::Ok
        );
    }

    #[test]
    fn test_should_not_purge_unrelated_inbox_rows_on_delete() {
        setup();

        let other = "https://remote.example/users/dave/statuses/8";
        handle_incoming(ReceiveActivityArgs {
            activity_json: make_create_note_json(REMOTE_AUTHOR, REMOTE_NOTE_URI, "a"),
        });
        handle_incoming(ReceiveActivityArgs {
            activity_json: make_create_note_json(REMOTE_AUTHOR, other, "b"),
        });

        let delete = make_delete_json(REMOTE_AUTHOR, REMOTE_NOTE_URI);
        handle_incoming(ReceiveActivityArgs {
            activity_json: delete,
        });

        // verify via raw select that exactly one inbox row remains and it
        // points to the un-deleted note URI.
        use ic_dbms_canister::prelude::DBMS_CONTEXT;
        use wasm_dbms::WasmDbmsDatabase;
        use wasm_dbms_api::prelude::{Database, Query};
        let count = DBMS_CONTEXT.with(|ctx| {
            let db = WasmDbmsDatabase::oneshot(ctx, crate::schema::Schema);
            db.select_raw("inbox", Query::builder().all().build())
                .unwrap()
                .len()
        });
        assert_eq!(count, 1);
    }
}
