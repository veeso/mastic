# Media Attachments

This document defines the validation rules for the `media` table in the
User Canister. Limits follow the
[Mastodon defaults](https://docs.joinmastodon.org/) to ensure
compatibility with the broader fediverse.

## `media_type`

MIME type of the attachment. Enforced by `MimeValidator` in `db-utils`.

| Rule           | Value                                               |
| :------------- | :-------------------------------------------------- |
| Format         | `type/subtype` per [RFC 6838](https://www.rfc-editor.org/rfc/rfc6838) |
| Slash count    | Exactly one `/`                                     |
| Allowed chars  | Lowercase ASCII graphic (`!`..=`~` minus uppercase) |
| Whitespace     | Rejected                                            |
| Maximum length | 127 bytes                                           |
| Nullable       | No                                                  |

Examples accepted: `image/png`, `image/jpeg`, `video/mp4`,
`application/vnd.mastic.v1+json`.
Examples rejected: `Image/png` (uppercase), `image /png` (whitespace),
`image/png/x` (extra slash), `imagepng` (no slash).

## `description`

Alt-text for the attachment.

| Rule           | Value                            |
| :------------- | :------------------------------- |
| Sanitization   | Trim leading/trailing whitespace |
| Maximum length | 1500 characters                  |
| Minimum length | 1 (empty string rejected)        |
| Nullable       | Yes                              |
| Length unit    | Unicode scalar values            |

Enforced by `TrimSanitizer` + `BoundedTextValidator(1500)` in `db-utils`.

## `blurhash`

Compact [blurhash](https://blurha.sh/) preview string. Enforced by
`BlurhashValidator` in `db-utils`.

| Rule           | Value                                                                                            |
| :------------- | :----------------------------------------------------------------------------------------------- |
| Alphabet       | Base83: `0-9`, `A-Z`, `a-z`, `#$%*+,-.:;=?@[]^_{|}~`                                              |
| Minimum length | 6 bytes                                                                                          |
| Maximum length | 128 bytes                                                                                        |
| Nullable       | Yes                                                                                              |

Blurhash length is a function of the encoded component count. The
allowed range covers every valid `componentsX × componentsY` pairing
with a reasonable safety margin on the upper bound to prevent storage
blow-up.

## `bytes`

Raw media payload (`BLOB`). No schema-level validation; size and
content-type enforcement happen at the upload-endpoint layer. See
WI-1.16 (#59) and WI-1.17 (#67) for chunked upload handling.

## `status_id`

Foreign key to `statuses.id`. Enforced by the storage layer.
