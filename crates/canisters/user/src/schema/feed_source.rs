//! Database custom type for distinguishing feed entry sources.

use std::fmt;

use serde::{Deserialize, Serialize};
use wasm_dbms_api::memory::Encode;
use wasm_dbms_api::prelude::*;

/// Identifies whether a feed entry originated from the user's own outbox
/// or from a received inbox activity.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    CustomDataType,
)]
#[type_tag = "feed_source"]
pub enum FeedSource {
    /// The feed entry is the user's own published status.
    #[default]
    Outbox,
    /// The feed entry is a received activity from a followed user.
    Inbox,
}

impl fmt::Display for FeedSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let source_str = match self {
            Self::Outbox => "outbox",
            Self::Inbox => "inbox",
        };
        write!(f, "{}", source_str)
    }
}

impl Encode for FeedSource {
    const ALIGNMENT: PageOffset = 1;

    const SIZE: DataSize = DataSize::Fixed(1);

    fn encode(&'_ self) -> std::borrow::Cow<'_, [u8]> {
        std::borrow::Cow::Owned(vec![match self {
            Self::Outbox => 0,
            Self::Inbox => 1,
        }])
    }

    fn decode(data: std::borrow::Cow<[u8]>) -> MemoryResult<Self>
    where
        Self: Sized,
    {
        if data.is_empty() {
            return Err(MemoryError::DecodeError(DecodeError::TooShort));
        }

        let byte = data[0];
        match byte {
            0 => Ok(Self::Outbox),
            1 => Ok(Self::Inbox),
            _ => Err(MemoryError::DecodeError(DecodeError::InvalidDiscriminant(
                byte,
            ))),
        }
    }

    fn size(&self) -> MSize {
        Self::SIZE.get_fixed_size().unwrap() as MSize
    }
}

impl DataType for FeedSource {}

#[cfg(test)]
mod tests {

    use std::borrow::Cow;

    use super::*;

    #[test]
    fn test_default_is_outbox() {
        assert_eq!(FeedSource::default(), FeedSource::Outbox);
    }

    #[test]
    fn test_display() {
        assert_eq!(FeedSource::Outbox.to_string(), "outbox");
        assert_eq!(FeedSource::Inbox.to_string(), "inbox");
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let cases = [FeedSource::Outbox, FeedSource::Inbox];

        for source in cases {
            let encoded = source.encode();
            let decoded = FeedSource::decode(encoded).unwrap();
            assert_eq!(source, decoded);
        }
    }

    #[test]
    fn test_encode_values() {
        assert_eq!(FeedSource::Outbox.encode().as_ref(), &[0]);
        assert_eq!(FeedSource::Inbox.encode().as_ref(), &[1]);
    }

    #[test]
    fn test_decode_empty_data_returns_error() {
        let result = FeedSource::decode(Cow::Borrowed(&[]));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_discriminant_returns_error() {
        for byte in [2u8, 3, 4, 10, 255] {
            let result = FeedSource::decode(Cow::Borrowed(&[byte]));
            assert!(result.is_err(), "expected error for discriminant {byte}");
        }
    }

    #[test]
    fn test_size_is_one() {
        let s = FeedSource::default();
        assert_eq!(s.size(), 1);
    }

    #[test]
    fn test_alignment_is_one() {
        assert_eq!(FeedSource::ALIGNMENT, 1);
    }

    #[test]
    fn test_size_is_fixed() {
        assert_eq!(FeedSource::SIZE, DataSize::Fixed(1));
    }

    #[test]
    fn test_ord() {
        assert!(FeedSource::Outbox < FeedSource::Inbox);
    }

    #[test]
    fn test_clone_and_copy() {
        let s = FeedSource::Inbox;
        let copied = s;
        assert_eq!(s, copied);
    }

    #[test]
    fn test_debug() {
        let debug_str = format!("{:?}", FeedSource::Outbox);
        assert!(debug_str.contains("Outbox"));
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(FeedSource::Outbox);
        set.insert(FeedSource::Outbox);
        assert_eq!(set.len(), 1);

        set.insert(FeedSource::Inbox);
        assert_eq!(set.len(), 2);
    }
}
