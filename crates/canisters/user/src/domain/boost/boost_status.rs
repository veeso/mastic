//! Flow for boosting a status.
//!
//! See `.claude/specs/2026-05-02-wi-1-7-boost-status-design.md` for the
//! full design.
//!
//! Pipeline:
//! 1. Idempotency: if a boost row for `status_url` already exists → `Ok`.
//! 2. Fetch the original status via `Federation.fetch_status` so the
//!    wrapper's denormalized content is verified, not caller-supplied.
//! 3. Mint a single Snowflake reused as `boosts.id`, `boosts.status_id`,
//!    wrapper `statuses.id`, and `feed.id`. The wrapper status URL
//!    `<own_actor_uri>/statuses/<snowflake>` doubles as the `Announce`
//!    activity id.
//! 4. Insert the wrapper Status, Boost, and FeedEntry in one transaction.
//! 5. Build an `Announce` activity addressed to followers ∪ {original
//!    author} (deduplicated; self-boost addresses self) and dispatch
//!    a per-recipient batch via the federation adapter.

use activitypub::Activity;
use activitypub::activity::{ActivityObject, ActivityType};
use activitypub::context::ACTIVITY_STREAMS_CONTEXT;
use activitypub::object::{BaseObject, OneOrMany};
use did::common::Status;
use did::federation::{
    FetchStatusArgs, FetchStatusResponse, SendActivityArgs, SendActivityArgsObject,
};
use did::user::{BoostStatusArgs, BoostStatusError, BoostStatusResponse};

use crate::domain::boost::repository::BoostRepository;
use crate::domain::follower::FollowerRepository;
use crate::domain::profile::ProfileRepository;
use crate::domain::snowflake::Snowflake;
use crate::domain::urls;
use crate::error::{CanisterError, CanisterResult};

const AS_PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";

pub async fn boost_status(BoostStatusArgs { status_url }: BoostStatusArgs) -> BoostStatusResponse {
    match boost_status_inner(status_url).await {
        Ok(()) => BoostStatusResponse::Ok,
        Err(err) => {
            ic_utils::log!("Failed to boost status: {err}");
            BoostStatusResponse::Err(BoostStatusError::Internal(err.to_string()))
        }
    }
}

async fn boost_status_inner(status_url: String) -> CanisterResult<()> {
    ic_utils::log!("Boosting status {status_url}");

    if BoostRepository::is_boosted(&status_url)? {
        ic_utils::log!("Status already boosted: {status_url}");
        return Ok(());
    }

    let own_profile = ProfileRepository::get_profile()?;
    let own_actor_uri = urls::actor_uri(&own_profile.handle.0)?;

    let fetched = match crate::adapters::federation::fetch_status(FetchStatusArgs {
        uri: status_url.clone(),
        requester_actor_uri: Some(own_actor_uri.clone()),
    })
    .await?
    {
        FetchStatusResponse::Ok(status) => status,
        FetchStatusResponse::Err(err) => {
            return Err(CanisterError::Internal(format!(
                "fetch_status failed: {err:?}"
            )));
        }
    };

    let snowflake: u64 = Snowflake::new().into();
    let now = ic_utils::now();

    BoostRepository::insert_boost_with_wrapper(
        snowflake,
        &status_url,
        &fetched.content,
        fetched.visibility.into(),
        fetched.spoiler_text.as_deref(),
        fetched.sensitive,
        now,
    )?;

    let recipients = compute_recipients(&fetched, &own_actor_uri)?;
    if recipients.is_empty() {
        ic_utils::log!("Boost: no recipients to dispatch");
        return Ok(());
    }
    let activity = make_announce_activity(&own_actor_uri, snowflake, &status_url, &recipients);
    let activity_json =
        serde_json::to_string(&activity).expect("Activity serialization must not fail");
    let batch: Vec<SendActivityArgsObject> = recipients
        .iter()
        .map(|recipient| SendActivityArgsObject {
            activity_json: activity_json.clone(),
            target_inbox: format!("{recipient}/inbox"),
        })
        .collect();

    crate::adapters::federation::send_activity(SendActivityArgs::Batch(batch)).await?;
    Ok(())
}

/// Compute the dedup'd recipient list for an Announce: followers ∪ author.
///
/// - For a self-boost (author == self), include self only if it isn't
///   already in the followers set.
/// - For a third-party boost, include the author unconditionally
///   (deduplication via sort + dedup handles overlap with followers).
fn compute_recipients(fetched: &Status, own_actor_uri: &str) -> CanisterResult<Vec<String>> {
    let mut recipients: Vec<String> = FollowerRepository::get_followers()?
        .into_iter()
        .map(|f| f.actor_uri.0)
        .collect();
    if fetched.author == own_actor_uri {
        if !recipients.iter().any(|r| r == own_actor_uri) {
            recipients.push(own_actor_uri.to_string());
        }
    } else if !fetched.author.is_empty() {
        recipients.push(fetched.author.clone());
    }
    recipients.sort();
    recipients.dedup();
    Ok(recipients)
}

fn make_announce_activity(
    own_actor_uri: &str,
    wrapper_id: u64,
    status_url: &str,
    recipients: &[String],
) -> Activity {
    let cc = if recipients.is_empty() {
        None
    } else {
        Some(OneOrMany::Many(recipients.to_vec()))
    };
    Activity {
        base: BaseObject {
            id: Some(format!("{own_actor_uri}/statuses/{wrapper_id}")),
            context: Some(activitypub::context::Context::Uri(
                ACTIVITY_STREAMS_CONTEXT.to_string(),
            )),
            kind: ActivityType::Announce,
            to: Some(OneOrMany::One(AS_PUBLIC.to_string())),
            cc,
            ..Default::default()
        },
        actor: Some(own_actor_uri.to_string()),
        object: Some(ActivityObject::Id(status_url.to_string())),
        target: None,
        result: None,
        origin: None,
        instrument: None,
    }
}

#[cfg(test)]
mod tests {
    use activitypub::activity::ActivityType;
    use did::common::{Status, Visibility};
    use did::federation::{FetchStatusError, FetchStatusResponse, SendActivityArgs};
    use did::user::{BoostStatusArgs, BoostStatusError, BoostStatusResponse};

    use super::*;
    use crate::adapters::federation::mock::{captured, push_fetch_status_response};
    use crate::domain::boost::repository::BoostRepository;
    use crate::domain::follower::FollowerRepository;
    use crate::test_utils::setup;

    const STATUS_URI: &str = "https://remote.example/users/bob/statuses/42";
    const OWN_URI: &str = "https://mastic.social/users/rey_canisteryo";
    const FOLLOWER_URI: &str = "https://remote.example/users/charlie";

    fn fixture_status() -> Status {
        Status {
            id: 42,
            content: "hello".into(),
            author: "https://remote.example/users/bob".into(),
            created_at: 1_000,
            visibility: Visibility::Public,
            like_count: 0,
            boost_count: 0,
            spoiler_text: Some("cw".into()),
            sensitive: true,
        }
    }

    fn insert_follower(uri: &str) {
        FollowerRepository::insert(uri).expect("insert follower");
    }

    #[tokio::test]
    async fn test_should_boost_status() {
        setup();
        insert_follower(FOLLOWER_URI);
        push_fetch_status_response(FetchStatusResponse::Ok(fixture_status()));

        let resp = boost_status(BoostStatusArgs {
            status_url: STATUS_URI.to_string(),
        })
        .await;
        assert_eq!(resp, BoostStatusResponse::Ok);
        assert!(BoostRepository::is_boosted(STATUS_URI).expect("query"));

        let captured = captured();
        assert_eq!(captured.len(), 1);
        let SendActivityArgs::Batch(batch) = &captured[0] else {
            panic!("expected batch");
        };
        // recipients: follower + bob (author), deduplicated
        assert_eq!(batch.len(), 2);
        for entry in batch {
            let activity: activitypub::Activity =
                serde_json::from_str(&entry.activity_json).unwrap();
            assert_eq!(activity.base.kind, ActivityType::Announce);
            assert_eq!(activity.actor.as_deref(), Some(OWN_URI));
        }
    }

    #[tokio::test]
    async fn test_should_be_idempotent_when_already_boosted() {
        setup();
        insert_follower(FOLLOWER_URI);
        push_fetch_status_response(FetchStatusResponse::Ok(fixture_status()));

        assert_eq!(
            boost_status(BoostStatusArgs {
                status_url: STATUS_URI.into()
            })
            .await,
            BoostStatusResponse::Ok
        );
        assert_eq!(
            boost_status(BoostStatusArgs {
                status_url: STATUS_URI.into()
            })
            .await,
            BoostStatusResponse::Ok
        );
        assert_eq!(captured().len(), 1, "only one dispatch");
    }

    #[tokio::test]
    async fn test_should_propagate_fetch_error_as_internal() {
        setup();
        push_fetch_status_response(FetchStatusResponse::Err(FetchStatusError::NotFound));

        let resp = boost_status(BoostStatusArgs {
            status_url: STATUS_URI.into(),
        })
        .await;
        assert!(matches!(
            resp,
            BoostStatusResponse::Err(BoostStatusError::Internal(_))
        ));
        assert!(!BoostRepository::is_boosted(STATUS_URI).expect("query"));
    }

    #[tokio::test]
    async fn test_self_boost_allowed() {
        setup();
        let mut status = fixture_status();
        status.author = OWN_URI.into();
        push_fetch_status_response(FetchStatusResponse::Ok(status));

        let resp = boost_status(BoostStatusArgs {
            status_url: format!("{OWN_URI}/statuses/9"),
        })
        .await;
        assert_eq!(resp, BoostStatusResponse::Ok);
    }

    #[tokio::test]
    async fn test_dispatch_dedups_when_follower_is_author() {
        setup();
        insert_follower("https://remote.example/users/bob");
        push_fetch_status_response(FetchStatusResponse::Ok(fixture_status()));

        let _ = boost_status(BoostStatusArgs {
            status_url: STATUS_URI.into(),
        })
        .await;
        let captured = captured();
        let SendActivityArgs::Batch(batch) = &captured[0] else {
            unreachable!()
        };
        assert_eq!(batch.len(), 1, "follower == author should dedup to 1");
    }
}
