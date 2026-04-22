//! Database custom type wrapping [`did::directory::UserCanisterStatus`].

use std::fmt;

use did::directory::UserCanisterStatus as DidUserCanisterStatus;
use serde::{Deserialize, Serialize};
use wasm_dbms_api::memory::Encode;
use wasm_dbms_api::prelude::*;

/// Database type for [`DidUserCanisterStatus`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, CustomDataType,
)]
#[type_tag = "user_canister_status"]
pub struct UserCanisterStatus(pub DidUserCanisterStatus);

impl Default for UserCanisterStatus {
    fn default() -> Self {
        Self(DidUserCanisterStatus::CreationPending)
    }
}

impl From<DidUserCanisterStatus> for UserCanisterStatus {
    fn from(activity_type: DidUserCanisterStatus) -> Self {
        Self(activity_type)
    }
}

impl From<UserCanisterStatus> for DidUserCanisterStatus {
    fn from(activity_type: UserCanisterStatus) -> Self {
        activity_type.0
    }
}

impl fmt::Display for UserCanisterStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let activity_str = match self.0 {
            DidUserCanisterStatus::Active => "active",
            DidUserCanisterStatus::CreationPending => "creation_pending",
            DidUserCanisterStatus::CreationFailed => "creation_failed",
            DidUserCanisterStatus::DeletionPending => "deletion_pending",
        };
        write!(f, "{}", activity_str)
    }
}

impl Encode for UserCanisterStatus {
    const ALIGNMENT: PageOffset = 1;

    const SIZE: DataSize = DataSize::Fixed(1);

    fn encode(&'_ self) -> std::borrow::Cow<'_, [u8]> {
        std::borrow::Cow::Owned(vec![match self.0 {
            DidUserCanisterStatus::Active => 0,
            DidUserCanisterStatus::CreationPending => 1,
            DidUserCanisterStatus::CreationFailed => 2,
            DidUserCanisterStatus::DeletionPending => 3,
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
        let activity_type: DidUserCanisterStatus = match byte {
            0 => DidUserCanisterStatus::Active,
            1 => DidUserCanisterStatus::CreationPending,
            2 => DidUserCanisterStatus::CreationFailed,
            3 => DidUserCanisterStatus::DeletionPending,
            _ => {
                return Err(MemoryError::DecodeError(DecodeError::InvalidDiscriminant(
                    byte,
                )));
            }
        };

        Ok(Self(activity_type))
    }

    fn size(&self) -> MSize {
        Self::SIZE.get_fixed_size().unwrap() as MSize
    }
}

impl DataType for UserCanisterStatus {}

#[cfg(test)]
mod tests {

    use std::borrow::Cow;

    use super::*;

    #[test]
    fn test_default_is_create() {
        let a = UserCanisterStatus::default();
        assert_eq!(
            DidUserCanisterStatus::from(a),
            DidUserCanisterStatus::CreationPending
        );
    }

    #[test]
    fn test_from_ap_activity_type() {
        let cases = [
            DidUserCanisterStatus::Active,
            DidUserCanisterStatus::CreationPending,
            DidUserCanisterStatus::CreationFailed,
        ];

        for ap_type in cases {
            let a = UserCanisterStatus::from(ap_type);
            assert_eq!(DidUserCanisterStatus::from(a), ap_type);
        }
    }

    #[test]
    fn test_into_ap_activity_type() {
        assert_eq!(
            DidUserCanisterStatus::from(UserCanisterStatus::from(DidUserCanisterStatus::Active)),
            DidUserCanisterStatus::Active
        );
        assert_eq!(
            DidUserCanisterStatus::from(UserCanisterStatus::from(
                DidUserCanisterStatus::CreationPending
            )),
            DidUserCanisterStatus::CreationPending
        );
        assert_eq!(
            DidUserCanisterStatus::from(UserCanisterStatus::from(
                DidUserCanisterStatus::CreationFailed
            )),
            DidUserCanisterStatus::CreationFailed
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(
            UserCanisterStatus::from(DidUserCanisterStatus::Active).to_string(),
            "active"
        );
        assert_eq!(
            UserCanisterStatus::from(DidUserCanisterStatus::CreationPending).to_string(),
            "creation_pending"
        );
        assert_eq!(
            UserCanisterStatus::from(DidUserCanisterStatus::CreationFailed).to_string(),
            "creation_failed"
        );
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let cases = [
            DidUserCanisterStatus::Active,
            DidUserCanisterStatus::CreationPending,
            DidUserCanisterStatus::CreationFailed,
        ];

        for ap_type in cases {
            let original = UserCanisterStatus::from(ap_type);
            let encoded = original.encode();
            let decoded = UserCanisterStatus::decode(encoded).unwrap();
            assert_eq!(original, decoded);
        }
    }

    #[test]
    fn test_encode_values() {
        assert_eq!(
            UserCanisterStatus::from(DidUserCanisterStatus::Active)
                .encode()
                .as_ref(),
            &[0]
        );
        assert_eq!(
            UserCanisterStatus::from(DidUserCanisterStatus::CreationPending)
                .encode()
                .as_ref(),
            &[1]
        );
        assert_eq!(
            UserCanisterStatus::from(DidUserCanisterStatus::CreationFailed)
                .encode()
                .as_ref(),
            &[2]
        );
    }

    #[test]
    fn test_decode_empty_data_returns_error() {
        let result = UserCanisterStatus::decode(Cow::Borrowed(&[]));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_discriminant_returns_error() {
        for byte in [14u8, 15, 50, 255] {
            let result = UserCanisterStatus::decode(Cow::Borrowed(&[byte]));
            assert!(result.is_err(), "expected error for discriminant {byte}");
        }
    }

    #[test]
    fn test_size_is_one() {
        let a = UserCanisterStatus::default();
        assert_eq!(a.size(), 1);
    }

    #[test]
    fn test_alignment_is_one() {
        assert_eq!(UserCanisterStatus::ALIGNMENT, 1);
    }

    #[test]
    fn test_size_is_fixed() {
        assert_eq!(UserCanisterStatus::SIZE, DataSize::Fixed(1));
    }

    #[test]
    fn test_ord() {
        let active = UserCanisterStatus::from(DidUserCanisterStatus::Active);
        let pending = UserCanisterStatus::from(DidUserCanisterStatus::CreationPending);
        let failed = UserCanisterStatus::from(DidUserCanisterStatus::CreationFailed);

        assert!(active < pending);
        assert!(pending < failed);
    }

    #[test]
    fn test_clone_and_copy() {
        let a = UserCanisterStatus::from(DidUserCanisterStatus::Active);
        let copied = a;
        assert_eq!(a, copied);
    }

    #[test]
    fn test_debug() {
        let a = UserCanisterStatus::from(DidUserCanisterStatus::Active);
        let debug_str = format!("{a:?}");
        assert!(debug_str.contains("UserCanisterStatus"));
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(UserCanisterStatus::from(DidUserCanisterStatus::Active));
        set.insert(UserCanisterStatus::from(DidUserCanisterStatus::Active));
        assert_eq!(set.len(), 1);

        set.insert(UserCanisterStatus::from(
            DidUserCanisterStatus::CreationFailed,
        ));
        assert_eq!(set.len(), 2);
    }
}
