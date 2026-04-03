//! Snowflake ID

use std::cell::RefCell;
use std::fmt;

use wasm_dbms_api::prelude::Uint64;

const MASTIC_EPOCH_MS: u64 = 1_767_225_600_000; // 2026-01-01T00:00:00Z

thread_local! {
    /// The last timestamp used for generating a Snowflake ID.
    static LAST_TIMESTAMP_MS: RefCell<u64> = const { RefCell::new(0) };

    /// The sequence number for the current millisecond.
    static SEQUENCE: RefCell<u16> = const { RefCell::new(0) };
}

/// A Mastic Snowflake ID is a `u64` with the following structure:
///
/// | Bits  | Width | Field     | Description                                      |
/// | ----- | ----- | --------- | ------------------------------------------------ |
/// | 63-16 | 48    | Timestamp | Milliseconds since the Mastic epoch              |
/// | 15-0  | 16    | Sequence  | Per-millisecond monotonic counter (0-65 535)     |
///
/// **Total**: 48 + 16 = 64 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Snowflake(u64);

impl fmt::Display for Snowflake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<u64> for Snowflake {
    fn from(value: u64) -> Self {
        Snowflake(value)
    }
}

impl From<Snowflake> for u64 {
    fn from(snowflake: Snowflake) -> Self {
        snowflake.0
    }
}

impl From<Uint64> for Snowflake {
    fn from(value: Uint64) -> Self {
        Snowflake(value.0)
    }
}

impl From<Snowflake> for Uint64 {
    fn from(snowflake: Snowflake) -> Self {
        Uint64(snowflake.0)
    }
}

impl Snowflake {
    /// Generates a new [`Snowflake`] ID using the following algorithm:
    ///
    /// ```txt
    /// 1. current_ms = [`ic_utils::now`]
    /// 2. timestamp  = current_ms - MASTIC_EPOCH_MS
    /// 3. if timestamp == last_timestamp_ms:
    ///        sequence += 1
    ///        if sequence > `0xFFFF`:
    ///            trap("Snowflake sequence overflow")
    ///    else:
    ///       sequence = 0
    ///       last_timestamp_ms = timestamp
    /// 4. id = (timestamp << 16) | sequence
    /// ```
    pub fn new() -> Self {
        let current_unix_ms = ic_utils::now();
        let timestamp_ms = current_unix_ms.saturating_sub(MASTIC_EPOCH_MS); // Handle clock going backwards by saturating at 0
        let last_timestamp_ms = LAST_TIMESTAMP_MS.with_borrow(|ms| *ms);

        let sequence = if timestamp_ms == last_timestamp_ms {
            SEQUENCE.with_borrow_mut(|seq| {
                match seq.checked_add(1) {
                    Some(new_seq) => {
                        *seq = new_seq;
                        new_seq
                    }
                    None => {
                        // it's okay to trap to give backpressure to clients,
                        // since this is an extremely unlikely edge case that only occurs if the canister receives more than 65 535 ID generation requests in the same millisecond
                        ic_utils::trap!("Snowflake sequence overflow");
                    }
                }
            })
        } else {
            // set sequence to zero
            SEQUENCE.with_borrow_mut(|seq| *seq = 0);
            // update last_timestamp_ms
            LAST_TIMESTAMP_MS.with_borrow_mut(|ms| *ms = timestamp_ms);
            0
        };
        Snowflake((timestamp_ms << 16) | sequence as u64)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Resets thread-local state so tests are independent of execution order.
    fn reset_state() {
        LAST_TIMESTAMP_MS.with_borrow_mut(|ms| *ms = 0);
        SEQUENCE.with_borrow_mut(|seq| *seq = 0);
    }

    // M-UNIT-TEST: conversion from u64 round-trips through Snowflake.
    #[test]
    fn test_from_u64_round_trip() {
        let raw: u64 = 0xDEAD_BEEF_CAFE_1234;
        let snowflake = Snowflake::from(raw);
        let back: u64 = snowflake.into();
        assert_eq!(back, raw);
    }

    // M-UNIT-TEST: conversion from Uint64 round-trips through Snowflake.
    #[test]
    fn test_from_uint64_round_trip() {
        let raw = Uint64(0x1234_5678_9ABC_DEF0);
        let snowflake = Snowflake::from(raw);
        let back: Uint64 = snowflake.into();
        assert_eq!(back.0, raw.0);
    }

    // M-UNIT-TEST: Display renders the inner u64 value.
    #[test]
    fn test_display() {
        let snowflake = Snowflake(42);
        assert_eq!(snowflake.to_string(), "42");
    }

    // M-UNIT-TEST: Snowflake ordering follows u64 ordering.
    #[test]
    fn test_ordering() {
        let a = Snowflake(1);
        let b = Snowflake(2);
        assert!(a < b);
        assert!(b > a);
        assert_eq!(Snowflake(5), Snowflake(5));
    }

    // M-UNIT-TEST: new() produces a snowflake with sequence 0 on first call.
    #[test]
    fn test_new_first_call_sequence_zero() {
        reset_state();
        let snowflake = Snowflake::new();
        let raw: u64 = snowflake.into();
        // sequence occupies the lower 16 bits; first call should be 0
        assert_eq!(raw & 0xFFFF, 0);
    }

    // M-UNIT-TEST: new() increments sequence when timestamp matches.
    #[test]
    fn test_new_same_timestamp_increments_sequence() {
        reset_state();
        // Pre-set the last timestamp to match what now() will return.
        let current_ms = ic_utils::now();
        let timestamp_ms = current_ms.saturating_sub(MASTIC_EPOCH_MS);
        LAST_TIMESTAMP_MS.with_borrow_mut(|ms| *ms = timestamp_ms);
        SEQUENCE.with_borrow_mut(|seq| *seq = 0);

        let snowflake = Snowflake::new();
        let raw: u64 = snowflake.into();
        // sequence should have been incremented from 0 to 1
        assert_eq!(raw & 0xFFFF, 1);
    }

    // M-UNIT-TEST: new() resets sequence to 0 when timestamp differs.
    #[test]
    fn test_new_different_timestamp_resets_sequence() {
        reset_state();
        // Set last timestamp to a value that won't match current time.
        LAST_TIMESTAMP_MS.with_borrow_mut(|ms| *ms = 1);
        SEQUENCE.with_borrow_mut(|seq| *seq = 42);

        let snowflake = Snowflake::new();
        let raw: u64 = snowflake.into();
        // sequence should have been reset to 0
        assert_eq!(raw & 0xFFFF, 0);
        // last_timestamp_ms should have been updated
        let last = LAST_TIMESTAMP_MS.with_borrow(|ms| *ms);
        assert!(last > 1);
    }

    // M-UNIT-TEST: new() embeds the correct timestamp in the upper 48 bits.
    #[test]
    fn test_new_timestamp_encoding() {
        reset_state();
        let before_ms = ic_utils::now().saturating_sub(MASTIC_EPOCH_MS);
        let snowflake = Snowflake::new();
        let after_ms = ic_utils::now().saturating_sub(MASTIC_EPOCH_MS);
        let raw: u64 = snowflake.into();
        let encoded_ts = raw >> 16;
        assert!(encoded_ts >= before_ms);
        assert!(encoded_ts <= after_ms);
    }

    // M-UNIT-TEST: new() panics on sequence overflow (u16::MAX reached).
    #[test]
    #[should_panic(expected = "Snowflake sequence overflow")]
    fn test_new_sequence_overflow_traps() {
        reset_state();
        // Pre-set the last timestamp to match what now() will return and
        // set sequence to u16::MAX so the next increment overflows.
        let current_ms = ic_utils::now();
        let timestamp_ms = current_ms.saturating_sub(MASTIC_EPOCH_MS);
        LAST_TIMESTAMP_MS.with_borrow_mut(|ms| *ms = timestamp_ms);
        SEQUENCE.with_borrow_mut(|seq| *seq = u16::MAX);

        Snowflake::new();
    }
}
