//! Database custom type wrapping [`did::common::Visibility`].

use std::fmt;

use did::common::Visibility as DidVisibility;
use serde::{Deserialize, Serialize};
use wasm_dbms_api::memory::Encode;
use wasm_dbms_api::prelude::*;

/// Database type for [`DidVisibility`]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, CustomDataType,
)]
#[type_tag = "visibility"]
pub struct Visibility(DidVisibility);

impl Default for Visibility {
    fn default() -> Self {
        Self(DidVisibility::Public)
    }
}

impl From<DidVisibility> for Visibility {
    fn from(visibility: DidVisibility) -> Self {
        Self(visibility)
    }
}

impl From<Visibility> for DidVisibility {
    fn from(visibility: Visibility) -> Self {
        visibility.0
    }
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let visibility_str = match self.0 {
            DidVisibility::Public => "public",
            DidVisibility::Unlisted => "unlisted",
            DidVisibility::FollowersOnly => "followers_only",
            DidVisibility::Direct => "direct",
        };
        write!(f, "{}", visibility_str)
    }
}

impl Encode for Visibility {
    const ALIGNMENT: PageOffset = 1;

    const SIZE: DataSize = DataSize::Fixed(1);

    fn encode(&'_ self) -> std::borrow::Cow<'_, [u8]> {
        std::borrow::Cow::Owned(vec![match self.0 {
            DidVisibility::Public => 0,
            DidVisibility::Unlisted => 1,
            DidVisibility::FollowersOnly => 2,
            DidVisibility::Direct => 3,
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
        let visibility: DidVisibility = match byte {
            0 => DidVisibility::Public,
            1 => DidVisibility::Unlisted,
            2 => DidVisibility::FollowersOnly,
            3 => DidVisibility::Direct,
            _ => {
                return Err(MemoryError::DecodeError(DecodeError::InvalidDiscriminant(
                    byte,
                )));
            }
        };

        Ok(Self(visibility))
    }

    fn size(&self) -> MSize {
        Self::SIZE.get_fixed_size().unwrap() as MSize
    }
}

impl DataType for Visibility {}

#[cfg(test)]
mod tests {

    use std::borrow::Cow;

    use super::*;

    #[test]
    fn test_default_is_public() {
        let v = Visibility::default();
        assert_eq!(DidVisibility::from(v), DidVisibility::Public);
    }

    #[test]
    fn test_from_did_visibility() {
        let cases = [
            DidVisibility::Public,
            DidVisibility::Unlisted,
            DidVisibility::FollowersOnly,
            DidVisibility::Direct,
        ];

        for did_vis in cases {
            let v = Visibility::from(did_vis);
            assert_eq!(DidVisibility::from(v), did_vis);
        }
    }

    #[test]
    fn test_into_did_visibility() {
        assert_eq!(
            DidVisibility::from(Visibility::from(DidVisibility::Public)),
            DidVisibility::Public
        );
        assert_eq!(
            DidVisibility::from(Visibility::from(DidVisibility::Unlisted)),
            DidVisibility::Unlisted
        );
        assert_eq!(
            DidVisibility::from(Visibility::from(DidVisibility::FollowersOnly)),
            DidVisibility::FollowersOnly
        );
        assert_eq!(
            DidVisibility::from(Visibility::from(DidVisibility::Direct)),
            DidVisibility::Direct
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(
            Visibility::from(DidVisibility::Public).to_string(),
            "public"
        );
        assert_eq!(
            Visibility::from(DidVisibility::Unlisted).to_string(),
            "unlisted"
        );
        assert_eq!(
            Visibility::from(DidVisibility::FollowersOnly).to_string(),
            "followers_only"
        );
        assert_eq!(
            Visibility::from(DidVisibility::Direct).to_string(),
            "direct"
        );
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let cases = [
            DidVisibility::Public,
            DidVisibility::Unlisted,
            DidVisibility::FollowersOnly,
            DidVisibility::Direct,
        ];

        for did_vis in cases {
            let original = Visibility::from(did_vis);
            let encoded = original.encode();
            let decoded = Visibility::decode(encoded).unwrap();
            assert_eq!(original, decoded);
        }
    }

    #[test]
    fn test_encode_values() {
        assert_eq!(
            Visibility::from(DidVisibility::Public).encode().as_ref(),
            &[0]
        );
        assert_eq!(
            Visibility::from(DidVisibility::Unlisted).encode().as_ref(),
            &[1]
        );
        assert_eq!(
            Visibility::from(DidVisibility::FollowersOnly)
                .encode()
                .as_ref(),
            &[2]
        );
        assert_eq!(
            Visibility::from(DidVisibility::Direct).encode().as_ref(),
            &[3]
        );
    }

    #[test]
    fn test_decode_empty_data_returns_error() {
        let result = Visibility::decode(Cow::Borrowed(&[]));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_discriminant_returns_error() {
        for byte in [4u8, 5, 10, 255] {
            let result = Visibility::decode(Cow::Borrowed(&[byte]));
            assert!(result.is_err(), "expected error for discriminant {byte}");
        }
    }

    #[test]
    fn test_size_is_one() {
        let v = Visibility::default();
        assert_eq!(v.size(), 1);
    }

    #[test]
    fn test_alignment_is_one() {
        assert_eq!(Visibility::ALIGNMENT, 1);
    }

    #[test]
    fn test_size_is_fixed() {
        assert_eq!(Visibility::SIZE, DataSize::Fixed(1));
    }

    #[test]
    fn test_ord() {
        let public = Visibility::from(DidVisibility::Public);
        let unlisted = Visibility::from(DidVisibility::Unlisted);
        let followers = Visibility::from(DidVisibility::FollowersOnly);
        let direct = Visibility::from(DidVisibility::Direct);

        assert!(public < unlisted);
        assert!(unlisted < followers);
        assert!(followers < direct);
    }

    #[test]
    fn test_clone_and_copy() {
        let v = Visibility::from(DidVisibility::Unlisted);
        let copied = v;
        assert_eq!(v, copied);
    }

    #[test]
    fn test_debug() {
        let v = Visibility::from(DidVisibility::Public);
        let debug_str = format!("{v:?}");
        assert!(debug_str.contains("Visibility"));
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Visibility::from(DidVisibility::Public));
        set.insert(Visibility::from(DidVisibility::Public));
        assert_eq!(set.len(), 1);

        set.insert(Visibility::from(DidVisibility::Direct));
        assert_eq!(set.len(), 2);
    }
}
