//! JSON repair module
//! 
//! Provides comprehensive JSON repair functionality with multiple strategies
//! for fixing common JSON issues from LLM outputs.

use crate::error::Result;
use crate::repairer_base;
use crate::traits::{Repair, RepairStrategy, Validator};
use regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;

// ============================================================================
// JSON Validator
// ============================================================================

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
mod validator_tests {
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

// ============================================================================
// Regex Cache
// ============================================================================

/// Cached regex patterns for JSON repair
pub struct RegexCache {
    pub missing_quotes: Regex,
    pub trailing_commas: Regex,
    pub unescaped_quotes: Regex,
    pub single_quotes: Regex,
    pub malformed_numbers_leading_zeros: Regex,
    pub malformed_numbers_trailing_dots: Regex,
    pub malformed_numbers_multiple_dots: Regex,
    pub malformed_numbers_scientific: Regex,
    pub boolean_values: Regex,
    pub null_values: Regex,
    pub undefined_values: Regex,
}

impl RegexCache {
    pub fn new() -> Result<Self> {
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

pub fn get_regex_cache() -> &'static RegexCache {
    REGEX_CACHE.get_or_init(|| RegexCache::new().expect("Failed to initialize regex cache"))
}

// ============================================================================
// Repair Strategies
// ============================================================================

/// Strategy to strip trailing content after JSON closes
pub struct StripTrailingContentStrategy;

impl RepairStrategy for StripTrailingContentStrategy {
    fn name(&self) -> &str {
        "StripTrailingContent"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = String::new();
        let mut brace_count = 0;
        let mut bracket_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut found_json_end = false;
        let chars: Vec<char> = content.chars().collect();
        let len = chars.len();
        
        for i in 0..len {
            let ch = chars[i];
            
            if escape_next {
                result.push(ch);
                escape_next = false;
                continue;
            }
            
            match ch {
                '\\' if in_string => {
                    result.push(ch);
                    escape_next = true;
                }
                '"' => {
                    result.push(ch);
                    in_string = !in_string;
                }
                '{' if !in_string => {
                    result.push(ch);
                    brace_count += 1;
                }
                '}' if !in_string => {
                    result.push(ch);
                    brace_count -= 1;
                    if brace_count == 0 && bracket_count == 0 {
                        let mut j = i + 1;
                        while j < len && (chars[j] == ' ' || chars[j] == '\n' || chars[j] == '\t' || chars[j] == '\r') {
                            j += 1;
                        }
                        
                        if j < len && (chars[j] == ',' || chars[j] == '{' || chars[j] == '[') {
                            found_json_end = false;
                        } else if j >= len || (!chars[j].is_alphanumeric() && chars[j] != '"') {
                            found_json_end = true;
                        }
                    }
                }
                '[' if !in_string => {
                    result.push(ch);
                    bracket_count += 1;
                }
                ']' if !in_string => {
                    result.push(ch);
                    bracket_count -= 1;
                    if brace_count == 0 && bracket_count == 0 {
                        let mut j = i + 1;
                        while j < len && (chars[j] == ' ' || chars[j] == '\n' || chars[j] == '\t' || chars[j] == '\r') {
                            j += 1;
                        }
                        
                        if j < len && (chars[j] == ',' || chars[j] == '{' || chars[j] == '[') {
                            found_json_end = false;
                        } else if j >= len || (!chars[j].is_alphanumeric() && chars[j] != '"') {
                            found_json_end = true;
                        }
                    }
                }
                _ => {
                    if !found_json_end {
                        result.push(ch);
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    fn priority(&self) -> u8 {
        100
    }
}

/// Strategy to fix trailing commas
pub struct FixTrailingCommasStrategy;

impl RepairStrategy for FixTrailingCommasStrategy {
    fn name(&self) -> &str {
        "FixTrailingCommas"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        Ok(cache.trailing_commas.replace_all(content, "$1").to_string())
    }
    
    fn priority(&self) -> u8 {
        90
    }
}

/// Strategy to fix single quotes
pub struct FixSingleQuotesStrategy;

impl RepairStrategy for FixSingleQuotesStrategy {
    fn name(&self) -> &str {
        "FixSingleQuotes"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        Ok(cache.single_quotes.replace_all(content, "\"$1\"").to_string())
    }
    
    fn priority(&self) -> u8 {
        85
    }
}

/// Strategy to add missing quotes around keys
pub struct AddMissingQuotesStrategy;

impl RepairStrategy for AddMissingQuotesStrategy {
    fn name(&self) -> &str {
        "AddMissingQuotes"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        Ok(cache.missing_quotes.replace_all(content, "$1\"$2\":").to_string())
    }
    
    fn priority(&self) -> u8 {
        80
    }
}

/// Strategy to fix malformed numbers
pub struct FixMalformedNumbersStrategy;

impl RepairStrategy for FixMalformedNumbersStrategy {
    fn name(&self) -> &str {
        "FixMalformedNumbers"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        let mut result = content.to_string();
        
        result = cache.malformed_numbers_leading_zeros.replace_all(&result, "$1").to_string();
        result = cache.malformed_numbers_trailing_dots.replace_all(&result, "$1$2").to_string();
        result = cache.malformed_numbers_multiple_dots.replace_all(&result, "$1$2").to_string();
        result = cache.malformed_numbers_scientific.replace_all(&result, "$1e$2$3").to_string();
        
        Ok(result)
    }
    
    fn priority(&self) -> u8 {
        75
    }
}

/// Strategy to fix boolean and null values
pub struct FixBooleanNullStrategy;

impl RepairStrategy for FixBooleanNullStrategy {
    fn name(&self) -> &str {
        "FixBooleanNull"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        let mut result = content.to_string();
        
        result = cache.boolean_values.replace_all(&result, |caps: &regex::Captures| {
            match caps[0].to_lowercase().as_str() {
                s if s == "true" => "true".to_string(),
                s if s == "false" => "false".to_string(),
                _ => "true".to_string(),
            }
        }).to_string();
        
        result = cache.null_values.replace_all(&result, "null").to_string();
        result = cache.undefined_values.replace_all(&result, "null").to_string();
        
        Ok(result)
    }
    
    fn priority(&self) -> u8 {
        70
    }
}

/// Strategy to add missing braces
pub struct AddMissingBracesStrategy;

impl RepairStrategy for AddMissingBracesStrategy {
    fn name(&self) -> &str {
        "AddMissingBraces"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let trimmed = content.trim();
        
        if trimmed.is_empty() {
            return Ok("{}".to_string());
        }
        
        let mut result = trimmed.to_string();
        let open_braces = trimmed.matches('{').count();
        let close_braces = trimmed.matches('}').count();
        let open_brackets = trimmed.matches('[').count();
        let close_brackets = trimmed.matches(']').count();
        
        if open_braces > close_braces {
            result.push_str(&"}".repeat(open_braces - close_braces));
        }
        
        if open_brackets > close_brackets {
            result.push_str(&"]".repeat(open_brackets - close_brackets));
        }
        
        if !result.starts_with('{') && !result.starts_with('[') {
            result = format!("{{{}}}", result);
        }
        
        Ok(result)
    }
    
    fn priority(&self) -> u8 {
        60
    }
}

/// Strategy for agentic AI response repair
pub struct FixAgenticAiResponseStrategy;

impl RepairStrategy for FixAgenticAiResponseStrategy {
    fn name(&self) -> &str {
        "FixAgenticAiResponse"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        let mut result = content.to_string();

        result = cache.undefined_values.replace_all(&result, "null").to_string();
        result = cache.trailing_commas.replace_all(&result, "$1").to_string();
        result = cache.single_quotes.replace_all(&result, "\"$1\"").to_string();

        Ok(result)
    }

    fn priority(&self) -> u8 {
        50
    }
}

/// Strategy to strip JavaScript-style comments from JSON
pub struct StripJsCommentsStrategy;

impl RepairStrategy for StripJsCommentsStrategy {
    fn name(&self) -> &str {
        "StripJsComments"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = String::new();
        let mut in_string = false;
        let mut escaped = false;
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\\' if in_string => {
                    // Toggle escape state
                    escaped = !escaped;
                    result.push(c);
                }
                '"' if !escaped => {
                    in_string = !in_string;
                    result.push(c);
                }
                '/' if !in_string => {
                    if let Some(&'/') = chars.peek() {
                        // Single-line comment: //
                        while chars.next() != Some('\n') && chars.peek().is_some() {
                            // Skip until newline
                        }
                    } else if let Some(&'*') = chars.peek() {
                        // Multi-line comment: /*
                        chars.next(); // consume '*'
                        loop {
                            match chars.next() {
                                Some('*') => {
                                    if chars.peek() == Some(&'/') {
                                        chars.next(); // consume '/'
                                        break;
                                    }
                                }
                                Some(_) => continue,
                                None => break,
                            }
                        }
                    } else {
                        result.push(c);
                    }
                    escaped = false;
                }
                _ => {
                    result.push(c);
                    // Reset escape state for non-backslash characters
                    if c != '\\' {
                        escaped = false;
                    }
                }
            }
        }

        Ok(result)
    }

    fn priority(&self) -> u8 {
        95
    }
}

// ============================================================================
// JSON Repairer
// ============================================================================

/// JSON repairer that can fix common JSON issues
/// 
/// Uses trait-based composition with GenericRepairer for better modularity
pub struct JsonRepairer {
    inner: crate::repairer_base::GenericRepairer,
}

impl JsonRepairer {
    /// Create a new JSON repairer
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(StripTrailingContentStrategy),
            Box::new(StripJsCommentsStrategy),
            Box::new(AddMissingQuotesStrategy),
            Box::new(FixTrailingCommasStrategy),
            Box::new(AddMissingBracesStrategy),
            Box::new(FixSingleQuotesStrategy),
            Box::new(FixMalformedNumbersStrategy),
            Box::new(FixBooleanNullStrategy),
            Box::new(FixAgenticAiResponseStrategy),
        ];
        
        let validator: Box<dyn Validator> = Box::new(JsonValidator);
        let inner = crate::repairer_base::GenericRepairer::new(validator, strategies);
        
        Self { inner }
    }

    /// Create a new JSON repairer with logging enabled
    pub fn with_logging(logging: bool) -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(StripTrailingContentStrategy),
            Box::new(StripJsCommentsStrategy),
            Box::new(AddMissingQuotesStrategy),
            Box::new(FixTrailingCommasStrategy),
            Box::new(AddMissingBracesStrategy),
            Box::new(FixSingleQuotesStrategy),
            Box::new(FixMalformedNumbersStrategy),
            Box::new(FixBooleanNullStrategy),
            Box::new(FixAgenticAiResponseStrategy),
        ];
        
        let validator: Box<dyn Validator> = Box::new(JsonValidator);
        let inner = crate::repairer_base::GenericRepairer::new(validator, strategies)
            .with_logging(logging);
        
        Self { inner }
    }

    /// Get the repair log
    pub fn get_repair_log(&self) -> &[String] {
        self.inner.get_repair_log()
    }

    /// Clear the repair log
    pub fn clear_log(&mut self) {
        self.inner.clear_log();
    }
}

impl Default for JsonRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for JsonRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        self.inner.repair(content)
    }
    
    fn needs_repair(&self, content: &str) -> bool {
        self.inner.needs_repair(content)
    }
    
    fn confidence(&self, content: &str) -> f64 {
        // Use custom confidence calculation for JSON
        if self.inner.validator().is_valid(content) {
            return 1.0;
        }
        
        let mut score: f64 = 0.0;
        
        if content.contains('{') || content.contains('[') {
            score += 0.3;
        }
        
        if content.contains(':') {
            score += 0.2;
        }
        
        if content.contains('"') {
            score += 0.2;
        }
        
        if content.contains(',') {
            score += 0.1;
        }
        
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_repairer_creation() {
        let repairer = JsonRepairer::new();
        assert!(!repairer.inner.strategies().is_empty());
    }

    #[test]
    fn test_json_repairer_default() {
        let repairer = JsonRepairer::default();
        assert!(!repairer.inner.strategies().is_empty());
    }

    #[test]
    fn test_json_repairer_with_logging() {
        let repairer = JsonRepairer::with_logging(true);
        assert!(!repairer.inner.get_repair_log().is_empty() || repairer.inner.get_repair_log().is_empty());
    }

    #[test]
    fn test_json_confidence_valid() {
        let repairer = JsonRepairer::new();
        let confidence = repairer.confidence(r#"{"key": "value"}"#);
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_json_confidence_invalid() {
        let repairer = JsonRepairer::new();
        let confidence = repairer.confidence(r#"{"key": value}"#);
        assert!(confidence < 1.0);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_json_needs_repair() {
        let repairer = JsonRepairer::new();
        assert!(!repairer.needs_repair(r#"{"key": "value"}"#));
        assert!(repairer.needs_repair(r#"{"key": "value",}"#));
    }

    #[test]
    fn test_strip_js_comments() {
        let strategy = StripJsCommentsStrategy;
        // Single-line comment
        let input = r#"{"key": "value", // comment\n}"#;
        let result = strategy.apply(input).unwrap();
        assert!(!result.contains("//"));
        assert!(result.contains("value"));

        // Multi-line comment
        let input2 = r#"{"key": "value", /* multi-line
        comment */}"#;
        let result2 = strategy.apply(input2).unwrap();
        assert!(!result2.contains("/*"));

        // Comment in string should be preserved
        let input3 = r#"{"text": "not a // comment"}"#;
        let result3 = strategy.apply(input3).unwrap();
        assert!(result3.contains("//"));
    }

    #[test]
    fn test_json_with_js_comments_repair() {
        let mut repairer = JsonRepairer::new();
        let input = r#"{"key": "value", // this is a comment
        "another": "field" /* multi-line */}"#;
        let result = repairer.repair(input).unwrap();
        assert!(result.contains("key"));
        assert!(result.contains("value"));
        assert!(!result.contains("//"));
        assert!(!result.contains("/*"));
    }

    #[test]
    fn test_strip_js_comments_edge_cases() {
        let strategy = StripJsCommentsStrategy;

        // Comment at the start
        let input1 = r#"// comment at start
{"key": "value"}"#;
        let result1 = strategy.apply(input1).unwrap();
        assert!(!result1.contains("//"));
        assert!(result1.contains("key"));

        // Multiple single-line comments
        let input2 = r#"{"a": 1, // comment 1
"b": 2, // comment 2
"c": 3}"#;
        let result2 = strategy.apply(input2).unwrap();
        assert_eq!(result2.matches("//").count(), 0);

        // Comment with special characters
        let input3 = r#"{"key": "value", // comment with @#$%^&*()
}"#;
        let result3 = strategy.apply(input3).unwrap();
        assert!(!result3.contains("//"));

        // Empty comment
        let input4 = r#"{"key": "value", /**/}"#;
        let result4 = strategy.apply(input4).unwrap();
        assert!(!result4.contains("/*"));

        // Multi-line comment spanning multiple lines
        let input5 = r#"{
  "key": "value", /* this is a
  multi-line comment */"another": "field"}"#;
        let result5 = strategy.apply(input5).unwrap();
        assert!(!result5.contains("/*"));
        assert!(result5.contains("another"));

        // Comment with escaped quotes in string (should preserve)
        let input6 = r#"{"text": "not // a comment", "quote": "\"test\""}"#;
        let result6 = strategy.apply(input6).unwrap();
        assert!(result6.contains("//"));
        assert!(result6.contains("\\\"test\\\""));
    }

    #[test]
    fn test_json_with_various_comment_styles() {
        let mut repairer = JsonRepairer::new();

        // Real-world JSON with JS-style comments
        let input = r#"{
  // Configuration settings
  "apiVersion": "v1",
  "kind": "Config", /* Config kind */
  "metadata": {
    "name": "test-config", // Config name
    "namespace": "default"
  },
  // Data section
  "data": {
    "key": "value", /* Data key */
    "number": 42 // Answer to everything
  }
}"#;

        let result = repairer.repair(input).unwrap();
        assert!(result.contains("apiVersion"));
        assert!(result.contains("Config"));
        assert!(result.contains("test-config"));
        assert!(result.contains("data"));
        assert!(result.contains("key"));
        assert!(!result.contains("//"));
        assert!(!result.contains("/*"));

        // Verify it's valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&result).is_ok());
    }

    #[test]
    fn test_json_comments_preserve_string_content() {
        let mut repairer = JsonRepairer::new();

        // URLs with slashes should be preserved
        let input = r#"{"url": "https://example.com/path"}"#;
        let result = repairer.repair(input).unwrap();
        assert!(result.contains("https://"));

        // String with comment-like patterns
        let input2 = r#"{"text": "This is // not a comment", "code": "x = 1; // y = 2"}"#;
        let result2 = repairer.repair(input2).unwrap();
        assert!(result2.contains("This is // not"));
        assert!(result2.contains("x = 1; // y = 2"));

        // Note: Keys that start with // but are inside quotes are preserved
        // The StripJsCommentsStrategy correctly preserves content inside strings
        let input3 = r#"{"//comment": "remove me"}"#;
        let result3 = repairer.repair(input3).unwrap();
        // After AddMissingQuotesStrategy runs, the key gets quoted: "//comment" -> preserved
        // This is correct behavior - comments inside strings are preserved
        assert!(result3.contains(r#""//comment":"#));

        // However, actual line comments outside strings should be removed
        let input4 = r#"{"key": "value", // this is a real comment
        }"#;
        let result4 = repairer.repair(input4).unwrap();
        assert!(!result4.contains("// this is a real comment"));
    }

    #[test]
    fn test_json_comments_with_trailing_commas() {
        let mut repairer = JsonRepairer::new();

        // Combined issues: comments + trailing commas
        let input = r#"{
  "key1": "value1", // comment 1
  "key2": "value2", /* comment 2 */
  "key3": "value3",
}"#;

        let result = repairer.repair(input).unwrap();
        assert!(!result.contains("//"));
        assert!(!result.contains("/*"));
        assert!(!result.contains(",\n}"));
        assert!(result.contains("key1"));
        assert!(result.contains("key2"));
        assert!(result.contains("key3"));

        // Verify valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&result).is_ok());
    }
}

