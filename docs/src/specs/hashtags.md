# Hashtag Validation

This document defines the validation rules for hashtags in Mastic. These
rules follow the [Mastodon hashtag conventions](https://docs.joinmastodon.org/)
to ensure compatibility with the broader fediverse.

Hashtags are stored in two User Canister tables — `hashtags` (local
per-user index) and `featured_tags` (profile-featured list) — and are
materialized into the `status_hashtags` join table when a status is
published. A future Hashtag Canister (WI-1.28) will aggregate hashtags
across the instance.

## Tag Format

A **tag** is the text portion of a hashtag, stripped of the leading `#`.
For example, the hashtag `#rust` has tag `rust`.

| Rule               | Value                       |
| :----------------- | :-------------------------- |
| Allowed characters | `a-z`, `0-9`, `_`           |
| Minimum length     | 1                           |
| Maximum length     | 30 Unicode scalar values    |
| Case sensitivity   | Case-insensitive            |
| Storage            | Stored as lowercase, no `#` |

**Regex**: `^[a-z0-9_]{1,30}$`

Underscores are allowed in any position, including leading, trailing, and
consecutive.

Hyphens (`-`), dots (`.`), whitespace, and any non-ASCII characters are
**not** allowed. Strict ASCII enforcement keeps the uniqueness constraint
on the `hashtags.tag` column well-defined and avoids Unicode
normalization ambiguity.

## Sanitization

Input is sanitized before validation:

1. Leading and trailing whitespace is trimmed.
2. The string is lowercased.
3. A single leading `#`, if present, is stripped.

`#` characters in any other position are **not** stripped and will cause
validation to fail.

### Examples

| Input          | Sanitized | Valid |
| :------------- | :-------- | :---- |
| `rust`         | `rust`    | yes   |
| `Rust`         | `rust`    | yes   |
| `  #Rust  `    | `rust`    | yes   |
| `web3`         | `web3`    | yes   |
| `rust_lang`    | `rust_lang` | yes |
| `rust-lang`    | `rust-lang` | no (hyphen) |
| `#rust#2`      | `rust#2`  | no (`#` in middle) |
| `` (empty)     | ``        | no (too short) |
| `a` × 31       | `a` × 31  | no (too long) |

## Implementation

The rules are enforced by `HashtagSanitizer` and `HashtagValidator` in
the `db-utils` crate. Both are attached to every `Text` column holding
a tag (`hashtags.tag`, `featured_tags.tag`). `status_hashtags` does not
store a tag string directly — it references `hashtags.id` — so no
validator is needed there.
