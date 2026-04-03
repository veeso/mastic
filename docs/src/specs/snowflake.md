# Snowflake IDs

Mastic uses **Snowflake IDs** as unique identifiers for statuses and other
user-generated entities. A Snowflake ID is a 64-bit unsigned integer that
encodes a creation timestamp, making IDs roughly sortable by time without
requiring a secondary index.

## Bit Layout

A Mastic Snowflake ID is a `u64` with the following structure:

| Bits  | Width | Field     | Description                                      |
| ----- | ----- | --------- | ------------------------------------------------ |
| 63-16 | 48    | Timestamp | Milliseconds since the Mastic epoch              |
| 15-0  | 16    | Sequence  | Per-millisecond monotonic counter (0-65 535)     |

**Total**: 48 + 16 = 64 bits.

## Epoch

The Mastic epoch is **2026-01-01T00:00:00Z** (Unix timestamp 1 767 225 600 000
ms).

With 48 bits of millisecond precision the timestamp space covers approximately
**8 919 years**, which is more than sufficient even when measured from the Unix
epoch (1970). The custom Mastic epoch is therefore not strictly necessary for
correctness, but subtracting it produces smaller numeric values in the early
years of the platform, resulting in shorter IDs when serialised as decimal
strings in URLs and JSON-LD payloads.

## Generation

Each User Canister maintains its own Snowflake generator with:

- `last_timestamp_ms`: the timestamp of the last generated ID.
- `sequence`: a 16-bit counter, reset to 0 whenever `last_timestamp_ms`
  advances.

### Algorithm

```text
1. current_ms = ic_cdk::api::time() / 1_000_000  (nanoseconds to milliseconds)
2. timestamp  = current_ms - MASTIC_EPOCH_MS
3. if timestamp == last_timestamp_ms:
       sequence += 1
       if sequence > 0xFFFF:
           trap("Snowflake sequence overflow")
   else:
       sequence = 0
       last_timestamp_ms = timestamp
4. id = (timestamp << 16) | sequence
```

### Properties

- **Uniqueness**: guaranteed within a single canister because the sequence
  counter prevents collisions within the same millisecond, and the IC
  provides monotonic time.
- **Global uniqueness**: achieved because the full ActivityPub `id` is a URL
  that includes the user handle, e.g.
  `https://{domain}/users/{handle}/statuses/{snowflake}`.
- **Sortability**: IDs are monotonically increasing within a canister and
  roughly chronologically ordered across canisters.

## No Worker Bits

Unlike the original Twitter Snowflake or Mastodon's variant, Mastic does
**not** include worker/node bits. Each User Canister is the sole generator of
its own IDs, so there is no risk of cross-worker collision. This simplifies the
layout and maximises the sequence space.

## Representation

- **On-chain (Candid)**: `nat64`
- **Over ActivityPub (JSON-LD)**: decimal string, e.g. `"116301527915219032"`
- **In URLs**: decimal, e.g. `/users/alice/statuses/116301527915219032`
