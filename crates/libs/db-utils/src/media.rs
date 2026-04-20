//! Validators for media attachment metadata columns.
//!
//! See [`docs/src/specs/media.md`](../../../docs/src/specs/media.md) for
//! the full specification.

use wasm_dbms_api::prelude::{DbmsError, DbmsResult, Validate, Value};

/// Maximum length of a `media_type` value, in bytes. Covers every
/// reasonable MIME type including long vendor / parameter suffixes.
pub const MAX_MIME_LENGTH: usize = 127;

/// Minimum length of a valid blurhash string.
pub const MIN_BLURHASH_LENGTH: usize = 6;

/// Maximum length of a blurhash string.
pub const MAX_BLURHASH_LENGTH: usize = 128;

/// Validator for the `media.media_type` column.
///
/// Enforces the `type/subtype` shape defined by
/// [RFC 6838](https://www.rfc-editor.org/rfc/rfc6838).
pub struct MimeValidator;

impl MimeValidator {
    /// Check that `mime` is a well-formed `type/subtype` value.
    ///
    /// - Length between 3 and [`MAX_MIME_LENGTH`] bytes.
    /// - Contains exactly one `/`.
    /// - `type` and `subtype` are non-empty.
    /// - All characters are ASCII graphic (no whitespace).
    /// - No uppercase letters (MIME types are stored lowercased).
    pub fn check(mime: &str) -> Result<(), String> {
        if mime.is_empty() || mime.len() > MAX_MIME_LENGTH {
            return Err(format!(
                "media_type length must be between 1 and {MAX_MIME_LENGTH} bytes"
            ));
        }

        let slash_count = mime.bytes().filter(|b| *b == b'/').count();
        if slash_count != 1 {
            return Err("media_type must contain exactly one `/`".to_string());
        }

        let Some((ty, subty)) = mime.split_once('/') else {
            return Err("media_type must be `type/subtype`".to_string());
        };

        if ty.is_empty() || subty.is_empty() {
            return Err("media_type `type` and `subtype` must be non-empty".to_string());
        }

        if !mime
            .chars()
            .all(|c| c.is_ascii_graphic() && !c.is_ascii_uppercase())
        {
            return Err(
                "media_type must contain only lowercase ASCII graphic characters".to_string(),
            );
        }

        Ok(())
    }
}

impl Validate for MimeValidator {
    fn validate(&self, value: &Value) -> DbmsResult<()> {
        match value {
            Value::Text(mime) => Self::check(mime.as_str()).map_err(DbmsError::Validation),
            Value::Null => Ok(()),
            _ => Err(DbmsError::Validation(
                "media_type must be a `Text`".to_string(),
            )),
        }
    }
}

const BLURHASH_ALPHABET: &[u8] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz#$%*+,-.:;=?@[]^_{|}~";

/// Validator for the `media.blurhash` column.
///
/// Enforces the base83 alphabet defined by the
/// [blurhash specification](https://blurha.sh/).
pub struct BlurhashValidator;

impl BlurhashValidator {
    /// Check that `blurhash` is composed of the base83 alphabet and
    /// falls within the accepted length range.
    pub fn check(blurhash: &str) -> Result<(), String> {
        let len = blurhash.len();
        if !(MIN_BLURHASH_LENGTH..=MAX_BLURHASH_LENGTH).contains(&len) {
            return Err(format!(
                "blurhash length must be between {MIN_BLURHASH_LENGTH} and {MAX_BLURHASH_LENGTH} characters"
            ));
        }

        if !blurhash.bytes().all(|b| BLURHASH_ALPHABET.contains(&b)) {
            return Err("blurhash contains characters outside the base83 alphabet".to_string());
        }

        Ok(())
    }
}

impl Validate for BlurhashValidator {
    fn validate(&self, value: &Value) -> DbmsResult<()> {
        match value {
            Value::Text(blurhash) => Self::check(blurhash.as_str()).map_err(DbmsError::Validation),
            Value::Null => Ok(()),
            _ => Err(DbmsError::Validation(
                "blurhash must be a `Text`".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_mime_accepts_common_types() {
        for mime in ["image/png", "image/jpeg", "video/mp4", "application/json"] {
            assert!(
                MimeValidator.validate(&mime.to_string().into()).is_ok(),
                "{mime} should be accepted"
            );
        }
    }

    #[test]
    fn test_mime_accepts_vendor_suffix() {
        assert!(
            MimeValidator
                .validate(&"application/vnd.mastic.v1+json".to_string().into())
                .is_ok()
        );
    }

    #[test]
    fn test_mime_rejects_empty() {
        assert!(MimeValidator.validate(&String::new().into()).is_err());
    }

    #[test]
    fn test_mime_rejects_missing_slash() {
        assert!(
            MimeValidator
                .validate(&"imagepng".to_string().into())
                .is_err()
        );
    }

    #[test]
    fn test_mime_rejects_multiple_slashes() {
        assert!(
            MimeValidator
                .validate(&"image/png/x".to_string().into())
                .is_err()
        );
    }

    #[test]
    fn test_mime_rejects_empty_subtype() {
        assert!(
            MimeValidator
                .validate(&"image/".to_string().into())
                .is_err()
        );
    }

    #[test]
    fn test_mime_rejects_whitespace() {
        assert!(
            MimeValidator
                .validate(&"image /png".to_string().into())
                .is_err()
        );
    }

    #[test]
    fn test_mime_rejects_uppercase() {
        assert!(
            MimeValidator
                .validate(&"Image/png".to_string().into())
                .is_err()
        );
    }

    #[test]
    fn test_mime_rejects_over_max_length() {
        let long = format!("image/{}", "x".repeat(MAX_MIME_LENGTH));
        assert!(MimeValidator.validate(&long.into()).is_err());
    }

    #[test]
    fn test_mime_rejects_non_text() {
        assert!(MimeValidator.validate(&Value::Int32(1.into())).is_err());
    }

    #[test]
    fn test_blurhash_accepts_valid_string() {
        let sample = "LEHV6nWB2yk8pyo0adR*.7kCMdnj";
        assert!(
            BlurhashValidator
                .validate(&sample.to_string().into())
                .is_ok()
        );
    }

    #[test]
    fn test_blurhash_rejects_too_short() {
        assert!(
            BlurhashValidator
                .validate(&"abc".to_string().into())
                .is_err()
        );
    }

    #[test]
    fn test_blurhash_rejects_too_long() {
        let long = "A".repeat(MAX_BLURHASH_LENGTH + 1);
        assert!(BlurhashValidator.validate(&long.into()).is_err());
    }

    #[test]
    fn test_blurhash_rejects_invalid_alphabet() {
        // `"` is not in the base83 alphabet.
        assert!(
            BlurhashValidator
                .validate(&"LEHV6\"WB2yk8".to_string().into())
                .is_err()
        );
    }

    #[test]
    fn test_blurhash_rejects_non_text() {
        assert!(BlurhashValidator.validate(&Value::Int32(1.into())).is_err());
    }
}
