# Database Schema

- [Database Schema](#database-schema)
  - [Shared Settings Table](#shared-settings-table)
  - [Directory Canister](#directory-canister)
    - [Directory Canister Settings Keys](#directory-canister-settings-keys)
    - [`moderators` Table](#moderators-table)
    - [`users` Table](#users-table)
  - [User Canister](#user-canister)
    - [User Canister Settings Keys](#user-canister-settings-keys)
    - [`profiles` Table](#profiles-table)
    - [`statuses` Table](#statuses-table)
    - [`inbox` Table](#inbox-table)
    - [`followers` Table](#followers-table)
    - [`following` Table](#following-table)
  - [Custom Data Types](#custom-data-types)
    - [`Visibility`](#visibility)
    - [`ActivityType`](#activitytype)
    - [`FollowStatus`](#followstatus)
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

## User Canister

### User Canister Settings Keys

| Key | Constant                      | Value Type | Description                          |
| :-- | :---------------------------- | :--------- | :----------------------------------- |
| `0` | `SETTING_FEDERATION_CANISTER` | `BLOB`     | Principal of the Federation canister |
| `1` | `SETTING_OWNER_PRINCIPAL`     | `BLOB`     | Principal of the canister owner      |

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

| Column       | Type         | Constraint  | Description                                     |
| :----------- | :----------- | :---------- | :---------------------------------------------- |
| `id`         | `UINT64`     | PRIMARY KEY | [Snowflake ID](../specs/snowflake.md)           |
| `content`    | `TEXT`       |             | Status body                                     |
| `visibility` | `Visibility` |             | `Public`, `Unlisted`, `FollowersOnly`, `Direct` |
| `created_at` | `UINT64`     | INDEX       | Creation timestamp (indexed for feed ordering)  |

### `inbox` Table

Stores inbound ActivityPub activities.

| Column          | Type           | Constraint  | Description                                      |
| :-------------- | :------------- | :---------- | :----------------------------------------------- |
| `id`            | `UINT64`       | PRIMARY KEY | [Snowflake ID](../specs/snowflake.md)            |
| `activity_type` | `ActivityType` |             | Activity discriminator (`Create`, `Follow`, ...) |
| `actor_uri`     | `TEXT`         | validated   | Originating actor's URI                          |
| `object_data`   | `JSON`         |             | Activity object payload                          |
| `created_at`    | `UINT64`       | INDEX       | Reception timestamp (indexed for feed ordering)  |

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

## Persistence

All tables are created during canister `init`. Data survives canister upgrades
because `wasm-dbms` stores everything in stable memory. The `post_upgrade`
function does not need to re-register the schema.
