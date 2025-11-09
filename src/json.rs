//! JSON repair module
//! 
//! Provides comprehensive JSON repair functionality with multiple strategies
//! for fixing common JSON issues from LLM outputs.

use crate::error::Result;
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

// ============================================================================
// JSON Repairer
// ============================================================================

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
            Box::new(StripTrailingContentStrategy),
            Box::new(AddMissingQuotesStrategy),
            Box::new(FixTrailingCommasStrategy),
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
    
    /// Apply all repair strategies to the content
    fn apply_strategies(&mut self, content: &str) -> Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_repairer_creation() {
        let repairer = JsonRepairer::new();
        assert!(!repairer.strategies.is_empty());
    }

    #[test]
    fn test_json_repairer_default() {
        let repairer = JsonRepairer::default();
        assert!(!repairer.strategies.is_empty());
    }

    #[test]
    fn test_json_repairer_with_logging() {
        let repairer = JsonRepairer::with_logging(true);
        assert!(repairer.logging);
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
}

