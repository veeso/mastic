//! URL validator for `Nullable<Text>` columns.

use wasm_dbms_api::prelude::{DbmsError, DbmsResult, UrlValidator, Validate, Value};

/// URL validator that accepts `Value::Null` in addition to valid URL
/// `Value::Text` inputs. Delegates to [`UrlValidator`] for non-null
/// values.
pub struct NullableUrlValidator;

impl Validate for NullableUrlValidator {
    fn validate(&self, value: &Value) -> DbmsResult<()> {
        match value {
            Value::Null => Ok(()),
            Value::Text(_) => UrlValidator.validate(value),
            _ => Err(DbmsError::Validation(
                "value must be a `Text` or `Null`".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_accepts_null() {
        assert!(NullableUrlValidator.validate(&Value::Null).is_ok());
    }

    #[test]
    fn test_accepts_valid_url() {
        let value: Value = "https://example.com/x".to_string().into();
        assert!(NullableUrlValidator.validate(&value).is_ok());
    }

    #[test]
    fn test_rejects_invalid_url() {
        let value: Value = "not-a-url".to_string().into();
        assert!(NullableUrlValidator.validate(&value).is_err());
    }

    #[test]
    fn test_rejects_non_text() {
        assert!(
            NullableUrlValidator
                .validate(&Value::Int32(1.into()))
                .is_err()
        );
    }
}
