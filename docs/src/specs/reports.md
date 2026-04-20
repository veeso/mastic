# Reports

The `reports` table in the Directory Canister stores moderation reports
submitted by users. This document defines the validation rules for
report columns.

## `reason`

Free-form text supplied by the reporter.

| Rule           | Value                            |
| :------------- | :------------------------------- |
| Sanitization   | Trim leading/trailing whitespace |
| Maximum length | 1000 characters                  |
| Minimum length | 1 (empty string rejected)        |
| Nullable       | No                               |
| Length unit    | Unicode scalar values            |

Enforced by `TrimSanitizer` + `BoundedTextValidator(1000)` in `db-utils`.

## `target_status_uri`

Optional URI of the specific status being reported. When null, the
report targets the user's account as a whole.

| Rule     | Value                           |
| :------- | :------------------------------ |
| Format   | Valid URL (per the `url` crate) |
| Nullable | Yes                             |

Enforced by `NullableUrlValidator` in `db-utils`.

## `state`

`Open` → submitted, awaiting moderator review.
`Resolved` → moderator took action.
`Dismissed` → moderator reviewed and declined to act.

See [`ReportState`](../architecture/database-schema.md#reportstate)
in the database schema reference for the on-disk encoding.

## `reporter`, `target_canister`, `resolved_by`

Typed `Principal` columns. Validated by the candid/type layer; no
schema-level validator.

## `created_at`, `resolved_at`

`UINT64` timestamps. `created_at` is indexed for recent-first
listing. `resolved_at` is null until state transitions out of `Open`.
