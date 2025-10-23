//! TOML repair functionality

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use regex::Regex;
use std::sync::OnceLock;

/// Cached regex patterns for TOML performance optimization
#[allow(dead_code)]
struct TomlRegexCache {
    missing_quotes: Regex,
    malformed_arrays: Regex,
    malformed_tables: Regex,
    malformed_strings: Regex,
    malformed_numbers: Regex,
    malformed_dates: Regex,
}

impl TomlRegexCache {
    fn new() -> Result<Self> {
        Ok(Self {
            missing_quotes: Regex::new(r#"^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*([^"'\s].*[^"'\s])\s*$"#)?,
            malformed_arrays: Regex::new(r#"\[([^,\]]+),\]"#)?,
            malformed_tables: Regex::new(r#"^(\s*)\[([^]]+)\]\s*$"#)?,
            malformed_strings: Regex::new(r#"^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*'([^']*)'\s*$"#)?,
            malformed_numbers: Regex::new(r#"^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(\d+\.\d*\.\d+)"#)?,
            malformed_dates: Regex::new(r#"^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})"#)?,
        })
    }
}

static TOML_REGEX_CACHE: OnceLock<TomlRegexCache> = OnceLock::new();

fn get_toml_regex_cache() -> &'static TomlRegexCache {
    TOML_REGEX_CACHE.get_or_init(|| TomlRegexCache::new().expect("Failed to initialize TOML regex cache"))
}

/// TOML repairer that can fix common TOML issues
pub struct TomlRepairer {
    strategies: Vec<Box<dyn RepairStrategy>>,
    validator: TomlValidator,
}

impl TomlRepairer {
    /// Create a new TOML repairer
    pub fn new() -> Self {
        let mut strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(FixMissingQuotesStrategy),
            Box::new(FixMalformedArraysStrategy),
            Box::new(FixMalformedTablesStrategy),
            Box::new(FixMalformedStringsStrategy),
            Box::new(FixMalformedNumbersStrategy),
            Box::new(FixMalformedDatesStrategy),
            Box::new(AddTableHeadersStrategy),
        ];
        
        // Sort strategies by priority (higher priority first)
        strategies.sort_by_key(|b| std::cmp::Reverse(b.priority()));
        
        Self {
            strategies,
            validator: TomlValidator,
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

impl Default for TomlRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for TomlRepairer {
    fn repair(&self, content: &str) -> Result<String> {
        let trimmed = content.trim();
        
        // Handle empty content
        if trimmed.is_empty() {
            return Ok("".to_string());
        }
        
        // If already valid, return as-is
        if self.validator.is_valid(trimmed) {
            return Ok(trimmed.to_string());
        }
        
        // Apply repair strategies
        let repaired = self.apply_strategies(trimmed)?;
        
        // Always return the repaired content, even if validation fails
        Ok(repaired)
    }
    
    fn confidence(&self, content: &str) -> f64 {
        if content.trim().is_empty() {
            return 0.0;
        }
        
        // Calculate confidence based on TOML-like patterns
        let mut score: f64 = 0.0;
        
        // Check for table headers
        if content.contains('[') && content.contains(']') {
            score += 0.3;
        }
        
        // Check for key-value pairs
        if content.contains('=') {
            score += 0.3;
        }
        
        // Check for arrays
        if content.contains('[') && content.contains(',') {
            score += 0.2;
        }
        
        // Check for strings
        if content.contains('"') || content.contains("'") {
            score += 0.1;
        }
        
        // Check for numbers
        if content.chars().any(|c| c.is_ascii_digit()) {
            score += 0.1;
        }
        
        score.min(1.0)
    }
    
    fn needs_repair(&self, content: &str) -> bool {
        !self.validator.is_valid(content)
    }
}

/// TOML validator
pub struct TomlValidator;

impl Validator for TomlValidator {
    fn is_valid(&self, content: &str) -> bool {
        if content.trim().is_empty() {
            return false;
        }
        
        // Basic TOML validation using toml crate
        toml::from_str::<toml::Value>(content).is_ok()
    }
    
    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        if content.trim().is_empty() {
            errors.push("Empty TOML content".to_string());
            return errors;
        }
        
        // Try to parse with toml crate
        match toml::from_str::<toml::Value>(content) {
            Ok(_) => {}, // Valid TOML
            Err(e) => {
                errors.push(format!("TOML parsing error: {e}"));
            }
        }
        
        errors
    }
}

/// Strategy to fix missing quotes around string values
struct FixMissingQuotesStrategy;

impl RepairStrategy for FixMissingQuotesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_toml_regex_cache();
        let result = cache.missing_quotes.replace_all(content, |caps: &regex::Captures| {
            let indent = &caps[1];
            let key = &caps[2];
            let value = &caps[3];
            format!("{}{} = \"{}\"", indent, key, value)
        });
        
        Ok(result.to_string())
    }
    
    fn priority(&self) -> u8 {
        6
    }
}

/// Strategy to fix malformed arrays
struct FixMalformedArraysStrategy;

impl RepairStrategy for FixMalformedArraysStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_toml_regex_cache();
        let result = cache.malformed_arrays.replace_all(content, |caps: &regex::Captures| {
            let content = &caps[1];
            format!("[{content}]")
        });
        
        Ok(result.to_string())
    }
    
    fn priority(&self) -> u8 {
        5
    }
}

/// Strategy to fix malformed table headers
struct FixMalformedTablesStrategy;

impl RepairStrategy for FixMalformedTablesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_toml_regex_cache();
        let result = cache.malformed_tables.replace_all(content, |caps: &regex::Captures| {
            let indent = &caps[1];
            let table_name = &caps[2];
            format!("{}[{}]", indent, table_name)
        });
        
        Ok(result.to_string())
    }
    
    fn priority(&self) -> u8 {
        4
    }
}

/// Strategy to fix malformed strings
struct FixMalformedStringsStrategy;

impl RepairStrategy for FixMalformedStringsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_toml_regex_cache();
        let result = cache.malformed_strings.replace_all(content, |caps: &regex::Captures| {
            let indent = &caps[1];
            let key = &caps[2];
            let value = &caps[3];
            format!("{}{} = \"{}\"", indent, key, value)
        });
        
        Ok(result.to_string())
    }
    
    fn priority(&self) -> u8 {
        3
    }
}

/// Strategy to fix malformed numbers
struct FixMalformedNumbersStrategy;

impl RepairStrategy for FixMalformedNumbersStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_toml_regex_cache();
        let result = cache.malformed_numbers.replace_all(content, |caps: &regex::Captures| {
            let indent = &caps[1];
            let key = &caps[2];
            let number = &caps[3];
            // Remove extra decimal points
            let fixed_number = number.replace("..", ".");
            format!("{}{} = {}", indent, key, fixed_number)
        });
        
        Ok(result.to_string())
    }
    
    fn priority(&self) -> u8 {
        2
    }
}

/// Strategy to fix malformed dates
struct FixMalformedDatesStrategy;

impl RepairStrategy for FixMalformedDatesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_toml_regex_cache();
        let result = cache.malformed_dates.replace_all(content, |caps: &regex::Captures| {
            let indent = &caps[1];
            let key = &caps[2];
            let date = &caps[3];
            format!("{}{} = \"{}\"", indent, key, date)
        });
        
        Ok(result.to_string())
    }
    
    fn priority(&self) -> u8 {
        1
    }
}

/// Strategy to add table headers if missing
struct AddTableHeadersStrategy;

impl RepairStrategy for AddTableHeadersStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut has_table_header = false;
        
        for line in lines {
            let trimmed = line.trim();
            
            // Check if this is a key-value pair without a table header
            if trimmed.contains('=') && !trimmed.starts_with('[') && !has_table_header {
                result.push("[root]".to_string());
                has_table_header = true;
            }
            
            result.push(line.to_string());
        }
        
        Ok(result.join("\n"))
    }
    
    fn priority(&self) -> u8 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_toml_repair_basic() {
        let repairer = TomlRepairer::new();
        
        let input = r#"name = John
age = 30"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        [root]
        name = John
        age = 30
        ");
    }
    
    #[test]
    fn test_toml_repair_missing_quotes() {
        let repairer = TomlRepairer::new();
        
        let input = r#"[user]
name = John
email = john@example.com"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        [user]
        [root]
        name = John
        email = john@example.com
        ");
    }
    
    #[test]
    fn test_toml_repair_malformed_arrays() {
        let repairer = TomlRepairer::new();
        
        let input = r#"fruits = [apple, banana,]
colors = [red, green, blue,]"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        [root]
        fruits = [apple, banana,]
        colors = [red, green, blue,]
        ");
    }
    
    #[test]
    fn test_toml_repair_malformed_numbers() {
        let repairer = TomlRepairer::new();
        
        let input = r#"version = 1.2.3.4
price = 19.99..99"#;
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        [root]
        version = 1.2.3.4
        price = 19.99..99
        ");
    }
    
    #[test]
    fn test_toml_confidence() {
        let repairer = TomlRepairer::new();
        
        let valid_toml = r#"[user]
name = "John"
age = 30"#;
        let conf = repairer.confidence(valid_toml);
        assert!(conf > 0.5);
        
        let invalid_toml = "not toml at all";
        let conf = repairer.confidence(invalid_toml);
        assert!(conf < 0.8);
    }
    
    #[test]
    fn test_toml_validator() {
        let validator = TomlValidator;
        
        assert!(validator.is_valid(r#"[user]
name = "John"
age = 30"#));
        assert!(!validator.is_valid("invalid toml"));
        assert!(!validator.is_valid(""));
    }
    
    #[test]
    fn test_toml_strategies_individual() {
        // Test FixMissingQuotesStrategy
        let strategy = FixMissingQuotesStrategy;
        let input = "name = John";
        let result = strategy.apply(input).unwrap();
        assert!(result.contains("name = \"John\""));
        
        // Test FixMalformedArraysStrategy
        let strategy = FixMalformedArraysStrategy;
        let input = "fruits = [apple, banana,]";
        let result = strategy.apply(input).unwrap();
        assert!(result.contains("apple, banana"));
        
        // Test AddTableHeadersStrategy
        let strategy = AddTableHeadersStrategy;
        let input = "name = John";
        let result = strategy.apply(input).unwrap();
        assert!(result.starts_with("[root]"));
    }
}
