//! JSON validation functionality

use crate::traits::Validator;
use serde_json::Value;

/// JSON validator
pub struct JsonValidator;

impl Validator for JsonValidator {
    fn is_valid(&self, content: &str) -> bool {
        serde_json::from_str::<Value>(content).is_ok()
    }
    
    fn validate(&self, content: &str) -> Vec<String> {
        match serde_json::from_str::<Value>(content) {
            Ok(_) => vec![],
            Err(e) => vec![e.to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let validator = JsonValidator;
        assert!(validator.is_valid(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_invalid_json() {
        let validator = JsonValidator;
        assert!(!validator.is_valid(r#"{"key": "value",}"#));
    }

    #[test]
    fn test_validate_errors() {
        let validator = JsonValidator;
        let errors = validator.validate(r#"{"key": "value",}"#);
        assert!(!errors.is_empty());
    }
}
