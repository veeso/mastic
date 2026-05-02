# Database Schema

- [Database Schema](#database-schema)
  - [Shared Settings Table](#shared-settings-table)
  - [Directory Canister](#directory-canister)
    - [Directory Canister Settings Keys](#directory-canister-settings-keys)
    - [`moderators` Table](#moderators-table)
    - [`users` Table](#users-table)
    - [`tombstones` Table](#tombstones-table)
    - [`reports` Table](#reports-table)
  - [User Canister](#user-canister)
    - [User Canister Settings Keys](#user-canister-settings-keys)
    - [`profiles` Table](#profiles-table)
    - [`statuses` Table](#statuses-table)
    - [`inbox` Table](#inbox-table)
    - [`followers` Table](#followers-table)
    - [`following` Table](#following-table)
    - [`liked` Table](#liked-table)
    - [`blocks` Table](#blocks-table)
    - [`mutes` Table](#mutes-table)
    - [`bookmarks` Table](#bookmarks-table)
    - [`boosts` Table](#boosts-table)
    - [`media` Table](#media-table)
    - [`edit_history` Table](#edit_history-table)
    - [`hashtags` Table](#hashtags-table)
    - [`status_hashtags` Table](#status_hashtags-table)
    - [`featured_tags` Table](#featured_tags-table)
    - [`pinned_statuses` Table](#pinned_statuses-table)
    - [`profile_metadata` Table](#profile_metadata-table)
  - [Custom Data Types](#custom-data-types)
    - [`Visibility`](#visibility)
    - [`ActivityType`](#activitytype)
    - [`FollowStatus`](#followstatus)
    - [`ReportState`](#reportstate)
  - [Persistence](#persistence)

Mastic uses [wasm-dbms](https://github.com/veeso/wasm-dbms) for persistent
storage inside the Directory and User canisters. Because `wasm-dbms` manages its
own stable memory, `ic-stable-structures` cannot be used alongside it in these
canisters. Configuration values that would normally live in stable cells are
stored in a shared **settings** key-value table instead.

The Federation Canister does **not** use `wasm-dbms`; it uses
`ic-stable-structures` directly.

## Shared Settings Table

Both the Directory and User canisters include a `settings` table with the same
schema. Each row maps an integer key to a polymorphic value
(`SettingValue`, backed by the `wasm-dbms` `Value` type).

| Column  | Type              | Constraint  | Description                |
| :------ | :---------------- | :---------- | :------------------------- |
| `key`   | `UINT16`          | PRIMARY KEY | Setting identifier         |
| `value` | `SettingValue`    |             | Typed value for the entry  |

The `Settings` table and its helper methods (`set_config_key`,
`get_required_settings_value`, `get_settings_value`, `get_as_principal`) live in
the `db-utils` crate and are shared by both canisters.

## Directory Canister

### Directory Canister Settings Keys

| Key | Constant                     | Value Type | Description                          |
| :-- | :--------------------------- | :--------- | :----------------------------------- |
| `0` | `SETTING_FEDERATION_CANISTER`| `BLOB`     | Principal of the Federation canister |

### `moderators` Table

| Column       | Type        | Constraint  | Description                  |
| :----------- | :---------- | :---------- | :--------------------------- |
| `principal`  | `Principal` | PRIMARY KEY | The moderator's principal    |
| `created_at` | `UINT64`    |             | Timestamp when added         |

### `users` Table

| Column            | Type                  | Constraint         | Description                      |
| :---------------- | :-------------------- | :----------------- | :------------------------------- |
| `principal`       | `Principal`           | PRIMARY KEY        | The user's principal             |
| `handle`          | `TEXT`                | UNIQUE, validated  | User's unique handle             |
| `canister_id`     | `Nullable<Principal>` | UNIQUE             | User Canister ID                 |
| `canister_status` | `CanisterStatus`      |                    | `Active`, `CreationPending`, ... |
| `created_at`      | `UINT64`              |                    | Timestamp when registered        |

The `handle` column uses `HandleSanitizer` (trims whitespace, lowercases,
strips leading `@`) and `HandleValidator` (enforces the
[handle rules](../specs/handles.md)). See the [Handle Validation](../specs/handles.md) page
for the full specification.

### `tombstones` Table

Retains deleted handles to block immediate re-registration and to keep
an audit trail. `handle` uses the same sanitizer/validator pair as the
`users.handle` column; see the [Handle Validation](../specs/handles.md) spec.

| Column       | Type        | Constraint                      | Description                         |
| :----------- | :---------- | :------------------------------ | :---------------------------------- |
| `handle`     | `TEXT`      | PRIMARY KEY, sanitized, validated | Handle of the deleted user        |
| `principal`  | `Principal` |                                 | Principal of the deleted user       |
| `deleted_at` | `UINT64`    |                                 | Timestamp when the user was deleted |

### `reports` Table

Stores user-submitted moderation reports. See the
[Reports](../specs/reports.md) spec for validation rules on `reason`
and `target_status_uri`.

| Column              | Type                  | Constraint           | Description                            |
| :------------------ | :-------------------- | :------------------- | :------------------------------------- |
| `id`                | `UINT64`              | PRIMARY KEY          | [Snowflake ID](../specs/snowflake.md)  |
| `reporter`          | `Principal`           |                      | Principal of the reporter              |
| `target_canister`   | `Principal`           |                      | Reported user's canister principal     |
| `target_status_uri` | `Nullable<Text>`      | validated            | URI of the reported status, if any     |
| `reason`            | `TEXT`                | sanitized, validated | Free-form reason                       |
| `state`             | `ReportState`         |                      | `Open`, `Resolved`, `Dismissed`        |
| `created_at`        | `UINT64`              | INDEX                | Submission timestamp                   |
| `resolved_at`       | `Nullable<Uint64>`    |                      | Timestamp when the report was resolved |
| `resolved_by`       | `Nullable<Principal>` |                      | Moderator who resolved the report      |

## User Canister

### User Canister Settings Keys

| Key | Constant                      | Value Type | Description                          |
| :-- | :---------------------------- | :--------- | :----------------------------------- |
| `0` | `SETTING_FEDERATION_CANISTER` | `BLOB`     | Principal of the Federation canister |
| `1` | `SETTING_OWNER_PRINCIPAL`     | `BLOB`     | Principal of the canister owner      |
| `2` | `SETTING_PUBLIC_URL`          | `TEXT`     | Public URL of the Mastic instance    |
| `3` | `SETTING_DIRECTORY_CANISTER`  | `BLOB`     | Principal of the Directory canister  |

### `profiles` Table

Single-row table holding the owner's profile.

| Column         | Type              | Constraint        | Description            |
| :------------- | :---------------- | :---------------- | :--------------------- |
| `principal`    | `Principal`       | PRIMARY KEY       | Owner's principal      |
| `handle`       | `TEXT`            | UNIQUE, validated | User's unique handle   |
| `display_name` | `Nullable<Text>`  |                   | Display name           |
| `bio`          | `Nullable<Text>`  |                   | Biography              |
| `avatar_data`  | `Nullable<Blob>`  |                   | Avatar image data      |
| `header_data`  | `Nullable<Blob>`  |                   | Header / banner data   |
| `created_at`   | `UINT64`          |                   | Account creation time  |
| `updated_at`   | `UINT64`          |                   | Last profile update    |

### `statuses` Table

See the [Status Content](../specs/status.md) spec for full validation
rules on `content`, `spoiler_text`, and `in_reply_to_uri`.

| Column            | Type               | Constraint         | Description                                     |
| :---------------- | :----------------- | :----------------- | :---------------------------------------------- |
| `id`              | `UINT64`           | PRIMARY KEY        | [Snowflake ID](../specs/snowflake.md)           |
| `content`         | `TEXT`             | validated          | Status body                                     |
| `visibility`      | `Visibility`       |                    | `Public`, `Unlisted`, `FollowersOnly`, `Direct` |
| `like_count`      | `UINT64`           |                    | Cached `Like` count                             |
| `boost_count`     | `UINT64`           |                    | Cached `Announce` (boost) count                 |
| `in_reply_to_uri` | `Nullable<Text>`   | INDEX, validated   | URI of the replied-to status                    |
| `spoiler_text`    | `Nullable<Text>`   | sanitized, validated | Optional content warning / spoiler            |
| `sensitive`       | `Boolean`          |                    | Whether clients should hide behind a CW         |
| `edited_at`       | `Nullable<Uint64>` |                    | Timestamp of the last edit                      |
| `created_at`      | `UINT64`           | INDEX              | Creation timestamp (indexed for feed ordering)  |

### `inbox` Table

Stores inbound ActivityPub activities.

| Column                | Type             | Constraint  | Description                                      |
| :-------------------- | :--------------- | :---------- | :----------------------------------------------- |
| `id`                  | `UINT64`         | PRIMARY KEY | [Snowflake ID](../specs/snowflake.md)            |
| `activity_type`       | `ActivityType`   |             | Activity discriminator (`Create`, `Follow`, ...) |
| `actor_uri`           | `TEXT`           | validated   | Originating actor's URI                          |
| `object_data`         | `JSON`           |             | Activity object payload                          |
| `is_boost`            | `Boolean`        |             | `true` when entry is an `Announce` (boost)       |
| `original_status_uri` | `Nullable<Text>` | validated   | URI of the boosted status                        |
| `created_at`          | `UINT64`         | INDEX       | Reception timestamp (indexed for feed ordering)  |

### `follow_requests` Table

| Column       | Type     | Constraint  | Description                              |
| :----------- | :------- | :---------- | :--------------------------------------- |
| `actor_uri`  | `TEXT`   | PRIMARY KEY | Requester's actor URI                    |
| `created_at` | `UINT64` |             | Timestamp when follow request received   |

### `followers` Table

| Column       | Type     | Constraint  | Description                    |
| :----------- | :------- | :---------- | :----------------------------- |
| `actor_uri`  | `TEXT`   | PRIMARY KEY | Follower's actor URI           |
| `created_at` | `UINT64` |             | Timestamp when follow accepted |

### `following` Table

| Column       | Type           | Constraint  | Description                          |
| :----------- | :------------- | :---------- | :----------------------------------- |
| `actor_uri`  | `TEXT`         | PRIMARY KEY | Followed actor's URI                 |
| `status`     | `FollowStatus` |             | `Pending` or `Accepted` (rejected entries are deleted) |
| `created_at` | `UINT64`       |             | Timestamp when follow was requested  |

### `liked` Table

| Column       | Type     | Constraint             | Description                         |
| :----------- | :------- | :--------------------- | :---------------------------------- |
| `status_uri` | `TEXT`   | PRIMARY KEY, validated | URI of the liked status             |
| `created_at` | `UINT64` |                        | Timestamp when the status was liked |

### `blocks` Table

| Column       | Type     | Constraint             | Description                    |
| :----------- | :------- | :--------------------- | :----------------------------- |
| `actor_uri`  | `TEXT`   | PRIMARY KEY, validated | URI of the blocked actor       |
| `created_at` | `UINT64` |                        | Timestamp when block was added |

### `mutes` Table

| Column       | Type     | Constraint             | Description                   |
| :----------- | :------- | :--------------------- | :---------------------------- |
| `actor_uri`  | `TEXT`   | PRIMARY KEY, validated | URI of the muted actor        |
| `created_at` | `UINT64` |                        | Timestamp when mute was added |

### `bookmarks` Table

| Column       | Type     | Constraint             | Description                     |
| :----------- | :------- | :--------------------- | :------------------------------ |
| `status_uri` | `TEXT`   | PRIMARY KEY, validated | URI of the bookmarked status    |
| `created_at` | `UINT64` |                        | Timestamp when bookmark was set |

### `boosts` Table

Tracks `Announce` activities emitted by the user. Each boost is
paired with a wrapper row in the `statuses` table.

| Column                | Type     | Constraint                    | Description                           |
| :-------------------- | :------- | :---------------------------- | :------------------------------------ |
| `id`                  | `UINT64` | PRIMARY KEY                   | [Snowflake ID](../specs/snowflake.md) |
| `status_id`           | `UINT64` | FK → `statuses.id`            | Wrapper status row                    |
| `original_status_uri` | `TEXT`   | validated                     | URI of the boosted status             |
| `created_at`          | `UINT64` | INDEX                         | Timestamp when the boost was emitted  |

The same Snowflake is reused as `boosts.id`, `boosts.status_id`, the
wrapper `statuses.id`, and the wrapper's `feed.id` — making the
wrapper status URL `<actor>/statuses/<snowflake>` also the canonical
`id` of the emitted `Announce` activity. One sequence increment per
boost; one URL that dereferences both the wrapper status and the boost
activity.

### `media` Table

See the [Media Attachments](../specs/media.md) spec for full validation
rules on `media_type`, `description`, and `blurhash`.

| Column        | Type             | Constraint                | Description                           |
| :------------ | :--------------- | :------------------------ | :------------------------------------ |
| `id`          | `UINT64`         | PRIMARY KEY               | [Snowflake ID](../specs/snowflake.md) |
| `status_id`   | `UINT64`         | FK → `statuses.id`, INDEX | Parent status                         |
| `media_type`  | `TEXT`           | validated                 | MIME-like media type                  |
| `description` | `Nullable<Text>` | sanitized, validated      | Alt-text description                  |
| `blurhash`    | `Nullable<Text>` | validated                 | Blurhash preview string               |
| `bytes`       | `BLOB`           |                           | Raw media bytes                       |
| `created_at`  | `UINT64`         |                           | Creation timestamp                    |

### `edit_history` Table

`previous_spoiler_text` uses the same sanitizer/validator pair as
`statuses.spoiler_text`; see the [Status Content](../specs/status.md)
spec.

| Column                  | Type             | Constraint                | Description                           |
| :---------------------- | :--------------- | :------------------------ | :------------------------------------ |
| `id`                    | `UINT64`         | PRIMARY KEY               | [Snowflake ID](../specs/snowflake.md) |
| `status_id`             | `UINT64`         | FK → `statuses.id`, INDEX | Status this entry belongs to          |
| `previous_content`      | `TEXT`           |                           | Content before the edit               |
| `previous_spoiler_text` | `Nullable<Text>` | sanitized, validated      | Spoiler text before the edit          |
| `edited_at`             | `UINT64`         | INDEX                     | Timestamp of the edit                 |

### `hashtags` Table

Local per-user index of hashtags referenced by the user's statuses.

| Column       | Type     | Constraint         | Description                                 |
| :----------- | :------- | :----------------- | :------------------------------------------ |
| `id`         | `UINT64` | PRIMARY KEY        | [Snowflake ID](../specs/snowflake.md)       |
| `tag`        | `TEXT`   | UNIQUE, validated  | Sanitized, lowercase tag (without `#`)      |
| `created_at` | `UINT64` |                    | Timestamp when the hashtag was first seen   |

The `tag` column uses `HashtagSanitizer` (trims whitespace, lowercases,
strips leading `#`) and `HashtagValidator`. See the
[Hashtag Validation](../specs/hashtags.md) page for the full
specification.

### `status_hashtags` Table

Join table between `statuses` and `hashtags`. Uses a surrogate `id`
primary key because the underlying storage layer does not support
composite primary keys; uniqueness of `(status_id, hashtag_id)` is
enforced by the application layer.

| Column       | Type     | Constraint                | Description                           |
| :----------- | :------- | :------------------------ | :------------------------------------ |
| `id`         | `UINT64` | PRIMARY KEY               | [Snowflake ID](../specs/snowflake.md) |
| `status_id`  | `UINT64` | FK → `statuses.id`, INDEX | Status                                |
| `hashtag_id` | `UINT64` | FK → `hashtags.id`, INDEX | Hashtag                               |

### `featured_tags` Table

Up to four hashtags featured on the user's profile. The `tag` column
uses the same [`HashtagSanitizer` / `HashtagValidator`](../specs/hashtags.md)
pair as the `hashtags` table.

| Column       | Type     | Constraint                   | Description                     |
| :----------- | :------- | :--------------------------- | :------------------------------ |
| `tag`        | `TEXT`   | PRIMARY KEY, validated       | Sanitized, lowercase tag        |
| `position`   | `UINT8`  | UNIQUE (`0`..=`3`)           | Display position                |
| `created_at` | `UINT64` |                              | Timestamp when tag was featured |

### `pinned_statuses` Table

Up to five statuses pinned on the user's profile.

| Column      | Type     | Constraint                 | Description                     |
| :---------- | :------- | :------------------------- | :------------------------------ |
| `status_id` | `UINT64` | PRIMARY KEY, FK → `statuses.id` | Pinned status              |
| `position`  | `UINT8`  | UNIQUE (`0`..=`4`)         | Display position                |
| `pinned_at` | `UINT64` |                            | Timestamp when status was pinned |

### `profile_metadata` Table

Up to four custom fields shown on the user's profile. See the
[Profile Metadata](../specs/profile-metadata.md) spec for full
validation rules.

| Column     | Type    | Constraint                | Description         |
| :--------- | :------ | :------------------------ | :------------------ |
| `position` | `UINT8` | PRIMARY KEY (`0`..=`3`)   | Position in the list |
| `name`     | `TEXT`  | sanitized, validated      | Field name          |
| `value`    | `TEXT`  | sanitized, validated      | Field value         |

## Custom Data Types

The following custom types are used in the schema and stored as compact
single-byte discriminants:

### `Visibility`

Maps to [`did::common::Visibility`](./types.md).

| Value | Variant         |
| :---- | :-------------- |
| `0`   | `Public`        |
| `1`   | `Unlisted`      |
| `2`   | `FollowersOnly` |
| `3`   | `Direct`        |

### `ActivityType`

Maps to `activitypub::ActivityType`.

| Value | Variant    |
| :---- | :--------- |
| `0`   | `Create`   |
| `1`   | `Update`   |
| `2`   | `Delete`   |
| `3`   | `Follow`   |
| `4`   | `Accept`   |
| `5`   | `Reject`   |
| `6`   | `Like`     |
| `7`   | `Announce` |
| `8`   | `Undo`     |
| `9`   | `Block`    |
| `10`  | `Add`      |
| `11`  | `Remove`   |
| `12`  | `Flag`     |
| `13`  | `Move`     |

### `FollowStatus`

| Value | Variant    |
| :---- | :--------- |
| `0`   | `Pending`  |
| `1`   | `Accepted` |

### `ReportState`

| Value | Variant     |
| :---- | :---------- |
| `0`   | `Open`      |
| `1`   | `Resolved`  |
| `2`   | `Dismissed` |

## Persistence

All tables are created during canister `init`. Data survives canister upgrades
because `wasm-dbms` stores everything in stable memory. The `post_upgrade`
function does not need to re-register the schema.
