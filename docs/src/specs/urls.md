# Canonical URL Patterns

All ActivityPub resource URLs in Mastic are built from the instance
`public_url` and the user's handle. These patterns are centralized in
the User Canister's `domain::urls` module to guarantee consistency
across the codebase.

## URL Table

| Pattern                                       | Purpose              |
| --------------------------------------------- | -------------------- |
| `{public_url}/users/{handle}`                 | Actor URI (profile)  |
| `{public_url}/users/{handle}/inbox`           | ActivityPub inbox    |
| `{public_url}/users/{handle}/outbox`          | ActivityPub outbox   |
| `{public_url}/users/{handle}/followers`       | Followers collection |
| `{public_url}/users/{handle}/following`       | Following collection |
| `{public_url}/users/{handle}/statuses/{id}`   | Status URL           |

When a user boosts a status, the wrapper status URL
`<actor>/statuses/<snowflake>` is also the canonical `id` of the
emitted `Announce` activity. The booster's `Boost` row, wrapper
`Status`, `FeedEntry`, and the `Announce` activity all share a single
Snowflake — one URL dereferences both the wrapper status and the boost
activity.

## Example

With `public_url = "https://mastic.social"` and `handle = "alice"`:

- Actor URI: `https://mastic.social/users/alice`
- Inbox: `https://mastic.social/users/alice/inbox`
- Outbox: `https://mastic.social/users/alice/outbox`
- Followers: `https://mastic.social/users/alice/followers`
- Following: `https://mastic.social/users/alice/following`

## Public URL Propagation

The `public_url` is configured at deploy time on the Federation Canister
and the Directory Canister. When the Directory Canister creates a new
User Canister during sign-up, it passes `public_url` in the init args.
Each User Canister stores it in settings and uses it via the
`domain::urls` module.

```text
Deploy ──► Federation Canister (public_url in init args)
       ──► Directory Canister  (public_url in init args)
                │
                ▼  sign_up
           User Canister (public_url passed in init args, stored in settings)
```
