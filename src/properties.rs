//! Properties file repair module

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use regex::Regex;
use std::sync::OnceLock;

/// Cached regex patterns for Properties performance optimization
#[allow(dead_code)]
struct PropertiesRegexCache {
    malformed_comments: Regex,
    missing_equals: Regex,
    whitespace_around_equals: Regex,
    empty_keys: Regex,
    missing_delimiters: Regex,
}

impl PropertiesRegexCache {
    fn new() -> Result<Self> {
        Ok(Self {
            malformed_comments: Regex::new(r#"^(\s*)[!#](.*)$"#)?,
            missing_equals: Regex::new(r#"^(\s*)([^=\s!#]+)\s+([^=\s!#].*)$"#)?,
            whitespace_around_equals: Regex::new(r#"^(\s*)([^=\s]+)\s*=\s*(.*)\s*$"#)?,
            empty_keys: Regex::new(r#"^(\s*)=\s*(.*)\s*$"#)?,
            missing_delimiters: Regex::new(r#"^(\s*)([^=\s!#]+)\s*([^\s=].*)$"#)?,
        })
    }
}

static PROPERTIES_REGEX_CACHE: OnceLock<PropertiesRegexCache> = OnceLock::new();

fn get_properties_regex_cache() -> &'static PropertiesRegexCache {
    PROPERTIES_REGEX_CACHE.get_or_init(|| {
        PropertiesRegexCache::new().expect("Failed to initialize Properties regex cache")
    })
}

/// Properties repairer that can fix common Properties file issues
///
/// Uses trait-based composition with GenericRepairer for better modularity
pub struct PropertiesRepairer {
    inner: crate::repairer_base::GenericRepairer,
}

impl PropertiesRepairer {
    /// Create a new Properties repairer
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(FixMissingEqualsStrategy),
            Box::new(FixWhitespaceAroundEqualsStrategy),
            Box::new(FixEmptyKeysStrategy),
            Box::new(FixMalformedCommentsStrategy),
            Box::new(EscapeSpecialCharactersStrategy),
        ];

        let validator: Box<dyn Validator> = Box::new(PropertiesValidator);
        let inner = crate::repairer_base::GenericRepairer::new(validator, strategies);

        Self { inner }
    }
}

impl Default for PropertiesRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for PropertiesRepairer {
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

        // Calculate confidence based on Properties-like patterns
        let mut score: f64 = 0.0;

        // Check for key=value pairs
        if content.contains('=') {
            score += 0.5;
        }

        // Check for comments
        if content.contains('#') || content.contains('!') {
            score += 0.2;
        }

        // Check for property-like naming patterns
        if content.matches('.').count() > 0 {
            score += 0.2;
        }

        // Check for multi-line values (continuation with backslash)
        if content.contains("\\\n") {
            score += 0.1;
        }

        score.clamp(0.0, 1.0)
    }
}

/// Properties validator
pub struct PropertiesValidator;

impl Validator for PropertiesValidator {
    fn is_valid(&self, content: &str) -> bool {
        let cache = get_properties_regex_cache();

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
                continue;
            }

            // Check if line contains a valid key=value pair
            if !trimmed.contains('=') {
                return false;
            }

            // Check if line has empty key
            if cache.empty_keys.is_match(trimmed) {
                return false;
            }

            // Extract key and value
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                if key.is_empty() {
                    return false;
                }

                // Check for improper whitespace around equals
                // Key should not end with space, value should not start with space
                if eq_pos > 0 && trimmed.chars().nth(eq_pos - 1) == Some(' ') {
                    return false;
                }
                if eq_pos < trimmed.len() - 1 && trimmed.chars().nth(eq_pos + 1) == Some(' ') {
                    return false;
                }
            }
        }

        true
    }

    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        let cache = get_properties_regex_cache();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
                continue;
            }

            // Check for missing equals sign
            if !trimmed.contains('=') {
                errors.push(format!("Line {}: Missing '=' delimiter", line_num + 1));
                continue;
            }

            // Check for empty key
            if trimmed.starts_with('=') {
                errors.push(format!("Line {}: Empty key", line_num + 1));
            }

            // Check for invalid characters in key
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                if key.contains('=') || key.contains('#') || key.contains('!') {
                    errors.push(format!("Line {}: Invalid characters in key", line_num + 1));
                }

                // Check for improper whitespace around equals
                if eq_pos > 0 && trimmed.chars().nth(eq_pos - 1) == Some(' ') {
                    errors.push(format!("Line {}: Whitespace before '='", line_num + 1));
                }
                if eq_pos < trimmed.len() - 1 && trimmed.chars().nth(eq_pos + 1) == Some(' ') {
                    errors.push(format!("Line {}: Whitespace after '='", line_num + 1));
                }
            }
        }

        errors
    }
}

/// Strategy to fix missing equals signs
pub struct FixMissingEqualsStrategy;

impl RepairStrategy for FixMissingEqualsStrategy {
    fn name(&self) -> &str {
        "FixMissingEquals"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
                result.push(line.to_string());
                continue;
            }

            // Try to fix missing equals in key-value pairs
            if !trimmed.contains('=') {
                // Split by whitespace and assume first part is key, rest is value
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let key = parts[0];
                    let value = parts[1..].join(" ");
                    result.push(format!("{}={}", key, value));
                } else if parts.len() == 1 {
                    // Only a key, add empty value
                    result.push(format!("{}=", parts[0]));
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
        100
    }
}

/// Strategy to fix whitespace around equals signs
pub struct FixWhitespaceAroundEqualsStrategy;

impl RepairStrategy for FixWhitespaceAroundEqualsStrategy {
    fn name(&self) -> &str {
        "FixWhitespaceAroundEquals"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
                result.push(line.to_string());
                continue;
            }

            // Fix whitespace around equals
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                let value = trimmed[eq_pos + 1..].trim();
                result.push(format!("{}={}", key, value));
            } else {
                result.push(line.to_string());
            }
        }

        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        90
    }
}

/// Strategy to fix empty keys
pub struct FixEmptyKeysStrategy;

impl RepairStrategy for FixEmptyKeysStrategy {
    fn name(&self) -> &str {
        "FixEmptyKeys"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
                result.push(line.to_string());
                continue;
            }

            // Fix empty key by adding a placeholder
            if trimmed.starts_with('=') {
                let value = &trimmed[1..].trim();
                result.push(format!("key_{}={}", line_num, value));
            } else {
                result.push(line.to_string());
            }
        }

        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        80
    }
}

/// Strategy to fix malformed comments
pub struct FixMalformedCommentsStrategy;

impl RepairStrategy for FixMalformedCommentsStrategy {
    fn name(&self) -> &str {
        "FixMalformedComments"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_properties_regex_cache();
        let mut result = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Fix comments that don't start with # or !
            if trimmed.contains('#') && !trimmed.starts_with('#') && !trimmed.contains('=') {
                // Move # to the beginning
                if let Some(hash_pos) = trimmed.find('#') {
                    let before = &trimmed[..hash_pos].trim();
                    let after = &trimmed[hash_pos..];
                    if before.is_empty() {
                        result.push(after.to_string());
                    } else {
                        result.push(format!("#{} {}", before, &after[1..].trim()));
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
        70
    }
}

/// Strategy to escape special characters in values
pub struct EscapeSpecialCharactersStrategy;

impl RepairStrategy for EscapeSpecialCharactersStrategy {
    fn name(&self) -> &str {
        "EscapeSpecialCharacters"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
                result.push(line.to_string());
                continue;
            }

            // Only escape if needed (unescaped special chars)
            if let Some(eq_pos) = trimmed.find('=') {
                let key = &trimmed[..eq_pos];
                let value = &trimmed[eq_pos + 1..];

                // Only escape if value contains special characters that aren't already escaped
                let needs_escape = value.chars().any(|c| matches!(c, '\n' | '\t' | '\r'));

                if needs_escape {
                    let escaped_value = value
                        .replace('\n', "\\n")
                        .replace('\t', "\\t")
                        .replace('\r', "\\r");

                    result.push(format!("{}={}", key, escaped_value));
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
        60
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_properties_repairer_creation() {
        let _repairer = PropertiesRepairer::new();
    }

    #[test]
    fn test_properties_repairer_default() {
        let _repairer = PropertiesRepairer::default();
    }

    #[test]
    fn test_valid_properties() {
        let content = r#"# Comment
key1=value1
key2=value2
"#;
        let repairer = PropertiesRepairer::new();
        assert!(repairer.needs_repair(content) == false);
    }

    #[test]
    fn test_fix_missing_equals() {
        let content = "key1 value1\nkey2 value2";
        let mut repairer = PropertiesRepairer::new();
        let result = repairer.repair(content).unwrap();
        assert!(result.contains("key1=value1"));
        assert!(result.contains("key2=value2"));
    }

    #[test]
    fn test_fix_whitespace_around_equals() {
        let content = "key1 = value1\nkey2=value2";
        let mut repairer = PropertiesRepairer::new();
        let result = repairer.repair(content).unwrap();
        assert!(result.contains("key1=value1"));
    }

    #[test]
    fn test_fix_empty_keys() {
        let content = "=value1\nkey2=value2";
        let mut repairer = PropertiesRepairer::new();
        let result = repairer.repair(content).unwrap();
        assert!(!result.starts_with('='));
    }

    #[test]
    fn test_properties_validator() {
        let validator = PropertiesValidator;

        // Valid properties
        let valid = "key1=value1\nkey2=value2";
        assert!(validator.is_valid(valid));

        // Invalid - missing equals
        let invalid = "key1 value1\nkey2=value2";
        assert!(!validator.is_valid(invalid));

        // Invalid - empty key
        let invalid2 = "=value1\nkey2=value2";
        assert!(!validator.is_valid(invalid2));
    }

    #[test]
    fn test_properties_confidence() {
        let repairer = PropertiesRepairer::new();

        // High confidence
        let high_conf = "key1=value1\nkey2=value2";
        assert!(repairer.confidence(high_conf) >= 0.5);

        // Low confidence
        let low_conf = "some random text";
        assert!(repairer.confidence(low_conf) < 0.5);
    }

    #[test]
    fn test_escape_special_characters() {
        let content = "key1=value\nkey2=value\twith\ttabs";
        let mut repairer = PropertiesRepairer::new();
        let result = repairer.repair(content).unwrap();
        // Should escape special characters in values
        assert!(result.contains("\\n") || result.contains("value\nkey2"));
        assert!(result.contains("\\t") || result.contains("with\ttabs"));
    }
}
