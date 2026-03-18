//! YAML repair functionality

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use regex::Regex;
use serde_yaml::Value;
use std::sync::OnceLock;

/// Cached regex patterns for YAML performance optimization
#[allow(dead_code)]
struct YamlRegexCache {
    missing_colons: Regex,
    list_items: Regex,
    quoted_strings: Regex,
}

impl YamlRegexCache {
    fn new() -> Result<Self> {
        Ok(Self {
            missing_colons: Regex::new(r#"^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s+([^:].*)$"#)?,
            list_items: Regex::new(r#"^\s*-\s*(.+)$"#)?,
            quoted_strings: Regex::new(
                r#"^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*([^'"].*[^'"])\s*$"#,
            )?,
        })
    }
}

static YAML_REGEX_CACHE: OnceLock<YamlRegexCache> = OnceLock::new();

fn get_yaml_regex_cache() -> &'static YamlRegexCache {
    YAML_REGEX_CACHE
        .get_or_init(|| YamlRegexCache::new().expect("Failed to initialize YAML regex cache"))
}

/// YAML repairer that can fix common YAML issues
///
/// Uses trait-based composition with GenericRepairer for better modularity
pub struct YamlRepairer {
    inner: crate::repairer_base::GenericRepairer,
}

impl YamlRepairer {
    /// Create a new YAML repairer
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(FixIndentationStrategy),
            Box::new(AddMissingColonsStrategy),
            Box::new(FixListFormattingStrategy),
            Box::new(AddDocumentSeparatorStrategy),
            Box::new(FixQuotedStringsStrategy),
            Box::new(AdvancedIndentationStrategy),
            Box::new(ComplexStructureStrategy),
        ];

        let validator: Box<dyn Validator> = Box::new(YamlValidator);
        let inner = crate::repairer_base::GenericRepairer::new(validator, strategies);

        Self { inner }
    }
}

impl Default for YamlRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for YamlRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        self.inner.repair(content)
    }

    fn needs_repair(&self, content: &str) -> bool {
        self.inner.needs_repair(content)
    }

    fn confidence(&self, content: &str) -> f64 {
        if self.inner.validator().is_valid(content) {
            return 1.0;
        }

        // Calculate confidence based on YAML-like patterns
        let mut score: f64 = 0.0;

        // Check for key-value pairs with colons
        if content.contains(':') {
            score += 0.3;
        }

        // Check for proper indentation patterns
        let lines: Vec<&str> = content.lines().collect();
        let mut has_consistent_indentation = true;
        let mut last_indent = 0;
        let mut has_content = false;

        for line in &lines {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            has_content = true;

            let indent = line.chars().take_while(|c| c.is_whitespace()).count();
            if last_indent > 0 && indent != last_indent && indent != last_indent + 2 {
                has_consistent_indentation = false;
                break;
            }
            last_indent = indent;
        }

        if has_consistent_indentation && has_content {
            score += 0.3;
        }

        // Check for list indicators
        if content.contains('-') {
            score += 0.2;
        }

        // Check for document separator
        if content.contains("---") {
            score += 0.1;
        }

        // Check for quoted strings
        if content.contains('"') || content.contains("'") {
            score += 0.1;
        }

        score.min(1.0_f64)
    }
}

/// YAML validator
pub struct YamlValidator;

impl Validator for YamlValidator {
    fn is_valid(&self, content: &str) -> bool {
        if content.trim().is_empty() {
            return false;
        }

        // Check for basic YAML syntax issues
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check for missing colons in key-value pairs
            if !trimmed.starts_with('-')
                && !trimmed.starts_with('[')
                && !trimmed.starts_with('{')
                && !trimmed.contains(':')
                && trimmed.contains(' ')
            {
                return false;
            }
        }

        serde_yaml::from_str::<Value>(content).is_ok()
    }

    fn validate(&self, content: &str) -> Vec<String> {
        match serde_yaml::from_str::<Value>(content) {
            Ok(_) => vec![],
            Err(e) => vec![e.to_string()],
        }
    }
}

/// Strategy to fix indentation issues
struct FixIndentationStrategy;

impl RepairStrategy for FixIndentationStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::<String>::new();
        let mut indent_stack = vec![0];

        for line in lines {
            if line.trim().is_empty() {
                result.push(line.to_string());
                continue;
            }

            let _current_indent = line.chars().take_while(|c| c.is_whitespace()).count();
            let trimmed = line.trim();

            // Determine expected indentation based on context
            let expected_indent = if trimmed.starts_with('-') {
                indent_stack.last().copied().unwrap_or(0)
            } else if trimmed.ends_with(':') {
                indent_stack.last().copied().unwrap_or(0)
            } else {
                indent_stack.last().copied().unwrap_or(0) + 2
            };

            // Fix missing colons for key-value pairs
            let fixed_trimmed = if !trimmed.contains(':') && trimmed.contains(' ') {
                // This looks like a key-value pair missing a colon
                trimmed.replacen(' ', ": ", 1)
            } else {
                trimmed.to_string()
            };

            // Fix indentation
            let fixed_line = format!("{}{}", " ".repeat(expected_indent), fixed_trimmed);
            result.push(fixed_line);

            // Update indent stack
            if fixed_trimmed.ends_with(':') || fixed_trimmed.starts_with('-') {
                indent_stack.push(expected_indent + 2);
            }
        }

        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        5
    }

    fn name(&self) -> &str {
        "FixIndentationStrategy"
    }
}

/// Strategy to add missing colons
struct AddMissingColonsStrategy;

impl RepairStrategy for AddMissingColonsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_yaml_regex_cache();
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();

        for line in lines {
            if cache.missing_colons.is_match(line) {
                let fixed = cache.missing_colons.replace(line, "$1$2: $3");
                result.push(fixed.to_string());
            } else {
                result.push(line.to_string());
            }
        }

        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        4
    }

    fn name(&self) -> &str {
        "AddMissingColonsStrategy"
    }
}

/// Strategy to fix list formatting
struct FixListFormattingStrategy;

impl RepairStrategy for FixListFormattingStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_yaml_regex_cache();
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();

        for line in lines {
            if cache.list_items.is_match(line) {
                let fixed = cache.list_items.replace(line, "- $1");
                result.push(fixed.to_string());
            } else {
                result.push(line.to_string());
            }
        }

        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        3
    }

    fn name(&self) -> &str {
        "FixListFormattingStrategy"
    }
}

/// Strategy to add document separator
struct AddDocumentSeparatorStrategy;

impl RepairStrategy for AddDocumentSeparatorStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let trimmed = content.trim();
        if !trimmed.starts_with("---") {
            Ok(format!("---\n{}", trimmed))
        } else {
            Ok(trimmed.to_string())
        }
    }

    fn priority(&self) -> u8 {
        2
    }

    fn name(&self) -> &str {
        "AddDocumentSeparatorStrategy"
    }
}

/// Strategy to fix quoted strings
struct FixQuotedStringsStrategy;

impl RepairStrategy for FixQuotedStringsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        // Convert single quotes to double quotes
        let single_quote_re = Regex::new(r"'([^']*)'")?;
        let result = single_quote_re.replace_all(content, r#""$1""#);
        Ok(result.to_string())
    }

    fn priority(&self) -> u8 {
        1
    }

    fn name(&self) -> &str {
        "FixQuotedStringsStrategy"
    }
}

/// Strategy for advanced indentation detection and fixing
struct AdvancedIndentationStrategy;

impl RepairStrategy for AdvancedIndentationStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let _indent_stack: Vec<usize> = Vec::new();
        let mut current_indent = 0;

        for line in lines {
            if line.trim().is_empty() || line.starts_with('#') {
                result.push(line.to_string());
                continue;
            }

            let line_indent = line.chars().take_while(|c| c.is_whitespace()).count();
            let trimmed = line.trim();

            // Detect list items
            if trimmed.starts_with('-') {
                // List items should be indented 2 spaces more than their parent
                let expected_indent = current_indent + 2;
                if line_indent != expected_indent {
                    let fixed = format!("{}- {}", " ".repeat(expected_indent), trimmed[1..].trim());
                    result.push(fixed);
                    current_indent = expected_indent;
                } else {
                    result.push(line.to_string());
                    current_indent = line_indent;
                }
            } else if trimmed.contains(':') {
                // Key-value pairs
                let expected_indent = current_indent;
                if line_indent != expected_indent {
                    let fixed = format!("{}{}", " ".repeat(expected_indent), trimmed);
                    result.push(fixed);
                    current_indent = expected_indent;
                } else {
                    result.push(line.to_string());
                    current_indent = line_indent;
                }
            } else {
                // Other content - maintain relative indentation
                result.push(line.to_string());
                current_indent = line_indent;
            }
        }

        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        6
    }

    fn name(&self) -> &str {
        "AdvancedIndentationStrategy"
    }
}

/// Strategy for handling complex nested structures
struct ComplexStructureStrategy;

impl RepairStrategy for ComplexStructureStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut in_multiline_string = false;
        let mut multiline_indent = 0;

        for (_i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() || line.starts_with('#') {
                result.push(line.to_string());
                continue;
            }

            let trimmed = line.trim();

            // Handle multiline strings
            if trimmed.starts_with('|') || trimmed.starts_with('>') {
                in_multiline_string = true;
                multiline_indent = line.chars().take_while(|c| c.is_whitespace()).count();
                result.push(line.to_string());
                continue;
            }

            if in_multiline_string {
                let line_indent = line.chars().take_while(|c| c.is_whitespace()).count();
                if line_indent > multiline_indent || line.trim().is_empty() {
                    result.push(line.to_string());
                    continue;
                } else {
                    in_multiline_string = false;
                }
            }

            // Fix nested object/array structures
            if trimmed.starts_with('-') && trimmed.contains(':') {
                // List item with key-value pair
                let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    let fixed = format!("- {}: {}", key, value);
                    result.push(fixed);
                } else {
                    result.push(line.to_string());
                }
            } else if trimmed.contains(':') && !trimmed.ends_with(':') {
                // Key-value pair
                let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    if value.is_empty() {
                        // Key with no value - might be a parent object
                        result.push(line.to_string());
                    } else {
                        let fixed = format!("{}: {}", key, value);
                        result.push(fixed);
                    }
                } else {
                    result.push(line.to_string());
                }
            } else {
                result.push(line.to_string());
            }
        }

        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        5
    }

    fn name(&self) -> &str {
        "ComplexStructureStrategy"
    }
}
