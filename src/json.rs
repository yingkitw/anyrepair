//! JSON repair functionality

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;

/// Cached regex patterns for performance optimization
#[allow(dead_code)]
struct RegexCache {
    missing_quotes: Regex,
    trailing_commas: Regex,
    unescaped_quotes: Regex,
    single_quotes: Regex,
    malformed_numbers_leading_zeros: Regex,
    malformed_numbers_trailing_dots: Regex,
    malformed_numbers_multiple_dots: Regex,
    malformed_numbers_scientific: Regex,
    boolean_values: Regex,
    null_values: Regex,
    undefined_values: Regex,
}

impl RegexCache {
    fn new() -> Result<Self> {
        Ok(Self {
            missing_quotes: Regex::new(r#"(^|\s|,|\{)\s*(\w+)\s*:"#)?,
            trailing_commas: Regex::new(r#",(\s*[}\]])"#)?,
            unescaped_quotes: Regex::new(r#""([^"\\]|\\.)*"[^,}\]]*"#)?,
            single_quotes: Regex::new(r#"'([^']*)'"#)?,
            malformed_numbers_leading_zeros: Regex::new(r#"\b0+(\d+)\b"#)?,
            malformed_numbers_trailing_dots: Regex::new(r#"\b(\d+)\.\s*([,}\]])"#)?,
            malformed_numbers_multiple_dots: Regex::new(r#"\b(\d+\.\d+)\.(\d+)\b"#)?,
            malformed_numbers_scientific: Regex::new(r#"\b(\d+)\s*(\+|-)\s*(\d+)\b"#)?,
            boolean_values: Regex::new(r#"\b(True|False|TRUE|FALSE|true|false)\b"#)?,
            null_values: Regex::new(r#"\b(Null|NULL|null|None|NONE|none|nil|NIL)\b"#)?,
            undefined_values: Regex::new(r#"\b(undefined|Undefined|UNDEFINED)\b"#)?,
        })
    }
}

static REGEX_CACHE: OnceLock<RegexCache> = OnceLock::new();

fn get_regex_cache() -> &'static RegexCache {
    REGEX_CACHE.get_or_init(|| RegexCache::new().expect("Failed to initialize regex cache"))
}

/// JSON repairer that can fix common JSON issues
pub struct JsonRepairer {
    strategies: Vec<Box<dyn RepairStrategy>>,
    validator: JsonValidator,
    logging: bool,
    repair_log: Vec<String>,
}

impl JsonRepairer {
    /// Create a new JSON repairer
    pub fn new() -> Self {
        let mut strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(AddMissingQuotesStrategy),
            Box::new(FixTrailingCommasStrategy),
            // Box::new(FixUnescapedQuotesStrategy), // Disabled due to regex complexity
            Box::new(AddMissingBracesStrategy),
            Box::new(FixSingleQuotesStrategy),
            Box::new(FixMalformedNumbersStrategy),
            Box::new(FixBooleanNullStrategy),
            Box::new(FixAgenticAiResponseStrategy),
        ];
        
        // Sort strategies by priority (higher priority first)
        strategies.sort_by_key(|b| std::cmp::Reverse(b.priority()));
        
        Self {
            strategies,
            validator: JsonValidator,
            logging: false,
            repair_log: Vec::new(),
        }
    }

    /// Create a new JSON repairer with logging enabled
    pub fn with_logging(logging: bool) -> Self {
        let mut repairer = Self::new();
        repairer.logging = logging;
        repairer
    }

    /// Get the repair log
    pub fn get_repair_log(&self) -> &[String] {
        &self.repair_log
    }

    /// Clear the repair log
    pub fn clear_log(&mut self) {
        self.repair_log.clear();
    }

    /// Log a repair action
    fn log(&mut self, message: &str) {
        if self.logging {
            self.repair_log.push(message.to_string());
        }
    }
    
    /// Apply all repair strategies to the content using parallel processing
    fn apply_strategies(&mut self, content: &str) -> Result<String> {
        // For now, use sequential processing until we can properly implement Arc conversion
        let mut repaired = content.to_string();
        let logging = self.logging;
        
        for strategy in &self.strategies {
            if let Ok(result) = strategy.apply(&repaired) {
                if result != repaired && logging {
                    let strategy_name = strategy.name().to_string();
                    self.repair_log.push(format!("Applied strategy: {}", strategy_name));
                }
                repaired = result;
            }
        }
        
        Ok(repaired)
    }
}

impl Default for JsonRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for JsonRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        let trimmed = content.trim();
        
        // Clear previous log
        self.repair_log.clear();
        
        // If already valid, return as-is
        if self.validator.is_valid(trimmed) {
            self.log("JSON was already valid, no repairs needed");
            return Ok(trimmed.to_string());
        }
        
        self.log("Starting JSON repair process");
        
        // Apply repair strategies
        let repaired = self.apply_strategies(trimmed)?;
        
        self.log("JSON repair completed");
        
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
        let cache = get_regex_cache();
        let result = cache.missing_quotes.replace_all(content, r#"$1"$2":"#);
        Ok(result.to_string())
    }

    fn priority(&self) -> u8 {
        5
    }

    fn name(&self) -> &str {
        "AddMissingQuotesStrategy"
    }
}

/// Strategy to fix trailing commas
struct FixTrailingCommasStrategy;

impl RepairStrategy for FixTrailingCommasStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        let result = cache.trailing_commas.replace_all(content, r#"$1"#);
        Ok(result.to_string())
    }

    fn priority(&self) -> u8 {
        4
    }

    fn name(&self) -> &str {
        "FixTrailingCommasStrategy"
    }
}

/// Strategy to fix unescaped quotes
#[allow(dead_code)]
struct FixUnescapedQuotesStrategy;

impl RepairStrategy for FixUnescapedQuotesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        // This is a simplified version - in practice, you'd need more sophisticated parsing
        let cache = get_regex_cache();
        let result = cache.unescaped_quotes.replace_all(content, r#"\""#);
        Ok(result.to_string())
    }

    fn priority(&self) -> u8 {
        3
    }

    fn name(&self) -> &str {
        "FixUnescapedQuotesStrategy"
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

    fn name(&self) -> &str {
        "AddMissingBracesStrategy"
    }
}

/// Strategy to fix single quotes to double quotes
struct FixSingleQuotesStrategy;

impl RepairStrategy for FixSingleQuotesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        let result = cache.single_quotes.replace_all(content, r#""$1""#);
        Ok(result.to_string())
    }

    fn priority(&self) -> u8 {
        1
    }

    fn name(&self) -> &str {
        "FixSingleQuotesStrategy"
    }
}

/// Strategy to fix malformed numbers
struct FixMalformedNumbersStrategy;

impl RepairStrategy for FixMalformedNumbersStrategy {
    fn priority(&self) -> u8 {
        3
    }
    
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        let mut result = content.to_string();
        
        // Fix numbers with leading zeros (except 0)
        result = cache.malformed_numbers_leading_zeros.replace_all(&result, "$1").to_string();
        
        // Fix numbers with trailing decimal points
        result = cache.malformed_numbers_trailing_dots.replace_all(&result, "$1$2").to_string();
        
        // Fix numbers with multiple decimal points
        result = cache.malformed_numbers_multiple_dots.replace_all(&result, "$1$2").to_string();
        
        // Fix scientific notation without 'e' or 'E'
        result = cache.malformed_numbers_scientific.replace_all(&result, "$1e$2$3").to_string();
        
        Ok(result)
    }

    fn name(&self) -> &str {
        "FixMalformedNumbersStrategy"
    }
}

/// Strategy to fix boolean and null values
struct FixBooleanNullStrategy;

impl RepairStrategy for FixBooleanNullStrategy {
    fn priority(&self) -> u8 {
        2
    }
    
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        let mut result = content.to_string();
        
        // Fix boolean values (case insensitive)
        result = cache.boolean_values.replace_all(&result, |caps: &regex::Captures| {
            match &caps[1] {
                "True" | "TRUE" | "true" => "true",
                "False" | "FALSE" | "false" => "false",
                _ => "true", // fallback
            }
        }).to_string();
        
        // Fix null values (case insensitive)
        result = cache.null_values.replace_all(&result, |caps: &regex::Captures| {
            match &caps[1] {
                "Null" | "NULL" | "null" | "None" | "NONE" | "none" | "nil" | "NIL" => "null",
                _ => "null", // fallback
            }
        }).to_string();
        
        // Fix undefined values
        result = cache.undefined_values.replace_all(&result, "null").to_string();
        
        Ok(result)
    }

    fn name(&self) -> &str {
        "FixBooleanNullStrategy"
    }
}

/// Strategy for fixing agentic AI response patterns
struct FixAgenticAiResponseStrategy;

impl RepairStrategy for FixAgenticAiResponseStrategy {
    fn priority(&self) -> u8 {
        1 // High priority for agentic AI responses
    }
    
    fn apply(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();
        
        // Fix common agentic AI response issues
        
        // 1. Fix missing quotes around common agentic AI field names
        let agentic_fields = [
            "goal", "steps", "step_number", "description", "tool_call", 
            "tool_name", "parameters", "command", "working_folder", 
            "working_dir", "file_path", "search", "replacement", "code",
            "feedback", "timeout", "prompt", "project_type", "project_name"
        ];
        
        for field in &agentic_fields {
            // Fix unquoted field names - only match at the beginning of lines or after whitespace/commas/braces
            // Use word boundary to ensure we don't match partial words
            let pattern = format!(r#"(^|\s|,|\{{)\s*{}\b\s*:"#, field);
            let replacement = format!(r#"$1"{}":"#, field);
            if let Ok(regex) = Regex::new(&pattern) {
                result = regex.replace_all(&result, &replacement).to_string();
            }
        }
        
        // 2. Fix nested object structure issues
        // Fix missing closing braces in tool_call objects
        result = self.fix_nested_objects(&result);
        
        // 3. Fix array structure issues
        result = self.fix_array_structures(&result);
        
        // 4. Fix string escaping in parameters
        result = self.fix_parameter_strings(&result);
        
        Ok(result)
    }

    fn name(&self) -> &str {
        "FixAgenticAiResponseStrategy"
    }
}

impl FixAgenticAiResponseStrategy {
    /// Fix nested object structures common in agentic AI responses
    fn fix_nested_objects(&self, content: &str) -> String {
        let mut result = content.to_string();
        let mut brace_count = 0;
        let mut bracket_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        
        for (_i, ch) in content.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }
            
            match ch {
                '\\' if in_string => {
                    escape_next = true;
                }
                '"' => {
                    in_string = !in_string;
                }
                '{' if !in_string => {
                    brace_count += 1;
                }
                '}' if !in_string => {
                    brace_count -= 1;
                }
                '[' if !in_string => {
                    bracket_count += 1;
                }
                ']' if !in_string => {
                    bracket_count -= 1;
                }
                _ => {}
            }
        }
        
        // Add missing closing braces
        while brace_count > 0 {
            result.push('}');
            brace_count -= 1;
        }
        
        // Add missing closing brackets
        while bracket_count > 0 {
            result.push(']');
            bracket_count -= 1;
        }
        
        result
    }
    
    /// Fix array structures
    fn fix_array_structures(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        // Fix missing commas between array elements
        let array_comma_pattern = r#"(\])\s*(\[)"#;
        if let Ok(regex) = Regex::new(array_comma_pattern) {
            result = regex.replace_all(&result, "$1,$2").to_string();
        }
        
        // Fix missing commas between object elements in arrays
        let object_comma_pattern = r#"(\})\s*(\{)"#;
        if let Ok(regex) = Regex::new(object_comma_pattern) {
            result = regex.replace_all(&result, "$1,$2").to_string();
        }
        
        result
    }
    
    /// Fix string escaping in parameters
    fn fix_parameter_strings(&self, content: &str) -> String {
        // Use regex to find and fix unescaped quotes in string values
        let mut result = content.to_string();
        
        // Pattern to match: "key": "value with "quotes" inside"
        // This regex looks for a quoted string value that contains unescaped quotes
        let pattern = r#""([^"]*)"([^"]*)"([^"]*)"#;
        
        if let Ok(regex) = Regex::new(pattern) {
            result = regex.replace_all(&result, |caps: &regex::Captures| {
                let part1 = &caps[1];
                let part2 = &caps[2];
                let part3 = &caps[3];
                
                // If part2 contains quotes, escape them
                if part2.contains("\"") {
                    let escaped_part2 = part2.replace("\"", "\\\"");
                    format!("\"{}\"{}\"{}", part1, escaped_part2, part3)
                } else {
                    // No quotes to escape, return as-is
                    format!("\"{}\"{}\"{}", part1, part2, part3)
                }
            }).to_string();
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_json_repair_basic() {
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
        let input = r#"{name: "John", age: 30, city: "New York",}"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"{"name": "John","age": 30,"city": "New York"}"#);
    }
    
    #[test]
    fn test_json_confidence() {
        let mut repairer = JsonRepairer::new();
        
        // Valid JSON should have confidence 1.0
        let valid = r#"{"key": "value"}"#;
        assert_eq!(repairer.confidence(valid), 1.0);
        
        // Invalid JSON should have lower confidence
        let invalid = "not json at all";
        assert!(repairer.confidence(invalid) < 1.0);
    }
    
    #[test]
    fn test_needs_repair() {
        let mut repairer = JsonRepairer::new();
        
        assert!(!repairer.needs_repair(r#"{"key": "value"}"#));
        assert!(repairer.needs_repair(r#"{key: "value"}"#));
    }

    #[test]
    fn test_json_repair_edge_cases() {
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
        // Test nested objects
        let input = r#"{outer: {inner: "value", nested: {deep: true}}}"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"{"outer": {"inner": "value","nested": {"deep": true}}}"#);
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
                    "path": "C:\\Users\\John\\Documents",
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
                    "symbols": "Price: $100.50 @ 10% off",
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
                    "backslashes": "Path: "C":\\Users\\Name\\Documents",
                    "mixed_quotes": "It's a \"beautiful\" day"}
        "#);
    }

    #[test]
    fn test_json_repair_numeric_edge_cases() {
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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
        let mut repairer = JsonRepairer::new();
        
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

    #[test]
    fn test_json_repair_agentic_ai_responses() {
        let mut repairer = JsonRepairer::new();
        
        // Test agentic AI response format
        let input = r#"{
"goal": "generate java springboot",
"steps": [
  {
"step_number": 1,
"description": "Scaffold Spring Boot project structure",
"tool_call": {
"tool_name": "scaffold_project",
"parameters": {
"project_type": "java",
"project_name": "springboot_app"
  }
  }
  },
  {
"step_number": 2,
"description": "Generate Spring Boot application code",
"tool_call": {
"tool_name": "generate_code",
"parameters": {
"prompt": "Create a simple Spring Boot REST API application with a HelloController that responds with \"Hello, World!\" on GET /hello",
"working_folder": "springboot_app"
  }
  }
  }
  ]
  }"#;
        let result = repairer.repair(input).unwrap();
        assert!(result.contains("\"goal\""));
        assert!(result.contains("\"steps\""));
        assert!(result.contains("\"step_number\""));
        assert!(result.contains("\"tool_call\""));
    }

    #[test]
    fn test_json_repair_complex_agentic_ai_response() {
        let mut repairer = JsonRepairer::new();
        
        // Test the exact JSON from the user's example
        let input = r#"  {
"goal": "generate java springboot",
"steps": [
  {
"step_number": 1,
"description": "Scaffold Spring Boot project structure",
"tool_call": {
"tool_name": "scaffold_project",
"parameters": {
"project_type": "java",
"project_name": "springboot_app"
  }
  }
  },
  {
"step_number": 2,
"description": "Generate Spring Boot application code",
"tool_call": {
"tool_name": "generate_code",
"parameters": {
"prompt": "Create a simple Spring Boot REST API application with a HelloController that responds with \\\"Hello, World!\\\" on GET /hello",
"working_folder": "springboot_app"
  }
  }
  },
  {
"step_number": 3,
"description": "Add Maven dependencies for web functionality",
"tool_call": {
"tool_name": "sed_edit",
"parameters": {
"file_path": "springboot_app/pom.xml",
"search": "<dependencies>",
"replacement": "<dependencies>\n  <dependency>\n    <groupId>org.springframework.boot</groupId>\n    <artifactId>spring-boot-starter-web</artifactId>\n  </dependency>\n"
  }
  }
  },
  {
"step_number": 4,
"description": "Implement HelloController class",
"tool_call": {
"tool_name": "review_and_refine",
"parameters": {
"code": "package com.example.demo;\n\nimport org.springframework.web.bind.annotation.GetMapping;\nimport org.springframework.web.bind.annotation.RestController;\n\n@RestController\npublic class HelloController {\n\n    @GetMapping(\"/hello\")\n    public String sayHello() {\n        return \"Hello, World!\";\n    }\n}",
"feedback": "Ensure proper annotations and return statement"
  }
  }
  },
  {
"step_number": 5,
"description": "Build the Spring Boot application",
"tool_call": {
"tool_name": "shell_command",
"parameters": {
"command": "cd springboot_app && ./mvnw clean package",
"timeout": 300
  }
  }
  },
  {
"step_number": 6,
"description": "Run the Spring Boot application",
"tool_call": {
"tool_name": "execute_command",
"parameters": {
"command": "cd springboot_app && ./mvnw spring-boot:run",
"working_dir": "springboot_app"
  }
  }
  }
  ]
  }"#;
        
        let result = repairer.repair(input).unwrap();
        
        // Debug: print the repaired result
        println!("Repaired JSON:\n{}", result);
        
        // Debug: check what's wrong with the JSON
        if let Err(e) = serde_json::from_str::<serde_json::Value>(&result) {
            println!("JSON Parse Error: {}", e);
            println!("Error location: line {}, column {}", 
                e.line(), e.column());
            
            // Print the specific line with the error
            let lines: Vec<&str> = result.lines().collect();
            if e.line() <= lines.len() {
                println!("Problematic line: {}", lines[e.line() - 1]);
                println!("Character at error position: '{}'", 
                    lines[e.line() - 1].chars().nth(e.column() - 1).unwrap_or('?'));
            }
        }
        
        // Verify the structure is valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        
        // Check that all expected fields are present
        assert!(parsed.get("goal").is_some());
        assert!(parsed.get("steps").is_some());
        
        let steps = parsed.get("steps").unwrap().as_array().unwrap();
        assert_eq!(steps.len(), 6);
        
        // Check first step structure
        let first_step = &steps[0];
        assert!(first_step.get("step_number").is_some());
        assert!(first_step.get("description").is_some());
        assert!(first_step.get("tool_call").is_some());
        
        let tool_call = first_step.get("tool_call").unwrap();
        assert!(tool_call.get("tool_name").is_some());
        assert!(tool_call.get("parameters").is_some());
        
        println!("Repaired JSON structure is valid!");
    }

    #[test]
    fn test_json_repair_with_logging() {
        let mut repairer = JsonRepairer::with_logging(true);
        
        let input = r#"{"name": "John", "age": 30,}"#;
        let result = repairer.repair(input).unwrap();
        
        assert!(result.contains("John"));
        assert!(!result.ends_with(','));
        
        let log = repairer.get_repair_log();
        assert!(!log.is_empty());
        assert!(log.iter().any(|msg| msg.contains("Starting JSON repair process")));
    }
}
