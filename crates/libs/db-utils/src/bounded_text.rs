//! Generic text sanitizer and bounded-length validator.
//!
//! Used by any `Text` column that needs whitespace trimming and/or a
//! maximum length expressed in Unicode scalar values.

use wasm_dbms_api::prelude::{DbmsError, DbmsResult, Sanitize, Validate, Value};

/// Sanitizer that trims ASCII and Unicode whitespace from both ends of
/// a `Text` value.
pub struct TrimSanitizer;

impl Sanitize for TrimSanitizer {
    fn sanitize(&self, value: Value) -> DbmsResult<Value> {
        match value {
            Value::Text(text) => Ok(text.as_str().trim().to_string().into()),
            Value::Null => Ok(Value::Null),
            _ => Err(DbmsError::Sanitize("value must be a `Text`".to_string())),
        }
    }
}

/// Validator that enforces a maximum length, in Unicode scalar values,
/// on a `Text` column. The minimum length is always 1 (empty strings
/// are rejected).
pub struct BoundedTextValidator(pub usize);

impl BoundedTextValidator {
    /// Check that `text` is non-empty and no longer than `max_len`
    /// Unicode scalar values.
    pub fn check(text: &str, max_len: usize) -> Result<(), String> {
        let len = text.chars().count();
        if len == 0 {
            return Err("value cannot be empty".to_string());
        }
        if len > max_len {
            return Err(format!(
                "value cannot exceed {max_len} characters (got {len})"
            ));
        }
        Ok(())
    }
}

impl Validate for BoundedTextValidator {
    fn validate(&self, value: &Value) -> DbmsResult<()> {
        match value {
            Value::Text(text) => Self::check(text.as_str(), self.0).map_err(DbmsError::Validation),
            Value::Null => Ok(()),
            _ => Err(DbmsError::Validation("value must be a `Text`".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_should_trim_whitespace() {
        let sanitizer = TrimSanitizer;
        let value: Value = "  hello  ".to_string().into();
        assert_eq!(
            sanitizer.sanitize(value).unwrap(),
            Value::Text("hello".into())
        );
    }

    #[test]
    fn test_should_trim_unicode_whitespace() {
        let sanitizer = TrimSanitizer;
        let value: Value = "\u{3000}hello\u{3000}".to_string().into();
        assert_eq!(
            sanitizer.sanitize(value).unwrap(),
            Value::Text("hello".into())
        );
    }

    #[test]
    fn test_trim_should_fail_on_non_text() {
        let sanitizer = TrimSanitizer;
        let value = Value::Int32(42.into());
        assert!(sanitizer.sanitize(value).is_err());
    }

    #[test]
    fn test_should_accept_text_within_limit() {
        let validator = BoundedTextValidator(10);
        assert!(validator.validate(&"hello".to_string().into()).is_ok());
    }

    #[test]
    fn test_should_accept_text_at_exact_limit() {
        let validator = BoundedTextValidator(5);
        assert!(validator.validate(&"hello".to_string().into()).is_ok());
    }

    #[test]
    fn test_should_reject_text_over_limit() {
        let validator = BoundedTextValidator(3);
        assert!(validator.validate(&"hello".to_string().into()).is_err());
    }

    #[test]
    fn test_should_reject_empty_text() {
        let validator = BoundedTextValidator(10);
        assert!(validator.validate(&String::new().into()).is_err());
    }

    #[test]
    fn test_should_count_unicode_scalar_values() {
        let validator = BoundedTextValidator(3);
        // "😀" is one scalar value (but 4 bytes)
        assert!(validator.validate(&"😀😀😀".to_string().into()).is_ok());
        assert!(validator.validate(&"😀😀😀😀".to_string().into()).is_err());
    }

    #[test]
    fn test_bounded_should_fail_on_non_text() {
        let validator = BoundedTextValidator(10);
        let value = Value::Int32(42.into());
        assert!(validator.validate(&value).is_err());
    }
}
