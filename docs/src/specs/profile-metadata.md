# Profile Metadata

The `profile_metadata` table stores up to four custom key/value rows
that are shown on the user's profile. This document defines the
validation rules for the `name` and `value` columns. Limits follow the
[Mastodon defaults](https://docs.joinmastodon.org/).

## `position`

| Rule  | Value                        |
| :---- | :--------------------------- |
| Type  | `UINT8`                      |
| Range | `0`..=`3` (primary key)      |

The table is capped at four rows via the primary key range. Enforcement
of the upper bound is done by the application layer that writes to the
table (WI-1.27).

## `name` / `value`

Both columns share the same rules.

| Rule           | Value                            |
| :------------- | :------------------------------- |
| Sanitization   | Trim leading/trailing whitespace |
| Maximum length | 255 characters                   |
| Minimum length | 1 (empty string rejected)        |
| Nullable       | No                               |
| Length unit    | Unicode scalar values            |

Enforced by `TrimSanitizer` + `BoundedTextValidator(255)` in `db-utils`.
