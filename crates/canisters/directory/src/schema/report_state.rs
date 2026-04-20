//! Database custom type representing the lifecycle state of a report.

use std::fmt;

use serde::{Deserialize, Serialize};
use wasm_dbms_api::memory::Encode;
use wasm_dbms_api::prelude::*;

/// Lifecycle state of a user report.
///
/// A report starts in [`Open`](ReportState::Open) when submitted and
/// transitions to [`Resolved`](ReportState::Resolved) if action was
/// taken, or [`Dismissed`](ReportState::Dismissed) if no action was
/// taken by a moderator.
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
#[type_tag = "report_state"]
pub enum ReportState {
    /// Report submitted, pending moderator review.
    #[default]
    Open,
    /// Moderator took action on the report.
    Resolved,
    /// Moderator reviewed the report and decided no action was needed.
    Dismissed,
}

impl fmt::Display for ReportState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Open => "open",
            Self::Resolved => "resolved",
            Self::Dismissed => "dismissed",
        };
        write!(f, "{}", s)
    }
}

impl Encode for ReportState {
    const ALIGNMENT: PageOffset = 1;

    const SIZE: DataSize = DataSize::Fixed(1);

    fn encode(&'_ self) -> std::borrow::Cow<'_, [u8]> {
        std::borrow::Cow::Owned(vec![match self {
            Self::Open => 0,
            Self::Resolved => 1,
            Self::Dismissed => 2,
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
            0 => Ok(Self::Open),
            1 => Ok(Self::Resolved),
            2 => Ok(Self::Dismissed),
            _ => Err(MemoryError::DecodeError(DecodeError::InvalidDiscriminant(
                byte,
            ))),
        }
    }

    fn size(&self) -> MSize {
        Self::SIZE.get_fixed_size().unwrap() as MSize
    }
}

impl DataType for ReportState {}

#[cfg(test)]
mod tests {

    use std::borrow::Cow;

    use super::*;

    #[test]
    fn test_default_is_open() {
        assert_eq!(ReportState::default(), ReportState::Open);
    }

    #[test]
    fn test_display() {
        assert_eq!(ReportState::Open.to_string(), "open");
        assert_eq!(ReportState::Resolved.to_string(), "resolved");
        assert_eq!(ReportState::Dismissed.to_string(), "dismissed");
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let cases = [
            ReportState::Open,
            ReportState::Resolved,
            ReportState::Dismissed,
        ];

        for state in cases {
            let encoded = state.encode();
            let decoded = ReportState::decode(encoded).unwrap();
            assert_eq!(state, decoded);
        }
    }

    #[test]
    fn test_encode_values() {
        assert_eq!(ReportState::Open.encode().as_ref(), &[0]);
        assert_eq!(ReportState::Resolved.encode().as_ref(), &[1]);
        assert_eq!(ReportState::Dismissed.encode().as_ref(), &[2]);
    }

    #[test]
    fn test_decode_empty_data_returns_error() {
        let result = ReportState::decode(Cow::Borrowed(&[]));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_discriminant_returns_error() {
        for byte in [3u8, 4, 50, 255] {
            let result = ReportState::decode(Cow::Borrowed(&[byte]));
            assert!(result.is_err(), "expected error for discriminant {byte}");
        }
    }

    #[test]
    fn test_size_is_one() {
        let s = ReportState::default();
        assert_eq!(s.size(), 1);
    }

    #[test]
    fn test_alignment_is_one() {
        assert_eq!(ReportState::ALIGNMENT, 1);
    }

    #[test]
    fn test_size_is_fixed() {
        assert_eq!(ReportState::SIZE, DataSize::Fixed(1));
    }

    #[test]
    fn test_ord() {
        assert!(ReportState::Open < ReportState::Resolved);
        assert!(ReportState::Resolved < ReportState::Dismissed);
    }
}
