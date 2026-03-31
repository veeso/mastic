//! ActivityPub actor types.
//!
//! An [`Actor`] is the identity behind all ActivityPub interactions. This
//! module models the five standard actor types plus Mastodon-specific profile
//! extensions (discoverability, featured collections, suspension flags) and
//! HTTP Signature public keys.
// Rust guideline compliant 2026-03-31

use serde::{Deserialize, Serialize};

use crate::mastodon::MediaReference;
use crate::object::{BaseObject, OneOrMany};

/// The ActivityPub actor family of type discriminators.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorType {
    /// A human account.
    Person,
    /// A software application.
    Application,
    /// A service actor.
    Service,
    /// A group actor.
    Group,
    /// An organization actor.
    Organization,
    /// Any other actor type not explicitly modeled above.
    #[serde(other)]
    Other,
}

/// A public key advertised on an ActivityPub actor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    /// Stable URI identifying the key.
    pub id: String,
    /// URI of the actor that owns the key.
    pub owner: String,
    /// PEM-encoded RSA public key material.
    pub public_key_pem: String,
}

/// Additional actor endpoints advertised to remote servers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
    /// Shared inbox endpoint for batched delivery to this actor's server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_inbox: Option<String>,
}

/// A fully described ActivityPub actor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    /// Shared ActivityStreams object fields for the actor payload.
    #[serde(flatten)]
    pub base: BaseObject<ActorType>,
    /// Inbox endpoint receiving server-to-server activities.
    pub inbox: String,
    /// Outbox endpoint serving locally authored activities.
    pub outbox: String,
    /// Collection of actors followed by this actor.
    pub following: String,
    /// Collection of actors following this actor.
    pub followers: String,
    /// Collection of liked activities or objects.
    pub liked: String,
    /// Preferred handle or username.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_username: Option<String>,
    /// Public verification key used for HTTP signatures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<PublicKey>,
    /// Additional advertised endpoints such as `sharedInbox`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Endpoints>,
    /// Whether follow requests require manual approval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manually_approves_followers: Option<bool>,
    /// Mastodon discoverability flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discoverable: Option<bool>,
    /// Mastodon indexability flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexable: Option<bool>,
    /// Mastodon suspension flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspended: Option<bool>,
    /// Mastodon memorial flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memorial: Option<bool>,
    /// Collection URI for featured or pinned posts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured: Option<String>,
    /// Collection URI for featured hashtags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured_tags: Option<String>,
    /// Alias URIs for account migration and identity linkage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub also_known_as: Option<OneOrMany<String>>,
    /// Domains that may attribute content to this actor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution_domains: Option<OneOrMany<String>>,
    /// Icon media for the actor profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<OneOrMany<MediaReference>>,
    /// Header or profile image media for the actor profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<OneOrMany<MediaReference>>,
}
