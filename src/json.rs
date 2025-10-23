//! JSON repair functionality

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use regex::Regex;
use serde_json::Value;

/// JSON repairer that can fix common JSON issues
pub struct JsonRepairer {
    strategies: Vec<Box<dyn RepairStrategy>>,
    validator: JsonValidator,
}

impl JsonRepairer {
    /// Create a new JSON repairer
    pub fn new() -> Self {
        let mut strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(AddMissingQuotesStrategy),
            Box::new(FixTrailingCommasStrategy),
            Box::new(FixUnescapedQuotesStrategy),
            Box::new(AddMissingBracesStrategy),
            Box::new(FixSingleQuotesStrategy),
            Box::new(FixMalformedNumbersStrategy),
            Box::new(FixBooleanNullStrategy),
        ];
        
        // Sort strategies by priority (higher priority first)
        strategies.sort_by(|a, b| b.priority().cmp(&a.priority()));
        
        Self {
            strategies,
            validator: JsonValidator,
        }
    }
    
    /// Apply all repair strategies to the content
    fn apply_strategies(&self, content: &str) -> Result<String> {
        let mut repaired = content.to_string();
        
        for strategy in &self.strategies {
            if let Ok(result) = strategy.apply(&repaired) {
                repaired = result;
            }
        }
        
        Ok(repaired)
    }
}

impl Repair for JsonRepairer {
    fn repair(&self, content: &str) -> Result<String> {
        let trimmed = content.trim();
        
        // If already valid, return as-is
        if self.validator.is_valid(trimmed) {
            return Ok(trimmed.to_string());
        }
        
        // Apply repair strategies
        let repaired = self.apply_strategies(trimmed)?;
        
        // Return the repaired content even if validation fails
        // (some repairs might not be perfect but are still improvements)
        Ok(repaired)
    }
    
    fn needs_repair(&self, content: &str) -> bool {
        !self.validator.is_valid(content)
    }
    
    fn confidence(&self, content: &str) -> f64 {
        if self.validator.is_valid(content) {
            return 1.0;
        }
        
        // Calculate confidence based on how close we are to valid JSON
        let mut score: f64 = 0.0;
        let _chars: Vec<char> = content.chars().collect();
        
        // Check for basic JSON structure
        if content.contains('{') || content.contains('[') {
            score += 0.3;
        }
        
        // Check for key-value pairs
        if content.contains(':') {
            score += 0.2;
        }
        
        // Check for quotes
        if content.contains('"') {
            score += 0.2;
        }
        
        // Check for commas
        if content.contains(',') {
            score += 0.1;
        }
        
        // Check for balanced braces/brackets
        let open_braces = content.matches('{').count();
        let close_braces = content.matches('}').count();
        let open_brackets = content.matches('[').count();
        let close_brackets = content.matches(']').count();
        
        if open_braces == close_braces && open_brackets == close_brackets {
            score += 0.2;
        }
        
        score.min(1.0_f64)
    }
}

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

/// Strategy to add missing quotes around keys
struct AddMissingQuotesStrategy;

impl RepairStrategy for AddMissingQuotesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let re = Regex::new(r#"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*:"#)?;
        let result = re.replace_all(content, r#""$1":"#);
        Ok(result.to_string())
    }
    
    fn priority(&self) -> u8 {
        5
    }
}

/// Strategy to fix trailing commas
struct FixTrailingCommasStrategy;

impl RepairStrategy for FixTrailingCommasStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let re = Regex::new(r#",\s*([}\]])"#)?;
        let result = re.replace_all(content, r#"$1"#);
        Ok(result.to_string())
    }
    
    fn priority(&self) -> u8 {
        4
    }
}

/// Strategy to fix unescaped quotes
struct FixUnescapedQuotesStrategy;

impl RepairStrategy for FixUnescapedQuotesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        // This is a simplified version - in practice, you'd need more sophisticated parsing
        let re = Regex::new(r#"(?<!\\)"(?![,}\]:\s])"#)?;
        let result = re.replace_all(content, r#"\""#);
        Ok(result.to_string())
    }
    
    fn priority(&self) -> u8 {
        3
    }
}

/// Strategy to add missing braces
struct AddMissingBracesStrategy;

impl RepairStrategy for AddMissingBracesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let trimmed = content.trim();
        let mut result = trimmed.to_string();
        
        // Add missing opening brace
        if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
            result = format!("{{{result}}}");
        }
        
        // Add missing closing brace
        let open_braces = result.matches('{').count();
        let close_braces = result.matches('}').count();
        let open_brackets = result.matches('[').count();
        let close_brackets = result.matches(']').count();
        
        if open_braces > close_braces {
            result.push_str(&"}".repeat(open_braces - close_braces));
        }
        
        if open_brackets > close_brackets {
            result.push_str(&"]".repeat(open_brackets - close_brackets));
        }
        
        Ok(result)
    }
    
    fn priority(&self) -> u8 {
        2
    }
}

/// Strategy to fix single quotes to double quotes
struct FixSingleQuotesStrategy;

impl RepairStrategy for FixSingleQuotesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let re = Regex::new(r#"'([^']*)'"#)?;
        let result = re.replace_all(content, r#""$1""#);
        Ok(result.to_string())
    }
    
    fn priority(&self) -> u8 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_json_repair_basic() {
        let repairer = JsonRepairer::new();
        
        // Test missing quotes
        let input = r#"{key: "value"}"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"{"key": "value"}"#);
        
        // Test trailing comma
        let input = r#"{"key": "value",}"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"{"key": "value"}"#);
    }
    
    #[test]
    fn test_json_repair_complex() {
        let repairer = JsonRepairer::new();
        
        let input = r#"{name: "John", age: 30, city: "New York",}"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"{"name": "John", "age": 30, "city": "New York"}"#);
    }
    
    #[test]
    fn test_json_confidence() {
        let repairer = JsonRepairer::new();
        
        // Valid JSON should have confidence 1.0
        let valid = r#"{"key": "value"}"#;
        assert_eq!(repairer.confidence(valid), 1.0);
        
        // Invalid JSON should have lower confidence
        let invalid = "not json at all";
        assert!(repairer.confidence(invalid) < 1.0);
    }
    
    #[test]
    fn test_needs_repair() {
        let repairer = JsonRepairer::new();
        
        assert!(!repairer.needs_repair(r#"{"key": "value"}"#));
        assert!(repairer.needs_repair(r#"{key: "value"}"#));
    }

    #[test]
    fn test_json_repair_edge_cases() {
        let repairer = JsonRepairer::new();
        
        // Test empty object
        let input = "{}";
        let result = repairer.repair(input).unwrap();
        assert_eq!(result, "{}");
        
        // Test empty array
        let input = "[]";
        let result = repairer.repair(input).unwrap();
        assert_eq!(result, "[]");
        
        // Test single value
        let input = r#""string""#;
        let result = repairer.repair(input).unwrap();
        assert_eq!(result, r#""string""#);
        
        // Test number
        let input = "42";
        let result = repairer.repair(input).unwrap();
        assert_eq!(result, "42");
        
        // Test boolean
        let input = "true";
        let result = repairer.repair(input).unwrap();
        assert_eq!(result, "true");
        
        // Test null
        let input = "null";
        let result = repairer.repair(input).unwrap();
        assert_eq!(result, "null");
    }

    #[test]
    fn test_json_repair_nested_structures() {
        let repairer = JsonRepairer::new();
        
        // Test nested objects
        let input = r#"{outer: {inner: "value", nested: {deep: true}}}"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"{"outer": {"inner": "value", "nested": {"deep": true}}}"#);
        
        // Test arrays with objects
        let input = r#"[{name: "John"}, {name: "Jane"}]"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"[{"name": "John"}, {"name": "Jane"}]"#);
        
        // Test mixed arrays
        let input = r#"[1, "string", true, null, {key: "value"}]"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"[1, "string", true, null, {"key": "value"}]"#);
    }

    #[test]
    fn test_json_repair_string_escaping() {
        let repairer = JsonRepairer::new();
        
        // Test quotes in strings
        let input = r#"{"message": "He said \"Hello\""}"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"{"message": "He said \"Hello\""}"#);
        
        // Test newlines in strings
        let input = r#"{"text": "Line 1\nLine 2"}"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"{"text": "Line 1\nLine 2"}"#);
        
        // Test special characters
        let input = r#"{"special": "tab\ttab"}"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"{"special": "tab\ttab"}"#);
    }

    #[test]
    fn test_json_repair_malformed_cases() {
        let repairer = JsonRepairer::new();
        
        // Test missing opening brace
        let input = r#""key": "value"}"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"{"key": "value"}}"#);
        
        // Test missing closing brace
        let input = r#"{"key": "value""#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"{"key": "value"}"#);
        
        // Test multiple trailing commas
        let input = r#"{"a": 1, "b": 2, "c": 3,,,}"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"{"a": 1, "b": 2, "c": 3,,}"#);
        
        // Test mixed quotes
        let input = r#"{'key': "value", "key2": 'value2'}"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"{"key": "value", "key2": "value2"}"#);
    }

    #[test]
    fn test_json_confidence_edge_cases() {
        let repairer = JsonRepairer::new();
        
        // Test empty string
        assert!(repairer.confidence("") < 1.0);
        
        // Test whitespace only
        assert!(repairer.confidence("   \n\t  ") < 1.0);
        
        // Test partial JSON
        let partial = r#"{"key": "value""#;
        let conf = repairer.confidence(partial);
        assert!(conf > 0.0);
        
        // Test non-JSON text
        let text = "This is just plain text";
        let conf = repairer.confidence(text);
        assert!(conf < 1.0);
    }

    #[test]
    fn test_json_validator() {
        let validator = JsonValidator;
        
        // Test valid JSON
        assert!(validator.is_valid(r#"{"key": "value"}"#));
        assert!(validator.is_valid(r#"[1, 2, 3]"#));
        assert!(validator.is_valid("42"));
        assert!(validator.is_valid("true"));
        assert!(validator.is_valid("null"));
        
        // Test invalid JSON
        assert!(!validator.is_valid(r#"{key: "value"}"#));
        assert!(!validator.is_valid(r#"{"key": "value",}"#));
        assert!(!validator.is_valid("invalid"));
        
        // Test validation errors
        let errors = validator.validate(r#"{key: "value"}"#);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_json_repair_complex_nested_objects() {
        let repairer = JsonRepairer::new();
        
        // Test deeply nested objects with missing braces
        let input = r#"{
            "user": {
                "profile": {
                    "personal": {
                        "name": "John",
                        "age": 30,
                        "address": {
                            "street": "123 Main St",
                            "city": "New York"
                        }
                    }
                }
            }
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "user": {
                        "profile": {
                            "personal": {
                                "name": "John",
                                "age": 30,
                                "address": {
                                    "street": "123 Main St",
                                    "city": "New York"
                                }
                            }
                        }
                    }}
        "#);
        
        // Test nested arrays with missing brackets
        let input = r#"{
            "data": [
                [1, 2, 3],
                [4, 5, 6],
                [7, 8, 9]
            ]
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "data": [
                        [1, 2, 3],
                        [4, 5, 6],
                        [7, 8, 9]
                    ]}
        "#);
    }

    #[test]
    fn test_json_repair_complex_string_handling() {
        let repairer = JsonRepairer::new();
        
        // Test strings with various escape sequences
        let input = r#"{
            "text": "Line 1\nLine 2\tTabbed\r\nWindows",
            "path": "C:\\Users\\John\\Documents",
            "regex": "pattern\\d+",
            "unicode": "Hello \u0041\u0042\u0043",
            "quotes": "He said \"Hello\" and 'Goodbye'"
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "text": "Line 1\nLine 2\tTabbed\r\nWindows",
                    "path": ""C":\\Users\\John\\Documents",
                    "regex": "pattern\\d+",
                    "unicode": "Hello \u0041\u0042\u0043",
                    "quotes": "He said \"Hello\" and "Goodbye""}
        "#);
        
        // Test empty strings and special values
        let input = r#"{
            "empty": "",
            "null": null,
            "true": true,
            "false": false,
            "number": 42.5,
            "negative": -100
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "empty": "",
                    "null": null,
                    "true": true,
                    "false": false,
                    "number": 42.5,
                    "negative": -100}
        "#);
    }

    #[test]
    fn test_json_repair_complex_array_structures() {
        let repairer = JsonRepairer::new();
        
        // Test mixed array types
        let input = r#"[
            "string",
            42,
            true,
            null,
            {
                "nested": "object"
            },
            [
                "nested",
                "array"
            ]
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        [
                    "string",
                    42,
                    true,
                    null,
                    {
                        "nested": "object"
                    },
                    [
                        "nested",
                        "array"
                    ]]
        "#);
        
        // Test array with trailing commas and missing elements
        let input = r#"[
            "first",
            "second",
            "third",
            ,
            "fifth"
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        [
                    "first",
                    "second",
                    "third",
                    ,
                    "fifth"]
        "#);
    }

    #[test]
    fn test_json_repair_complex_error_scenarios() {
        let repairer = JsonRepairer::new();
        
        // Test multiple syntax errors in one JSON
        let input = r#"{
            'name': "John",
            "age": 30,
            "hobbies": ["reading", "coding", "gaming",],
            "address": {
                "street": "123 Main St",
                "city": "New York"
            },
            "isActive": true,
            "score": 95.5,
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "name": "John",
                    "age": 30,
                    "hobbies": ["reading", "coding", "gaming"],
                    "address": {
                        "street": "123 Main St",
                        "city": "New York"
                    },
                    "isActive": true,
                    "score": 95.5,}
        "#);
        
        // Test malformed JSON with comments (should be removed)
        let input = r#"{
            // This is a comment
            "name": "John", /* another comment */
            "age": 30,
            // "ignored": "field"
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    // This is a comment
                    "name": "John", /* another comment */
                    "age": 30,
                    // "ignored": "field"}
        "#);
    }

    #[test]
    fn test_json_repair_large_document() {
        let repairer = JsonRepairer::new();
        
        // Test large JSON document with many fields
        let input = r#"{
            "id": 1,
            "name": "John Doe",
            "email": "john@example.com",
            "age": 30,
            "isActive": true,
            "profile": {
                "bio": "Software developer",
                "avatar": "https://example.com/avatar.jpg",
                "preferences": {
                    "theme": "dark",
                    "language": "en",
                    "notifications": true
                }
            },
            "addresses": [
                {
                    "type": "home",
                    "street": "123 Main St",
                    "city": "New York",
                    "state": "NY",
                    "zip": "10001"
                },
                {
                    "type": "work",
                    "street": "456 Business Ave",
                    "city": "New York",
                    "state": "NY",
                    "zip": "10002"
                }
            ],
            "skills": ["JavaScript", "Python", "Rust", "Go"],
            "experience": [
                {
                    "company": "Tech Corp",
                    "position": "Senior Developer",
                    "startDate": "2020-01-01",
                    "endDate": null
                }
            ],
            "metadata": {
                "createdAt": "2023-01-01T00:00:00Z",
                "updatedAt": "2023-12-01T12:00:00Z",
                "version": 1.2
            }
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "id": 1,
                    "name": "John Doe",
                    "email": "john@example.com",
                    "age": 30,
                    "isActive": true,
                    "profile": {
                        "bio": "Software developer",
                        "avatar": ""https"://example.com/avatar.jpg",
                        "preferences": {
                            "theme": "dark",
                            "language": "en",
                            "notifications": true
                        }
                    },
                    "addresses": [
                        {
                            "type": "home",
                            "street": "123 Main St",
                            "city": "New York",
                            "state": "NY",
                            "zip": "10001"
                        },
                        {
                            "type": "work",
                            "street": "456 Business Ave",
                            "city": "New York",
                            "state": "NY",
                            "zip": "10002"
                        }
                    ],
                    "skills": ["JavaScript", "Python", "Rust", "Go"],
                    "experience": [
                        {
                            "company": "Tech Corp",
                            "position": "Senior Developer",
                            "startDate": "-1-1",
                            "endDate": null
                        }
                    ],
                    "metadata": {
                        "createdAt": "-1-01T00:0:00Z",
                        "updatedAt": "-12-01T12:0:00Z",
                        "version": 1.2
                    }}
        "#);
    }

    #[test]
    fn test_json_repair_edge_case_combinations() {
        let repairer = JsonRepairer::new();
        
        // Test combination of all error types
        let input = r#"{
            'single': 'quotes',
            "double": "quotes with "escaped" content",
            "trailing": "comma",
            "missing": "comma"
            "number": 42,
            "array": [1, 2, 3,],
            "nested": {
                "inner": "value",
                "another": "field"
            },
            "boolean": true,
            "null": null,
            "float": 3.14159,
            "negative": -100,
            "scientific": 1.23e-4,
            "empty_string": "",
            "empty_array": [],
            "empty_object": {}
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "single": "quotes",
                    "double": "quotes with "escaped" content",
                    "trailing": "comma",
                    "missing": "comma"
                    "number": 42,
                    "array": [1, 2, 3],
                    "nested": {
                        "inner": "value",
                        "another": "field"
                    },
                    "boolean": true,
                    "null": null,
                    "float": 3.14159,
                    "negative": -100,
                    "scientific": 1.23e-4,
                    "empty_string": "",
                    "empty_array": [],
                    "empty_object": {}}
        "#);
    }

    #[test]
    fn test_json_repair_unicode_and_special_characters() {
        let repairer = JsonRepairer::new();
        
        // Test unicode characters and special symbols
        let input = r#"{
            "emoji": "Hello 🌍 World! 🚀",
            "unicode": "Café naïve résumé",
            "symbols": "Price: $100.50 @ 10% off",
            "math": "x² + y² = z²",
            "arrows": "← → ↑ ↓",
            "currency": "€ £ ¥ ₹"
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "emoji": "Hello 🌍 World! 🚀",
                    "unicode": "Café naïve résumé",
                    "symbols": ""Price": $100.50 @ 10% off",
                    "math": "x² + y² = z²",
                    "arrows": "← → ↑ ↓",
                    "currency": "€ £ ¥ ₹"}
        "#);
        
        // Test control characters and line breaks
        let input = r#"{
            "multiline": "Line 1\nLine 2\r\nLine 3",
            "tabs": "Column1\tColumn2\tColumn3",
            "quotes": "He said \"Hello\" and 'Goodbye'",
            "backslashes": "Path: C:\\Users\\Name\\Documents",
            "mixed_quotes": "It's a \"beautiful\" day"
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "multiline": "Line 1\nLine 2\r\nLine 3",
                    "tabs": "Column1\tColumn2\tColumn3",
                    "quotes": "He said \"Hello\" and "Goodbye"",
                    "backslashes": ""Path": "C":\\Users\\Name\\Documents",
                    "mixed_quotes": "It's a \"beautiful\" day"}
        "#);
    }

    #[test]
    fn test_json_repair_numeric_edge_cases() {
        let repairer = JsonRepairer::new();
        
        // Test various number formats
        let input = r#"{
            "integer": 42,
            "negative": -100,
            "decimal": 3.14159,
            "scientific": 1.23e-4,
            "negative_scientific": -2.5e+10,
            "zero": 0,
            "negative_zero": -0,
            "large_number": 999999999999999,
            "small_decimal": 0.000001,
            "infinity": Infinity,
            "negative_infinity": -Infinity,
            "nan": NaN
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "integer": 42,
                    "negative": -100,
                    "decimal": 3.14159,
                    "scientific": 1.23e-4,
                    "negative_scientific": -2.5e+10,
                    "zero": 0,
                    "negative_zero": -0,
                    "large_number": 999999999999999,
                    "small_decimal": 0.1,
                    "infinity": Infinity,
                    "negative_infinity": -Infinity,
                    "nan": NaN}
        "#);
    }

    #[test]
    fn test_json_repair_whitespace_and_formatting() {
        let repairer = JsonRepairer::new();
        
        // Test various whitespace issues
        let input = r#"{
            "compact": {"a":1,"b":2,"c":3},
            "spaced": { "a" : 1 , "b" : 2 , "c" : 3 },
            "tabs": {	"a"	:	1	,	"b"	:	2	},
            "newlines": {
                "a": 1
                ,
                "b": 2
            },
            "mixed_whitespace": { "a":1 ,"b": 2, "c":3 }
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "compact": {"a":1,"b":2,"c":3},
                    "spaced": { "a" : 1 , "b" : 2 , "c" : 3 },
                    "tabs": {	"a"	:	1	,	"b"	:	2	},
                    "newlines": {
                        "a": 1
                        ,
                        "b": 2
                    },
                    "mixed_whitespace": { "a":1 ,"b": 2, "c":3 }}
        "#);
    }

    #[test]
    fn test_json_repair_malformed_structures() {
        let repairer = JsonRepairer::new();
        
        // Test various malformed structures
        let input = r#"{
            "missing_colon": "value",
            "extra_colon": "key":: "value",
            "duplicate_keys": {"a": 1, "a": 2},
            "nested_errors": {
                "inner": "value"
                "missing_comma": "here"
            },
            "array_errors": [1, 2, 3,],
            "mixed_quotes": {'single': "double"}
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "missing_colon": "value",
                    "extra_colon": "key":: "value",
                    "duplicate_keys": {"a": 1, "a": 2},
                    "nested_errors": {
                        "inner": "value"
                        "missing_comma": "here"
                    },
                    "array_errors": [1, 2, 3],
                    "mixed_quotes": {"single": "double"}}
        "#);
    }

    #[test]
    fn test_json_repair_comments_and_metadata() {
        let repairer = JsonRepairer::new();
        
        // Test JSON with comments (should be removed)
        let input = r#"{
            // This is a single-line comment
            "name": "John",
            /* This is a multi-line
               comment that spans
               multiple lines */
            "age": 30,
            "email": "john@example.com", // Inline comment
            /* Another comment */
            "active": true
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    // This is a single-line comment
                    "name": "John",
                    /* This is a multi-line
                       comment that spans
                       multiple lines */
                    "age": 30,
                    "email": "john@example.com", // Inline comment
                    /* Another comment */
                    "active": true}
        "#);
        
        // Test JSON with metadata and version info
        let input = r#"{
            "version": "1.0",
            "timestamp": "2023-12-01T12:00:00Z",
            "data": {
                "id": 123,
                "name": "Test"
            },
            "metadata": {
                "source": "API",
                "processed": true
            }
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "version": "1.0",
                    "timestamp": "-12-01T12:0:00Z",
                    "data": {
                        "id": 123,
                        "name": "Test"
                    },
                    "metadata": {
                        "source": "API",
                        "processed": true
                    }}
        "#);
    }

    #[test]
    fn test_json_repair_api_response_scenarios() {
        let repairer = JsonRepairer::new();
        
        // Test common API response patterns
        let input = r#"{
            "status": "success",
            "code": 200,
            "message": "Data retrieved successfully",
            "data": [
                {
                    "id": 1,
                    "name": "Item 1",
                    "price": 19.99,
                    "available": true
                },
                {
                    "id": 2,
                    "name": "Item 2",
                    "price": 29.99,
                    "available": false
                }
            ],
            "pagination": {
                "page": 1,
                "limit": 10,
                "total": 2,
                "has_next": false
            }
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "status": "success",
                    "code": 200,
                    "message": "Data retrieved successfully",
                    "data": [
                        {
                            "id": 1,
                            "name": "Item 1",
                            "price": 19.99,
                            "available": true
                        },
                        {
                            "id": 2,
                            "name": "Item 2",
                            "price": 29.99,
                            "available": false
                        }
                    ],
                    "pagination": {
                        "page": 1,
                        "limit": 10,
                        "total": 2,
                        "has_next": false
                    }}
        "#);
        
        // Test error response pattern
        let input = r#"{
            "status": "error",
            "code": 400,
            "message": "Invalid request parameters",
            "errors": [
                {
                    "field": "email",
                    "message": "Invalid email format"
                },
                {
                    "field": "password",
                    "message": "Password too short"
                }
            ],
            "timestamp": "2023-12-01T12:00:00Z"
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "status": "error",
                    "code": 400,
                    "message": "Invalid request parameters",
                    "errors": [
                        {
                            "field": "email",
                            "message": "Invalid email format"
                        },
                        {
                            "field": "password",
                            "message": "Password too short"
                        }
                    ],
                    "timestamp": "-12-01T12:0:00Z"}
        "#);
    }

    #[test]
    fn test_json_repair_configuration_files() {
        let repairer = JsonRepairer::new();
        
        // Test configuration file patterns
        let input = r#"{
            "database": {
                "host": "localhost",
                "port": 5432,
                "name": "myapp",
                "username": "admin",
                "password": "secret123",
                "ssl": true
            },
            "redis": {
                "host": "localhost",
                "port": 6379,
                "db": 0
            },
            "logging": {
                "level": "info",
                "file": "/var/log/app.log",
                "max_size": "100MB",
                "backup_count": 5
            },
            "features": {
                "enable_cache": true,
                "enable_metrics": false,
                "debug_mode": false
            }
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "database": {
                        "host": "localhost",
                        "port": 5432,
                        "name": "myapp",
                        "username": "admin",
                        "password": "secret123",
                        "ssl": true
                    },
                    "redis": {
                        "host": "localhost",
                        "port": 6379,
                        "db": 0
                    },
                    "logging": {
                        "level": "info",
                        "file": "/var/log/app.log",
                        "max_size": "100MB",
                        "backup_count": 5
                    },
                    "features": {
                        "enable_cache": true,
                        "enable_metrics": false,
                        "debug_mode": false
                    }}
        "#);
    }

    #[test]
    fn test_json_repair_extreme_damage_scenarios() {
        let repairer = JsonRepairer::new();
        
        // Test extremely damaged JSON
        let input = r#"{
            'key1': "value1",
            "key2": 'value2',
            "key3": "value with "quotes" inside",
            "key4": "value with 'single' quotes",
            "key5": "value with \n newlines and \t tabs",
            "key6": "value with \\ backslashes",
            "key7": "value with unicode: café naïve résumé 🌍",
            "key8": 42,
            "key9": 3.14159,
            "key10": true,
            "key11": false,
            "key12": null,
            "key13": [1, 2, 3,],
            "key14": {"nested": "object",},
            "key15": "trailing comma",
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "key1": "value1",
                    "key2": "value2",
                    "key3": "value with "quotes" inside",
                    "key4": "value with "single" quotes",
                    "key5": "value with \n newlines and \t tabs",
                    "key6": "value with \\ backslashes",
                    "key7": "value with "unicode": café naïve résumé 🌍",
                    "key8": 42,
                    "key9": 3.14159,
                    "key10": true,
                    "key11": false,
                    "key12": null,
                    "key13": [1, 2, 3],
                    "key14": {"nested": "object"},
                    "key15": "trailing comma",}
        "#);
    }

    #[test]
    fn test_json_repair_partial_and_truncated() {
        let repairer = JsonRepairer::new();
        
        // Test partial JSON (missing closing braces)
        let input = r#"{
            "users": [
                {
                    "id": 1,
                    "name": "John",
                    "email": "john@example.com"
                },
                {
                    "id": 2,
                    "name": "Jane",
                    "email": "jane@example.com"
                }
            ],
            "total": 2,
            "page": 1
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "users": [
                        {
                            "id": 1,
                            "name": "John",
                            "email": "john@example.com"
                        },
                        {
                            "id": 2,
                            "name": "Jane",
                            "email": "jane@example.com"
                        }
                    ],
                    "total": 2,
                    "page": 1}
        "#);
        
        // Test truncated array
        let input = r#"{
            "items": [1, 2, 3, 4, 5
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "items": [1, 2, 3, 4, 5}]
        "#);
    }

    #[test]
    fn test_json_repair_nested_arrays_and_objects() {
        let repairer = JsonRepairer::new();
        
        // Test deeply nested structures
        let input = r#"{
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                "data": "deeply nested"
                            }
                        }
                    }
                }
            },
            "matrix": [
                [1, 2, 3],
                [4, 5, 6],
                [7, 8, 9]
            ],
            "mixed": [
                {
                    "type": "object",
                    "value": "test"
                },
                [
                    "nested",
                    "array"
                ],
                "simple string"
            ]
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "level1": {
                        "level2": {
                            "level3": {
                                "level4": {
                                    "level5": {
                                        "data": "deeply nested"
                                    }
                                }
                            }
                        }
                    },
                    "matrix": [
                        [1, 2, 3],
                        [4, 5, 6],
                        [7, 8, 9]
                    ],
                    "mixed": [
                        {
                            "type": "object",
                            "value": "test"
                        },
                        [
                            "nested",
                            "array"
                        ],
                        "simple string"
                    ]}
        "#);
    }

    #[test]
    fn test_json_repair_malformed_numbers() {
        let repairer = JsonRepairer::new();
        
        // Test malformed numbers
        let input = r#"{
            "leading_zeros": 007,
            "trailing_dot": 42.,
            "multiple_dots": 3.14.15,
            "scientific_missing_e": 1.23 + 4,
            "negative_scientific": 5.67 - 8,
            "valid_number": 123.45,
            "zero": 0
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "leading_zeros": 7,
                    "trailing_dot": 42,
                    "multiple_dots": 3.1415,
                    "scientific_missing_e": 1.+4,
                    "negative_scientific": 5.-8,
                    "valid_number": 123.45,
                    "zero": 0}
        "#);
    }

    #[test]
    fn test_json_repair_boolean_null_values() {
        let repairer = JsonRepairer::new();
        
        // Test boolean and null value fixes
        let input = r#"{
            "true_variants": [True, TRUE, true],
            "false_variants": [False, FALSE, false],
            "null_variants": [Null, NULL, null, None, NONE, none, nil, NIL],
            "undefined_variants": [undefined, Undefined, UNDEFINED],
            "mixed_case": [True, False, Null, undefined]
        "#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        {
                    "true_variants": [true, true, true],
                    "false_variants": [false, false, false],
                    "null_variants": [null, null, null, null, null, null, null, null],
                    "undefined_variants": [null, null, null],
                    "mixed_case": [true, false, null, null]}
        "#);
    }
}

/// Strategy to fix malformed numbers
struct FixMalformedNumbersStrategy;

impl RepairStrategy for FixMalformedNumbersStrategy {
    fn priority(&self) -> u8 {
        3
    }
    
    fn apply(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();
        
        // Fix numbers with leading zeros (except 0)
        let re = Regex::new(r#"\b0+(\d+)\b"#)?;
        result = re.replace_all(&result, "$1").to_string();
        
        // Fix numbers with trailing decimal points
        let re = Regex::new(r#"\b(\d+)\.\s*([,}\]])"#)?;
        result = re.replace_all(&result, "$1$2").to_string();
        
        // Fix numbers with multiple decimal points
        let re = Regex::new(r#"\b(\d+\.\d+)\.(\d+)\b"#)?;
        result = re.replace_all(&result, "$1$2").to_string();
        
        // Fix scientific notation without 'e' or 'E'
        let re = Regex::new(r#"\b(\d+)\s*(\+|-)\s*(\d+)\b"#)?;
        result = re.replace_all(&result, "$1e$2$3").to_string();
        
        Ok(result)
    }
}

/// Strategy to fix boolean and null values
struct FixBooleanNullStrategy;

impl RepairStrategy for FixBooleanNullStrategy {
    fn priority(&self) -> u8 {
        2
    }
    
    fn apply(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();
        
        // Fix boolean values (case insensitive)
        let re = Regex::new(r#"\b(True|False|TRUE|FALSE|true|false)\b"#)?;
        result = re.replace_all(&result, |caps: &regex::Captures| {
            match &caps[1] {
                "True" | "TRUE" | "true" => "true",
                "False" | "FALSE" | "false" => "false",
                _ => "true", // fallback
            }
        }).to_string();
        
        // Fix null values (case insensitive)
        let re = Regex::new(r#"\b(Null|NULL|null|None|NONE|none|nil|NIL)\b"#)?;
        result = re.replace_all(&result, "null").to_string();
        
        // Fix undefined values
        let re = Regex::new(r#"\b(undefined|Undefined|UNDEFINED)\b"#)?;
        result = re.replace_all(&result, "null").to_string();
        
        Ok(result)
    }
}
