use std::borrow::Cow;

use candid::Principal;
use ic_stable_structures::Storable;

use crate::memory::StorablePrincipal;

const MAX_HANDLE_LEN: usize = 30;

/// Per-user data cached in the federation canister for fast lookups.
///
/// Wire layout (fixed-size regions, length-prefixed principals):
///
/// ```text
/// [1 B len][29 B user_id padded][30 B handle padded][1 B len][29 B canister_id padded]
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserData {
    pub user_id: Principal,
    pub user_handle: String,
    pub user_canister_id: Principal,
}

impl UserData {
    /// Encode a principal as 1-byte length prefix + bytes + zero-padding to 29 bytes.
    fn encode_principal(buf: &mut Vec<u8>, principal: Principal) {
        let storable = StorablePrincipal::from(principal);
        let raw = storable.to_bytes();
        buf.push(raw.len() as u8);
        buf.extend_from_slice(&raw);
        // pad to fixed width
        buf.resize(
            buf.len() + StorablePrincipal::MAX_PRINCIPAL_LENGTH_IN_BYTES - raw.len(),
            0,
        );
    }

    /// Decode a principal from a 1 + 29 byte window.
    fn decode_principal(window: &[u8]) -> Principal {
        let len = window[0] as usize;
        *StorablePrincipal::from_bytes(Cow::Borrowed(&window[1..1 + len])).as_principal()
    }
}

/// Size of one length-prefixed principal field.
const PRINCIPAL_FIELD_SIZE: u32 = 1 + StorablePrincipal::MAX_PRINCIPAL_LENGTH_IN_BYTES as u32;

impl Storable for UserData {
    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: PRINCIPAL_FIELD_SIZE * 2 + MAX_HANDLE_LEN as u32,
            is_fixed_size: true,
        };

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf =
            Vec::with_capacity((PRINCIPAL_FIELD_SIZE * 2 + MAX_HANDLE_LEN as u32) as usize);

        Self::encode_principal(&mut buf, self.user_id);

        let mut handle_buf = [0u8; MAX_HANDLE_LEN];
        handle_buf[..self.user_handle.len()].copy_from_slice(self.user_handle.as_bytes());
        buf.extend_from_slice(&handle_buf);

        Self::encode_principal(&mut buf, self.user_canister_id);

        Cow::Owned(buf)
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        let bytes = bytes.as_ref();
        let pf = PRINCIPAL_FIELD_SIZE as usize;

        let user_id = Self::decode_principal(&bytes[..pf]);

        let handle_start = pf;
        let handle_end = handle_start + MAX_HANDLE_LEN;
        let user_handle = String::from_utf8_lossy(&bytes[handle_start..handle_end])
            .trim_end_matches('\0')
            .to_string();

        let user_canister_id = Self::decode_principal(&bytes[handle_end..handle_end + pf]);

        UserData {
            user_id,
            user_handle,
            user_canister_id,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_should_roundtrip_user_data() {
        let data = UserData {
            user_id: Principal::from_text("mfufu-x6j4c-gomzb-geilq").unwrap(),
            user_handle: "alice".to_string(),
            user_canister_id: Principal::from_text("bkyz2-fmaaa-aaaaa-qaaaq-cai").unwrap(),
        };

        let bytes = data.to_bytes();
        let decoded = UserData::from_bytes(bytes);

        assert_eq!(data, decoded);
    }

    #[test]
    fn test_should_roundtrip_user_data_with_max_handle() {
        let data = UserData {
            user_id: Principal::from_slice(&[7; 29]),
            user_handle: "a]".repeat(15),
            user_canister_id: Principal::from_slice(&[3; 10]),
        };

        let bytes = data.to_bytes();
        let decoded = UserData::from_bytes(bytes);

        assert_eq!(data, decoded);
    }

    #[test]
    fn test_should_produce_fixed_size_encoding() {
        let short = UserData {
            user_id: Principal::from_slice(&[1; 5]),
            user_handle: "a".to_string(),
            user_canister_id: Principal::from_slice(&[2; 5]),
        };
        let long = UserData {
            user_id: Principal::from_slice(&[1; 29]),
            user_handle: "abcdefghijklmnopqrstuvwxyz1234".to_string(),
            user_canister_id: Principal::from_slice(&[2; 29]),
        };

        assert_eq!(short.to_bytes().len(), long.to_bytes().len());
    }
}
