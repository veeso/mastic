use wasm_dbms_api::prelude::*;

pub struct StatusContentValidator;

impl StatusContentValidator {
    pub fn check_content(content: &str) -> DbmsResult<()> {
        if content.is_empty() {
            return Err(DbmsError::Validation(
                "Status content cannot be empty".to_string(),
            ));
        }

        if content.chars().count() > crate::domain::status::MAX_STATUS_LENGTH {
            return Err(DbmsError::Validation(
                "Status content cannot exceed 500 characters".to_string(),
            ));
        }

        Ok(())
    }
}

impl Validate for StatusContentValidator {
    fn validate(&self, value: &Value) -> DbmsResult<()> {
        let Value::Text(text) = value else {
            return Err(DbmsError::Validation(
                "Status content must be a string".to_string(),
            ));
        };

        Self::check_content(text.as_str())
    }
}

pub struct StatusContentSanitizer;

impl StatusContentSanitizer {
    pub fn sanitize_content(content: &str) -> String {
        // For simplicity, we just trim the content. More complex sanitization can be added here.
        content.trim().to_string()
    }
}

impl Sanitize for StatusContentSanitizer {
    fn sanitize(&self, value: Value) -> DbmsResult<Value> {
        let Value::Text(text) = value else {
            return Err(DbmsError::Sanitize(
                "Status content must be a string".to_string(),
            ));
        };

        Ok(Self::sanitize_content(text.as_str()).into())
    }
}
