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

| Rule               | Value                                                  |
| :----------------- | :----------------------------------------------------- |
| Allowed characters | Unicode letters, Unicode numbers, `_`                  |
| Minimum length     | 1                                                      |
| Maximum length     | 30 Unicode scalar values                               |
| Case sensitivity   | Case-insensitive (Unicode-aware lowercasing)           |
| Storage            | Stored as lowercase, no `#`                            |

**Pattern** (PCRE-style): `^[\p{L}\p{N}_]{1,30}$`, with the additional
constraint that no character may be in uppercase form after sanitization.

Underscores are allowed in any position, including leading, trailing, and
consecutive. Cased Unicode letters (Latin, Greek, Cyrillic, etc.) are
folded to lowercase by the sanitizer; non-cased scripts (Han, Arabic,
Myanmar, etc.) are accepted as-is.

Hyphens (`-`), dots (`.`), whitespace, punctuation, and emoji are **not**
allowed.

### Unicode normalization

Mastic does **not** currently apply Unicode Normalization Form C (NFC) to
incoming tags. Producers SHOULD send tags in NFC form. Two visually
identical tags that differ only in normalization (e.g. precomposed `é`
vs `e + COMBINING ACUTE`) will be treated as distinct values until
normalization is added (tracked separately).

## Sanitization

Input is sanitized before validation:

1. Leading and trailing whitespace is trimmed.
2. The string is lowercased.
3. A single leading `#`, if present, is stripped.

`#` characters in any other position are **not** stripped and will cause
validation to fail.

### Examples

| Input         | Sanitized   | Valid              |
| :------------ | :---------- | :----------------- |
| `rust`        | `rust`      | yes                |
| `Rust`        | `rust`      | yes                |
| `  #Rust  `   | `rust`      | yes                |
| `web3`        | `web3`      | yes                |
| `rust_lang`   | `rust_lang` | yes                |
| `汉字`        | `汉字`      | yes (Han)          |
| `Café`        | `café`      | yes                |
| `Ελληνικά`    | `ελληνικά`  | yes (Greek)        |
| `rust-lang`   | `rust-lang` | no (hyphen)        |
| `#rust#2`     | `rust#2`    | no (`#` in middle) |
| `🦀`          | `🦀`        | no (emoji)         |
| `` (empty)    | ``          | no (too short)     |
| `a` × 31      | `a` × 31    | no (too long)      |

## Implementation

The rules are enforced by `HashtagSanitizer` and `HashtagValidator` in
the `db-utils` crate. Both are attached to every `Text` column holding
a tag (`hashtags.tag`, `featured_tags.tag`). `status_hashtags` does not
store a tag string directly — it references `hashtags.id` — so no
validator is needed there.
