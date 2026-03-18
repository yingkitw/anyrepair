//! TOML repair module

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
            missing_quotes: Regex::new(
                r#"^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*([^"'\s].*[^"'\s])\s*$"#,
            )?,
            malformed_arrays: Regex::new(r#"\[([^,\]]+),\]"#)?,
            malformed_tables: Regex::new(r#"^(\s*)\[([^]]+)\]\s*$"#)?,
            malformed_strings: Regex::new(r#"^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*'([^']*)'\s*$"#)?,
            malformed_numbers: Regex::new(
                r#"^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(\d+\.\d*\.\d+)"#,
            )?,
            malformed_dates: Regex::new(
                r#"^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})"#,
            )?,
        })
    }
}

static TOML_REGEX_CACHE: OnceLock<TomlRegexCache> = OnceLock::new();

fn get_toml_regex_cache() -> &'static TomlRegexCache {
    TOML_REGEX_CACHE
        .get_or_init(|| TomlRegexCache::new().expect("Failed to initialize TOML regex cache"))
}

/// TOML repairer that can fix common TOML issues
///
/// Uses trait-based composition with GenericRepairer for better modularity
pub struct TomlRepairer {
    inner: crate::repairer_base::GenericRepairer,
}

impl TomlRepairer {
    /// Create a new TOML repairer
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(FixMissingQuotesStrategy),
            Box::new(FixMalformedArraysStrategy),
            Box::new(FixMalformedTablesStrategy),
            Box::new(FixMalformedStringsStrategy),
            Box::new(FixMalformedNumbersStrategy),
            Box::new(FixMalformedDatesStrategy),
            Box::new(AddTableHeadersStrategy),
        ];

        let validator: Box<dyn Validator> = Box::new(TomlValidator);
        let inner = crate::repairer_base::GenericRepairer::new(validator, strategies);

        Self { inner }
    }
}

impl Default for TomlRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for TomlRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        self.inner.repair(content)
    }

    fn needs_repair(&self, content: &str) -> bool {
        self.inner.needs_repair(content)
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
}

/// TOML validator
pub struct TomlValidator;

impl Validator for TomlValidator {
    fn is_valid(&self, content: &str) -> bool {
        if content.trim().is_empty() {
            return false;
        }

        // Check for missing quotes around string values
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
                continue;
            }

            // Check for key-value pairs where the value should be quoted
            if trimmed.contains('=') {
                let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let value = parts[1].trim();
                    // If value looks like a string but is not quoted, it's invalid
                    if !value.starts_with('"')
                        && !value.starts_with('\'')
                        && !value.starts_with('[')
                        && !value.starts_with('{')
                        && !value.parse::<i64>().is_ok()
                        && !value.parse::<f64>().is_ok()
                        && value != "true"
                        && value != "false"
                    {
                        return false;
                    }
                }
            }
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
            Ok(_) => {} // Valid TOML
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
        let result = cache
            .missing_quotes
            .replace_all(content, |caps: &regex::Captures| {
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

    fn name(&self) -> &str {
        "FixMissingQuotesStrategy"
    }
}

/// Strategy to fix malformed arrays
struct FixMalformedArraysStrategy;

impl RepairStrategy for FixMalformedArraysStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_toml_regex_cache();
        let result = cache
            .malformed_arrays
            .replace_all(content, |caps: &regex::Captures| {
                let content = &caps[1];
                format!("[{content}]")
            });

        Ok(result.to_string())
    }

    fn priority(&self) -> u8 {
        5
    }

    fn name(&self) -> &str {
        "FixMalformedArraysStrategy"
    }
}

/// Strategy to fix malformed table headers
struct FixMalformedTablesStrategy;

impl RepairStrategy for FixMalformedTablesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_toml_regex_cache();
        let result = cache
            .malformed_tables
            .replace_all(content, |caps: &regex::Captures| {
                let indent = &caps[1];
                let table_name = &caps[2];
                format!("{}[{}]", indent, table_name)
            });

        Ok(result.to_string())
    }

    fn priority(&self) -> u8 {
        4
    }

    fn name(&self) -> &str {
        "FixMalformedTablesStrategy"
    }
}

/// Strategy to fix malformed strings
struct FixMalformedStringsStrategy;

impl RepairStrategy for FixMalformedStringsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_toml_regex_cache();
        let result = cache
            .malformed_strings
            .replace_all(content, |caps: &regex::Captures| {
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

    fn name(&self) -> &str {
        "FixMalformedStringsStrategy"
    }
}

/// Strategy to fix malformed numbers
struct FixMalformedNumbersStrategy;

impl RepairStrategy for FixMalformedNumbersStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_toml_regex_cache();
        let result = cache
            .malformed_numbers
            .replace_all(content, |caps: &regex::Captures| {
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

    fn name(&self) -> &str {
        "FixMalformedNumbersStrategy"
    }
}

/// Strategy to fix malformed dates
struct FixMalformedDatesStrategy;

impl RepairStrategy for FixMalformedDatesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_toml_regex_cache();
        let result = cache
            .malformed_dates
            .replace_all(content, |caps: &regex::Captures| {
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

    fn name(&self) -> &str {
        "FixMalformedDatesStrategy"
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

    fn name(&self) -> &str {
        "AddTableHeadersStrategy"
    }
}
