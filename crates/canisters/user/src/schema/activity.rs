//! Database custom type wrapping [`activitypub::ActivityType`].

use std::fmt;

use activitypub::ActivityType as ApActivityType;
use serde::{Deserialize, Serialize};
use wasm_dbms_api::memory::Encode;
use wasm_dbms_api::prelude::*;

/// Database type for [`ApActivityType`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, CustomDataType,
)]
#[type_tag = "activity_type"]
pub struct ActivityType(ApActivityType);

impl Default for ActivityType {
    fn default() -> Self {
        Self(ApActivityType::Create)
    }
}

impl From<ApActivityType> for ActivityType {
    fn from(activity_type: ApActivityType) -> Self {
        Self(activity_type)
    }
}

impl From<ActivityType> for ApActivityType {
    fn from(activity_type: ActivityType) -> Self {
        activity_type.0
    }
}

impl fmt::Display for ActivityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let activity_str = match self.0 {
            ApActivityType::Create => "create",
            ApActivityType::Update => "update",
            ApActivityType::Delete => "delete",
            ApActivityType::Follow => "follow",
            ApActivityType::Accept => "accept",
            ApActivityType::Reject => "reject",
            ApActivityType::Like => "like",
            ApActivityType::Announce => "announce",
            ApActivityType::Undo => "undo",
            ApActivityType::Block => "block",
            ApActivityType::Add => "add",
            ApActivityType::Remove => "remove",
            ApActivityType::Flag => "flag",
            ApActivityType::Move => "move",
        };
        write!(f, "{}", activity_str)
    }
}

impl Encode for ActivityType {
    const ALIGNMENT: PageOffset = 1;

    const SIZE: DataSize = DataSize::Fixed(1);

    fn encode(&'_ self) -> std::borrow::Cow<'_, [u8]> {
        std::borrow::Cow::Owned(vec![match self.0 {
            ApActivityType::Create => 0,
            ApActivityType::Update => 1,
            ApActivityType::Delete => 2,
            ApActivityType::Follow => 3,
            ApActivityType::Accept => 4,
            ApActivityType::Reject => 5,
            ApActivityType::Like => 6,
            ApActivityType::Announce => 7,
            ApActivityType::Undo => 8,
            ApActivityType::Block => 9,
            ApActivityType::Add => 10,
            ApActivityType::Remove => 11,
            ApActivityType::Flag => 12,
            ApActivityType::Move => 13,
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
        let activity_type: ApActivityType = match byte {
            0 => ApActivityType::Create,
            1 => ApActivityType::Update,
            2 => ApActivityType::Delete,
            3 => ApActivityType::Follow,
            4 => ApActivityType::Accept,
            5 => ApActivityType::Reject,
            6 => ApActivityType::Like,
            7 => ApActivityType::Announce,
            8 => ApActivityType::Undo,
            9 => ApActivityType::Block,
            10 => ApActivityType::Add,
            11 => ApActivityType::Remove,
            12 => ApActivityType::Flag,
            13 => ApActivityType::Move,
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

impl DataType for ActivityType {}

#[cfg(test)]
mod tests {

    use std::borrow::Cow;

    use super::*;

    #[test]
    fn test_default_is_create() {
        let a = ActivityType::default();
        assert_eq!(ApActivityType::from(a), ApActivityType::Create);
    }

    #[test]
    fn test_from_ap_activity_type() {
        let cases = [
            ApActivityType::Create,
            ApActivityType::Update,
            ApActivityType::Delete,
            ApActivityType::Follow,
            ApActivityType::Accept,
            ApActivityType::Reject,
            ApActivityType::Like,
            ApActivityType::Announce,
            ApActivityType::Undo,
            ApActivityType::Block,
            ApActivityType::Add,
            ApActivityType::Remove,
            ApActivityType::Flag,
            ApActivityType::Move,
        ];

        for ap_type in cases {
            let a = ActivityType::from(ap_type);
            assert_eq!(ApActivityType::from(a), ap_type);
        }
    }

    #[test]
    fn test_into_ap_activity_type() {
        assert_eq!(
            ApActivityType::from(ActivityType::from(ApActivityType::Create)),
            ApActivityType::Create
        );
        assert_eq!(
            ApActivityType::from(ActivityType::from(ApActivityType::Update)),
            ApActivityType::Update
        );
        assert_eq!(
            ApActivityType::from(ActivityType::from(ApActivityType::Delete)),
            ApActivityType::Delete
        );
        assert_eq!(
            ApActivityType::from(ActivityType::from(ApActivityType::Follow)),
            ApActivityType::Follow
        );
        assert_eq!(
            ApActivityType::from(ActivityType::from(ApActivityType::Accept)),
            ApActivityType::Accept
        );
        assert_eq!(
            ApActivityType::from(ActivityType::from(ApActivityType::Reject)),
            ApActivityType::Reject
        );
        assert_eq!(
            ApActivityType::from(ActivityType::from(ApActivityType::Like)),
            ApActivityType::Like
        );
        assert_eq!(
            ApActivityType::from(ActivityType::from(ApActivityType::Announce)),
            ApActivityType::Announce
        );
        assert_eq!(
            ApActivityType::from(ActivityType::from(ApActivityType::Undo)),
            ApActivityType::Undo
        );
        assert_eq!(
            ApActivityType::from(ActivityType::from(ApActivityType::Block)),
            ApActivityType::Block
        );
        assert_eq!(
            ApActivityType::from(ActivityType::from(ApActivityType::Add)),
            ApActivityType::Add
        );
        assert_eq!(
            ApActivityType::from(ActivityType::from(ApActivityType::Remove)),
            ApActivityType::Remove
        );
        assert_eq!(
            ApActivityType::from(ActivityType::from(ApActivityType::Flag)),
            ApActivityType::Flag
        );
        assert_eq!(
            ApActivityType::from(ActivityType::from(ApActivityType::Move)),
            ApActivityType::Move
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(
            ActivityType::from(ApActivityType::Create).to_string(),
            "create"
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Update).to_string(),
            "update"
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Delete).to_string(),
            "delete"
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Follow).to_string(),
            "follow"
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Accept).to_string(),
            "accept"
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Reject).to_string(),
            "reject"
        );
        assert_eq!(ActivityType::from(ApActivityType::Like).to_string(), "like");
        assert_eq!(
            ActivityType::from(ApActivityType::Announce).to_string(),
            "announce"
        );
        assert_eq!(ActivityType::from(ApActivityType::Undo).to_string(), "undo");
        assert_eq!(
            ActivityType::from(ApActivityType::Block).to_string(),
            "block"
        );
        assert_eq!(ActivityType::from(ApActivityType::Add).to_string(), "add");
        assert_eq!(
            ActivityType::from(ApActivityType::Remove).to_string(),
            "remove"
        );
        assert_eq!(ActivityType::from(ApActivityType::Flag).to_string(), "flag");
        assert_eq!(ActivityType::from(ApActivityType::Move).to_string(), "move");
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let cases = [
            ApActivityType::Create,
            ApActivityType::Update,
            ApActivityType::Delete,
            ApActivityType::Follow,
            ApActivityType::Accept,
            ApActivityType::Reject,
            ApActivityType::Like,
            ApActivityType::Announce,
            ApActivityType::Undo,
            ApActivityType::Block,
            ApActivityType::Add,
            ApActivityType::Remove,
            ApActivityType::Flag,
            ApActivityType::Move,
        ];

        for ap_type in cases {
            let original = ActivityType::from(ap_type);
            let encoded = original.encode();
            let decoded = ActivityType::decode(encoded).unwrap();
            assert_eq!(original, decoded);
        }
    }

    #[test]
    fn test_encode_values() {
        assert_eq!(
            ActivityType::from(ApActivityType::Create).encode().as_ref(),
            &[0]
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Update).encode().as_ref(),
            &[1]
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Delete).encode().as_ref(),
            &[2]
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Follow).encode().as_ref(),
            &[3]
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Accept).encode().as_ref(),
            &[4]
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Reject).encode().as_ref(),
            &[5]
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Like).encode().as_ref(),
            &[6]
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Announce)
                .encode()
                .as_ref(),
            &[7]
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Undo).encode().as_ref(),
            &[8]
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Block).encode().as_ref(),
            &[9]
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Add).encode().as_ref(),
            &[10]
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Remove).encode().as_ref(),
            &[11]
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Flag).encode().as_ref(),
            &[12]
        );
        assert_eq!(
            ActivityType::from(ApActivityType::Move).encode().as_ref(),
            &[13]
        );
    }

    #[test]
    fn test_decode_empty_data_returns_error() {
        let result = ActivityType::decode(Cow::Borrowed(&[]));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_discriminant_returns_error() {
        for byte in [14u8, 15, 50, 255] {
            let result = ActivityType::decode(Cow::Borrowed(&[byte]));
            assert!(result.is_err(), "expected error for discriminant {byte}");
        }
    }

    #[test]
    fn test_size_is_one() {
        let a = ActivityType::default();
        assert_eq!(a.size(), 1);
    }

    #[test]
    fn test_alignment_is_one() {
        assert_eq!(ActivityType::ALIGNMENT, 1);
    }

    #[test]
    fn test_size_is_fixed() {
        assert_eq!(ActivityType::SIZE, DataSize::Fixed(1));
    }

    #[test]
    fn test_ord() {
        let create = ActivityType::from(ApActivityType::Create);
        let update = ActivityType::from(ApActivityType::Update);
        let delete = ActivityType::from(ApActivityType::Delete);
        let follow = ActivityType::from(ApActivityType::Follow);

        assert!(create < update);
        assert!(update < delete);
        assert!(delete < follow);
    }

    #[test]
    fn test_clone_and_copy() {
        let a = ActivityType::from(ApActivityType::Like);
        let copied = a;
        assert_eq!(a, copied);
    }

    #[test]
    fn test_debug() {
        let a = ActivityType::from(ApActivityType::Create);
        let debug_str = format!("{a:?}");
        assert!(debug_str.contains("ActivityType"));
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(ActivityType::from(ApActivityType::Create));
        set.insert(ActivityType::from(ApActivityType::Create));
        assert_eq!(set.len(), 1);

        set.insert(ActivityType::from(ApActivityType::Like));
        assert_eq!(set.len(), 2);
    }
}
