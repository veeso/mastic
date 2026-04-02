//! Follow status custom type definition for the `following` table.

use std::fmt;

use serde::{Deserialize, Serialize};
use wasm_dbms_api::memory::Encode;
use wasm_dbms_api::prelude::*;

/// Status of a follow relationship.
///
/// Tracks the lifecycle of a follow request: a `Follow` activity
/// starts in [`Pending`](FollowStatus::Pending), then transitions
/// to [`Accepted`](FollowStatus::Accepted) on `Accept(Follow)` or
/// [`Rejected`](FollowStatus::Rejected) on `Reject(Follow)`.
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
#[type_tag = "follow_status"]
pub enum FollowStatus {
    /// Follow request sent, awaiting response.
    #[default]
    Pending,
    /// Follow request accepted by the remote actor.
    Accepted,
    /// Follow request rejected by the remote actor.
    Rejected,
}

impl fmt::Display for FollowStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        };
        write!(f, "{}", status_str)
    }
}

impl Encode for FollowStatus {
    const ALIGNMENT: PageOffset = 1;

    const SIZE: DataSize = DataSize::Fixed(1);

    fn encode(&'_ self) -> std::borrow::Cow<'_, [u8]> {
        std::borrow::Cow::Owned(vec![match self {
            Self::Pending => 0,
            Self::Accepted => 1,
            Self::Rejected => 2,
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
            0 => Ok(Self::Pending),
            1 => Ok(Self::Accepted),
            2 => Ok(Self::Rejected),
            _ => Err(MemoryError::DecodeError(DecodeError::InvalidDiscriminant(
                byte,
            ))),
        }
    }

    fn size(&self) -> MSize {
        Self::SIZE.get_fixed_size().unwrap() as MSize
    }
}

impl DataType for FollowStatus {}

#[cfg(test)]
mod tests {

    use std::borrow::Cow;

    use super::*;

    #[test]
    fn test_default_is_pending() {
        assert_eq!(FollowStatus::default(), FollowStatus::Pending);
    }

    #[test]
    fn test_display() {
        assert_eq!(FollowStatus::Pending.to_string(), "pending");
        assert_eq!(FollowStatus::Accepted.to_string(), "accepted");
        assert_eq!(FollowStatus::Rejected.to_string(), "rejected");
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let cases = [
            FollowStatus::Pending,
            FollowStatus::Accepted,
            FollowStatus::Rejected,
        ];

        for status in cases {
            let encoded = status.encode();
            let decoded = FollowStatus::decode(encoded).unwrap();
            assert_eq!(status, decoded);
        }
    }

    #[test]
    fn test_encode_values() {
        assert_eq!(FollowStatus::Pending.encode().as_ref(), &[0]);
        assert_eq!(FollowStatus::Accepted.encode().as_ref(), &[1]);
        assert_eq!(FollowStatus::Rejected.encode().as_ref(), &[2]);
    }

    #[test]
    fn test_decode_empty_data_returns_error() {
        let result = FollowStatus::decode(Cow::Borrowed(&[]));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_discriminant_returns_error() {
        for byte in [3u8, 4, 10, 255] {
            let result = FollowStatus::decode(Cow::Borrowed(&[byte]));
            assert!(result.is_err(), "expected error for discriminant {byte}");
        }
    }

    #[test]
    fn test_size_is_one() {
        let s = FollowStatus::default();
        assert_eq!(s.size(), 1);
    }

    #[test]
    fn test_alignment_is_one() {
        assert_eq!(FollowStatus::ALIGNMENT, 1);
    }

    #[test]
    fn test_size_is_fixed() {
        assert_eq!(FollowStatus::SIZE, DataSize::Fixed(1));
    }

    #[test]
    fn test_ord() {
        assert!(FollowStatus::Pending < FollowStatus::Accepted);
        assert!(FollowStatus::Accepted < FollowStatus::Rejected);
    }

    #[test]
    fn test_clone_and_copy() {
        let s = FollowStatus::Accepted;
        let copied = s;
        assert_eq!(s, copied);
    }

    #[test]
    fn test_debug() {
        let debug_str = format!("{:?}", FollowStatus::Pending);
        assert!(debug_str.contains("Pending"));
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(FollowStatus::Pending);
        set.insert(FollowStatus::Pending);
        assert_eq!(set.len(), 1);

        set.insert(FollowStatus::Accepted);
        assert_eq!(set.len(), 2);
    }
}
