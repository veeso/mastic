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

Underscores are allowed in any position, including leading, trailing, and consecutive.

## Remote Handle Format

A **remote handle** identifies a user on another fediverse instance (e.g. `@bob@mastodon.social`).
To maintain federation compatibility with Pleroma, Misskey, GoToSocial, and other ActivityPub
implementations, remote handles accept a broader character set.

| Rule               | Value                       |
| :----------------- | :-------------------------- |
| Allowed characters | `a-z`, `0-9`, `_`, `.`, `-` |
| Minimum length     | 1                           |
| Maximum length     | 30                          |

**Regex**: `^[a-z0-9_.-]{1,30}$`

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
