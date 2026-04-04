//! Transaction utilities.

/// Build a transaction caller ID from a timestamp.
///
/// Converts the given `u64` timestamp into its 8-byte big-endian
/// representation, suitable for use as the caller argument to
/// `DbmsContext::begin_transaction`.
pub fn transaction_caller(timestamp: u64) -> Vec<u8> {
    timestamp.to_be_bytes().to_vec()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_transaction_caller_returns_8_bytes() {
        let caller = transaction_caller(1_000_000);
        assert_eq!(caller.len(), 8);
    }

    #[test]
    fn test_transaction_caller_is_big_endian() {
        let caller = transaction_caller(0x0102_0304_0506_0708);
        assert_eq!(caller, vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
    }
}
