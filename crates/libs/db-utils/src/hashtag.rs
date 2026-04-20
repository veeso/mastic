//! Sanitizer and validator for the Hashtag `tag` column.
//!
//! See [`docs/src/specs/hashtags.md`](../../../docs/src/specs/hashtags.md)
//! for the full specification.

use wasm_dbms_api::prelude::{DbmsError, DbmsResult, Sanitize, Validate, Value};

/// Maximum length, in Unicode scalar values, of a hashtag tag.
pub const MAX_HASHTAG_LENGTH: usize = 30;

/// Sanitizer for the Hashtag `tag` column.
///
/// Trims whitespace, lowercases, and strips a leading `#` if present.
pub struct HashtagSanitizer;

impl HashtagSanitizer {
    /// Sanitize a tag by trimming whitespace, lowercasing, and
    /// stripping a single leading `#` if present.
    pub fn sanitize_tag(tag: &str) -> String {
        let trimmed = tag.trim().to_lowercase();
        trimmed.strip_prefix('#').unwrap_or(&trimmed).to_string()
    }
}

impl Sanitize for HashtagSanitizer {
    fn sanitize(&self, value: Value) -> DbmsResult<Value> {
        let Value::Text(tag) = value else {
            return Err(DbmsError::Sanitize("tag must be a `Text`".to_string()));
        };

        Ok(Self::sanitize_tag(tag.as_str()).into())
    }
}

/// Validator for the Hashtag `tag` column.
pub struct HashtagValidator;

impl HashtagValidator {
    /// Check that a tag is valid according to the following rules:
    ///
    /// | Rule               | Value                        |
    /// | :----------------- | :--------------------------- |
    /// | Allowed characters | `a-z`, `0-9`, `_`            |
    /// | Minimum length     | 1                            |
    /// | Maximum length     | 30 Unicode scalar values     |
    /// | Case sensitivity   | Case-insensitive             |
    /// | Storage            | Stored as lowercase, no `#`  |
    pub fn check_tag(tag: &str) -> Result<(), String> {
        let len = tag.chars().count();
        if len == 0 || len > MAX_HASHTAG_LENGTH {
            return Err(format!(
                "tag must be between 1 and {MAX_HASHTAG_LENGTH} characters long"
            ));
        }

        if !tag
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(
                "tag can only contain lowercase letters, digits, and underscores".to_string(),
            );
        }

        Ok(())
    }
}

impl Validate for HashtagValidator {
    fn validate(&self, value: &Value) -> DbmsResult<()> {
        let Value::Text(tag) = value else {
            return Err(DbmsError::Validation("tag must be a `Text`".to_string()));
        };

        Self::check_tag(tag.as_str()).map_err(DbmsError::Validation)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_should_sanitize_tag_trimming_whitespace() {
        let sanitizer = HashtagSanitizer;
        let value: Value = "  rust  ".to_string().into();
        let result = sanitizer.sanitize(value).unwrap();
        assert_eq!(result, Value::Text("rust".into()));
    }

    #[test]
    fn test_should_sanitize_tag_lowercasing() {
        let sanitizer = HashtagSanitizer;
        let value: Value = "Rust".to_string().into();
        let result = sanitizer.sanitize(value).unwrap();
        assert_eq!(result, Value::Text("rust".into()));
    }

    #[test]
    fn test_should_sanitize_tag_stripping_leading_hash() {
        let sanitizer = HashtagSanitizer;
        let value: Value = "#rust".to_string().into();
        let result = sanitizer.sanitize(value).unwrap();
        assert_eq!(result, Value::Text("rust".into()));
    }

    #[test]
    fn test_should_sanitize_tag_applying_all_transformations() {
        let sanitizer = HashtagSanitizer;
        let value: Value = "  #Rust  ".to_string().into();
        let result = sanitizer.sanitize(value).unwrap();
        assert_eq!(result, Value::Text("rust".into()));
    }

    #[test]
    fn test_should_not_strip_hash_in_the_middle() {
        let sanitizer = HashtagSanitizer;
        let value: Value = "ru#st".to_string().into();
        let result = sanitizer.sanitize(value).unwrap();
        assert_eq!(result, Value::Text("ru#st".into()));
    }

    #[test]
    fn test_should_fail_to_sanitize_non_text_value() {
        let sanitizer = HashtagSanitizer;
        let value = Value::Int32(42.into());
        assert!(sanitizer.sanitize(value).is_err());
    }

    #[test]
    fn test_should_validate_tag_with_lowercase_letters() {
        let validator = HashtagValidator;
        let value: Value = "rust".to_string().into();
        assert!(validator.validate(&value).is_ok());
    }

    #[test]
    fn test_should_validate_tag_with_digits() {
        let validator = HashtagValidator;
        let value: Value = "web3".to_string().into();
        assert!(validator.validate(&value).is_ok());
    }

    #[test]
    fn test_should_validate_tag_with_underscores() {
        let validator = HashtagValidator;
        let value: Value = "rust_lang".to_string().into();
        assert!(validator.validate(&value).is_ok());
    }

    #[test]
    fn test_should_validate_tag_with_minimum_length() {
        let validator = HashtagValidator;
        let value: Value = "a".to_string().into();
        assert!(validator.validate(&value).is_ok());
    }

    #[test]
    fn test_should_validate_tag_with_maximum_length() {
        let validator = HashtagValidator;
        let value: Value = "a".repeat(MAX_HASHTAG_LENGTH).into();
        assert!(validator.validate(&value).is_ok());
    }

    #[test]
    fn test_should_reject_empty_tag() {
        let validator = HashtagValidator;
        let value: Value = String::new().into();
        assert!(validator.validate(&value).is_err());
    }

    #[test]
    fn test_should_reject_tag_exceeding_max_length() {
        let validator = HashtagValidator;
        let value: Value = "a".repeat(MAX_HASHTAG_LENGTH + 1).into();
        assert!(validator.validate(&value).is_err());
    }

    #[test]
    fn test_should_reject_tag_with_uppercase_letters() {
        let validator = HashtagValidator;
        let value: Value = "Rust".to_string().into();
        assert!(validator.validate(&value).is_err());
    }

    #[test]
    fn test_should_reject_tag_with_special_characters() {
        let validator = HashtagValidator;
        let value: Value = "rust!".to_string().into();
        assert!(validator.validate(&value).is_err());
    }

    #[test]
    fn test_should_reject_tag_with_spaces() {
        let validator = HashtagValidator;
        let value: Value = "rust lang".to_string().into();
        assert!(validator.validate(&value).is_err());
    }

    #[test]
    fn test_should_reject_tag_with_leading_hash() {
        let validator = HashtagValidator;
        let value: Value = "#rust".to_string().into();
        assert!(validator.validate(&value).is_err());
    }

    #[test]
    fn test_should_reject_tag_with_hyphen() {
        let validator = HashtagValidator;
        let value: Value = "rust-lang".to_string().into();
        assert!(validator.validate(&value).is_err());
    }

    #[test]
    fn test_should_fail_to_validate_non_text_value() {
        let validator = HashtagValidator;
        let value = Value::Int32(42.into());
        assert!(validator.validate(&value).is_err());
    }
}
