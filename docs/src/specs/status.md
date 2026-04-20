# Status Content

This document defines the validation rules for status fields in Mastic.
Limits follow the [Mastodon defaults](https://docs.joinmastodon.org/)
to ensure compatibility with the broader fediverse.

## `content`

| Rule           | Value                     |
| :------------- | :------------------------ |
| Maximum length | 500 characters            |
| Minimum length | 1 (empty statuses denied) |
| Encoding       | UTF-8                     |
| Length unit    | Unicode scalar values     |

Statuses whose content exceeds 500 characters are rejected with the
`ContentTooLong` error variant defined in `PublishStatusError`.

## `spoiler_text`

Optional content warning / spoiler text shown by clients before the
status body. Applies to both `statuses.spoiler_text` and
`edit_history.previous_spoiler_text`.

| Rule           | Value                          |
| :------------- | :----------------------------- |
| Sanitization   | Trim leading/trailing whitespace |
| Maximum length | 500 characters                 |
| Minimum length | 1 (empty string rejected)      |
| Nullable       | Yes                            |
| Length unit    | Unicode scalar values          |

Enforced by `TrimSanitizer` + `BoundedTextValidator(500)` in `db-utils`.

## `in_reply_to_uri`

URI of the status this one replies to. Threads are resolved by looking
up this URI against local statuses and remote inbox activities.

| Rule     | Value                                      |
| :------- | :----------------------------------------- |
| Format   | Valid URL (per the `url` crate)            |
| Nullable | Yes                                        |

Enforced by `NullableUrlValidator` in `db-utils`.

## `sensitive`

Boolean flag. Clients are expected to hide media and content behind a
"show more" gate when `sensitive = true`, even if `spoiler_text` is
null. No validation beyond type.

## `edited_at`

Nullable `Uint64` timestamp. Written by the edit flow; never set by
clients directly. No validation beyond type.
