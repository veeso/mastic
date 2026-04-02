//! Sanitizer and validator for the Handle type used in the database.

use wasm_dbms_api::prelude::{DbmsError, DbmsResult, Sanitize, Validate, Value};

/// Reserved handles that cannot be used by users in the Directory canister.
const RESERVED_HANDLES: &[&str] = &[
    "admin",
    "administrator",
    "autoconfig",
    "autodiscover",
    "help",
    "hostmaster",
    "info",
    "mailer-daemon",
    "postmaster",
    "root",
    "ssladmin",
    "support",
    "webmaster",
];

/// A sanitizer for user handles in the Directory canister.
///
/// It trims whitespace, converts to lowercase, and removes leading `@` if present.
pub struct HandleSanitizer;

impl Sanitize for HandleSanitizer {
    fn sanitize(&self, value: Value) -> DbmsResult<Value> {
        let Value::Text(handle) = value else {
            return Err(DbmsError::Sanitize("handle must be a `Text`".to_string()));
        };

        // Trim whitespace and convert to lowercase
        let sanitized_handle = handle.0.trim().to_lowercase();
        // remove leading `@`
        let sanitized_handle = sanitized_handle
            .strip_prefix('@')
            .unwrap_or(&sanitized_handle)
            .to_string();

        Ok(sanitized_handle.into())
    }
}

/// A validator for user handles in the Directory canister.
pub struct HandleValidator;

impl Validate for HandleValidator {
    fn validate(&self, value: &Value) -> DbmsResult<()> {
        let Value::Text(handle) = value else {
            return Err(DbmsError::Validation("handle must be a `Text`".to_string()));
        };

        // | Rule               | Value                  |
        // | :----------------- | :--------------------- |
        // | Allowed characters | `a-z`, `0-9`, `_`      |
        // | Minimum length     | 1                      |
        // | Maximum length     | 30                     |
        // | Case sensitivity   | Case-insensitive       |
        // | Storage            | Stored as lowercase    |

        // verify length is between 1 and 30 characters
        if handle.0.is_empty() || handle.0.len() > 30 {
            return Err(DbmsError::Validation(
                "handle must be between 1 and 30 characters long".to_string(),
            ));
        }

        // verify handle only contains allowed characters
        if !handle
            .0
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(DbmsError::Validation(
                "handle can only contain lowercase letters, digits, and underscores".to_string(),
            ));
        }

        if RESERVED_HANDLES.contains(&handle.0.as_str()) {
            return Err(DbmsError::Validation(
                "handle is reserved and cannot be used".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_should_sanitize_handle_trimming_whitespace() {
        let sanitizer = HandleSanitizer;
        let value: Value = "  alice  ".to_string().into();

        let result = sanitizer.sanitize(value).unwrap();

        assert_eq!(result, Value::Text("alice".into()));
    }

    #[test]
    fn test_should_sanitize_handle_converting_to_lowercase() {
        let sanitizer = HandleSanitizer;
        let value: Value = "Alice".to_string().into();

        let result = sanitizer.sanitize(value).unwrap();

        assert_eq!(result, Value::Text("alice".into()));
    }

    #[test]
    fn test_should_sanitize_handle_stripping_leading_at() {
        let sanitizer = HandleSanitizer;
        let value: Value = "@alice".to_string().into();

        let result = sanitizer.sanitize(value).unwrap();

        assert_eq!(result, Value::Text("alice".into()));
    }

    #[test]
    fn test_should_sanitize_handle_applying_all_transformations() {
        let sanitizer = HandleSanitizer;
        let value: Value = "  @Alice  ".to_string().into();

        let result = sanitizer.sanitize(value).unwrap();

        assert_eq!(result, Value::Text("alice".into()));
    }

    #[test]
    fn test_should_not_strip_at_sign_in_the_middle() {
        let sanitizer = HandleSanitizer;
        let value: Value = "al@ice".to_string().into();

        let result = sanitizer.sanitize(value).unwrap();

        assert_eq!(result, Value::Text("al@ice".into()));
    }

    #[test]
    fn test_should_fail_to_sanitize_non_text_value() {
        let sanitizer = HandleSanitizer;
        let value = Value::Int32(42.into());

        let result = sanitizer.sanitize(value);

        assert!(result.is_err());
    }

    #[test]
    fn test_should_validate_handle_with_lowercase_letters() {
        let validator = HandleValidator;
        let value: Value = "alice".to_string().into();

        assert!(validator.validate(&value).is_ok());
    }

    #[test]
    fn test_should_validate_handle_with_digits() {
        let validator = HandleValidator;
        let value: Value = "alice42".to_string().into();

        assert!(validator.validate(&value).is_ok());
    }

    #[test]
    fn test_should_validate_handle_with_underscores() {
        let validator = HandleValidator;
        let value: Value = "alice_bob".to_string().into();

        assert!(validator.validate(&value).is_ok());
    }

    #[test]
    fn test_should_validate_handle_with_minimum_length() {
        let validator = HandleValidator;
        let value: Value = "a".to_string().into();

        assert!(validator.validate(&value).is_ok());
    }

    #[test]
    fn test_should_validate_handle_with_maximum_length() {
        let validator = HandleValidator;
        let value: Value = "a".repeat(30).into();

        assert!(validator.validate(&value).is_ok());
    }

    #[test]
    fn test_should_reject_empty_handle() {
        let validator = HandleValidator;
        let value: Value = String::new().into();

        assert!(validator.validate(&value).is_err());
    }

    #[test]
    fn test_should_reject_handle_exceeding_max_length() {
        let validator = HandleValidator;
        let value: Value = "a".repeat(31).into();

        assert!(validator.validate(&value).is_err());
    }

    #[test]
    fn test_should_reject_handle_with_uppercase_letters() {
        let validator = HandleValidator;
        let value: Value = "Alice".to_string().into();

        assert!(validator.validate(&value).is_err());
    }

    #[test]
    fn test_should_reject_handle_with_special_characters() {
        let validator = HandleValidator;
        let value: Value = "alice!".to_string().into();

        assert!(validator.validate(&value).is_err());
    }

    #[test]
    fn test_should_reject_handle_with_spaces() {
        let validator = HandleValidator;
        let value: Value = "alice bob".to_string().into();

        assert!(validator.validate(&value).is_err());
    }

    #[test]
    fn test_should_reject_handle_with_at_sign() {
        let validator = HandleValidator;
        let value: Value = "@alice".to_string().into();

        assert!(validator.validate(&value).is_err());
    }

    #[test]
    fn test_should_fail_to_validate_non_text_value() {
        let validator = HandleValidator;
        let value = Value::Int32(42.into());

        assert!(validator.validate(&value).is_err());
    }

    #[test]
    fn test_should_reject_reserved_handle() {
        let validator = HandleValidator;
        let value: Value = "admin".to_string().into();

        assert!(validator.validate(&value).is_err());
    }
}
