# Handle Validation

This document defines the validation rules for user handles in Mastic.
These rules follow the [Mastodon username conventions](https://docs.joinmastodon.org/) to ensure
compatibility with the broader fediverse.

## Local Handle Format

A **local handle** is the username portion of a Mastic account (e.g. `alice` in `@alice@mastic.social`).

| Rule               | Value                  |
| :----------------- | :--------------------- |
| Allowed characters | `a-z`, `0-9`, `_`      |
| Minimum length     | 1                      |
| Maximum length     | 30                     |
| Case sensitivity   | Case-insensitive       |
| Storage            | Stored as lowercase    |

**Regex**: `^[a-z0-9_]{1,30}$`

Underscores are allowed in any position, including leading, trailing, and
consecutive.

Local handles are intentionally restricted to ASCII. Mastic owns the
local namespace and the `users.handle` column carries a unique index
that must remain free of Unicode normalization ambiguity (NFC vs NFD,
confusables, IDN homograph). Users on other fediverse instances may
have Unicode handles — see the next section.

## Remote Handle Format

A **remote handle** identifies a user on another fediverse instance
(e.g. `@bob@mastodon.social`, `@user@ꩰ.com`). Remote handles are
discovered via WebFinger ([RFC 7033](https://tools.ietf.org/html/rfc7033))
using the `acct:` URI scheme defined by
[RFC 7565](https://tools.ietf.org/html/rfc7565), where both the
userpart and the host are UTF-8.

To remain federation-compatible with Mastodon, Pleroma, Misskey,
GoToSocial, and other ActivityPub implementations — many of which allow
Unicode usernames and IDN domains — Mastic accepts a broader character
set for remote handles.

### Userpart

| Rule               | Value                                              |
| :----------------- | :------------------------------------------------- |
| Allowed characters | Unicode letters, Unicode numbers, `_`, `.`, `-`    |
| Minimum length     | 1                                                  |
| Maximum length     | 64 Unicode scalar values                           |
| Case sensitivity   | Compared case-insensitively (Unicode-aware fold)   |

**Pattern** (PCRE-style): `^[\p{L}\p{N}_.\-]{1,64}$`.

Punctuation other than `_`, `.`, `-` is rejected. Whitespace and control
characters are rejected.

### Host (domain)

The host is treated as an
[Internationalized Domain Name](https://datatracker.ietf.org/doc/html/rfc5890)
(IDN). Unicode domains such as `ꩰ.com` are accepted; for storage and
comparison the host is normalized to its ASCII Compatible Encoding
(Punycode, [UTS #46](https://www.unicode.org/reports/tr46/)).

| Rule               | Value                                              |
| :----------------- | :------------------------------------------------- |
| Input form         | U-label (Unicode) or A-label (Punycode/ACE)        |
| Stored form        | A-label (lowercase, ASCII)                         |
| Maximum length     | 253 octets in A-label form (DNS limit)             |

### Storage

Remote handles are stored in canonical form
`<unicode_userpart>@<ace_domain>`, with the userpart Unicode-lowercased
and the domain in lowercase A-label form. Two handles compare equal iff
their canonical forms are byte-equal.

> **Implementation note.** Mastic does not yet perform IDN normalization
> at runtime; remote handles arriving with Unicode hosts are accepted
> but stored as received. Punycode normalization will be wired into the
> federation canister alongside WebFinger lookup (tracked separately).

## Reserved Handles

The following handles are reserved and cannot be claimed during sign-up.
These match system routes and well-known service names commonly reserved
across fediverse implementations.

| Handle          | Reason             |
| :-------------- | :----------------- |
| `admin`         | System role        |
| `administrator` | System role        |
| `autoconfig`    | Service discovery  |
| `autodiscover`  | Service discovery  |
| `help`          | System route       |
| `hostmaster`    | Service role       |
| `info`          | System route       |
| `mailer-daemon` | Email service      |
| `postmaster`    | Email service      |
| `root`          | System role        |
| `ssladmin`      | Certificate admin  |
| `support`       | System route       |
| `webmaster`     | Service role       |
