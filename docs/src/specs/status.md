# Status Content

This document defines the validation rules for status content in Mastic.
The maximum content length follows the
[Mastodon default](https://docs.joinmastodon.org/) to ensure compatibility
with the broader fediverse.

## Content Constraints

| Rule             | Value                     |
| :--------------- | :------------------------ |
| Maximum length   | 500 characters            |
| Minimum length   | 1 (empty statuses denied) |
| Encoding         | UTF-8                     |
| Length unit       | Unicode scalar values     |

Statuses whose content exceeds 500 characters are rejected with the
`ContentTooLong` error variant defined in `PublishStatusError`.
