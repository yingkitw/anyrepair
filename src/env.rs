//! Environment variable file (.env) repair module

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use regex::Regex;
use std::sync::OnceLock;

/// Cached regex patterns for .env performance optimization
#[allow(dead_code)]
struct EnvRegexCache {
    double_quoted_values: Regex,
    single_quoted_values: Regex,
    variable_expansion: Regex,
    malformed_keys: Regex,
}

impl EnvRegexCache {
    fn new() -> Result<Self> {
        Ok(Self {
            double_quoted_values: Regex::new(r#"^(.+)="(.*)"$"#)?,
            single_quoted_values: Regex::new(r#"^(.+)=('(.*)')$"#)?,
            variable_expansion: Regex::new(r"\$\{?[\w]+\}?")?,
            malformed_keys: Regex::new(r#"^[^=\s]+"#)?,
        })
    }
}

static ENV_REGEX_CACHE: OnceLock<EnvRegexCache> = OnceLock::new();

fn get_env_regex_cache() -> &'static EnvRegexCache {
    ENV_REGEX_CACHE
        .get_or_init(|| EnvRegexCache::new().expect("Failed to initialize .env regex cache"))
}

/// Environment variable (.env) repairer that can fix common .env file issues
///
/// Uses trait-based composition with GenericRepairer for better modularity
pub struct EnvRepairer {
    inner: crate::repairer_base::GenericRepairer,
}

impl EnvRepairer {
    /// Create a new .env repairer
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(FixMissingEqualsStrategy),
            Box::new(FixWhitespaceAroundEqualsStrategy),
            Box::new(FixEmptyKeysStrategy),
            Box::new(FixMalformedCommentsStrategy),
            Box::new(FixQuotedValuesStrategy),
        ];

        let validator: Box<dyn Validator> = Box::new(EnvValidator);
        let inner = crate::repairer_base::GenericRepairer::new(validator, strategies);

        Self { inner }
    }
}

impl Default for EnvRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for EnvRepairer {
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

        // Calculate confidence based on .env-like patterns
        let mut score: f64 = 0.0;

        // Check for key=value pairs
        if content.contains('=') {
            score += 0.4;
        }

        // Check for comments
        if content.contains('#') {
            score += 0.2;
        }

        // Check for uppercase naming convention (common in .env files)
        let uppercase_count = content.matches(char::is_uppercase).count();
        let total_chars = content.len();
        if total_chars > 0 && uppercase_count as f64 / total_chars as f64 > 0.2 {
            score += 0.2;
        }

        // Check for underscores (common in variable names)
        if content.contains('_') {
            score += 0.1;
        }

        // Check for quoted values
        if content.contains('"') || content.contains('\'') {
            score += 0.1;
        }

        score.clamp(0.0, 1.0)
    }
}

/// Environment variable validator
pub struct EnvValidator;

impl Validator for EnvValidator {
    fn is_valid(&self, content: &str) -> bool {
        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check if line contains a valid key=value pair
            if !trimmed.contains('=') {
                return false;
            }

            // Extract key and value
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                if key.is_empty() {
                    return false;
                }

                // Check for improper whitespace around equals
                if eq_pos > 0 && trimmed.chars().nth(eq_pos - 1) == Some(' ') {
                    return false;
                }
                if eq_pos < trimmed.len() - 1 && trimmed.chars().nth(eq_pos + 1) == Some(' ') {
                    // Allow space after equals if value is quoted
                    let value = trimmed[eq_pos + 1..].trim();
                    if !value.starts_with('"') && !value.starts_with('\'') {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
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
                if key.contains('=') || key.contains('#') {
                    errors.push(format!("Line {}: Invalid characters in key", line_num + 1));
                }

                // Check for improper whitespace around equals
                if eq_pos > 0 && trimmed.chars().nth(eq_pos - 1) == Some(' ') {
                    errors.push(format!("Line {}: Whitespace before '='", line_num + 1));
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
            if trimmed.is_empty() || trimmed.starts_with('#') {
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
            if trimmed.is_empty() || trimmed.starts_with('#') {
                result.push(line.to_string());
                continue;
            }

            // Fix whitespace around equals
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                let value = trimmed[eq_pos + 1..].trim();

                // Check if value should be quoted
                if value.contains(' ') && !value.starts_with('"') && !value.starts_with('\'') {
                    result.push(format!("{}=\"{}\"", key, value));
                } else {
                    result.push(format!("{}={}", key, value));
                }
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
            if trimmed.is_empty() || trimmed.starts_with('#') {
                result.push(line.to_string());
                continue;
            }

            // Fix empty key by adding a placeholder
            if trimmed.starts_with('=') {
                let value = &trimmed[1..].trim();
                result.push(format!("ENV_VAR_{}={}", line_num, value));
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
        let mut result = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Fix comments that don't start with #
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

/// Strategy to fix quoted values
pub struct FixQuotedValuesStrategy;

impl RepairStrategy for FixQuotedValuesStrategy {
    fn name(&self) -> &str {
        "FixQuotedValues"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_env_regex_cache();
        let mut result = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                result.push(line.to_string());
                continue;
            }

            // Fix quoted values with mismatched quotes
            if let Some(eq_pos) = trimmed.find('=') {
                let value = &trimmed[eq_pos + 1..].trim();

                // Check if value has quotes but they're mismatched
                if value.starts_with('"') && !value.ends_with('"') {
                    result.push(format!("{}=\"{}\"", &trimmed[..=eq_pos], &value[1..]));
                } else if value.starts_with('\'') && !value.ends_with('\'') {
                    result.push(format!("{}='{}'", &trimmed[..=eq_pos], &value[1..]));
                } else if value.ends_with('"') && !value.starts_with('"') {
                    result.push(format!(
                        "{}\"{}\"",
                        &trimmed[..=eq_pos],
                        &value[..value.len() - 1]
                    ));
                } else if value.ends_with('\'') && !value.starts_with('\'') {
                    result.push(format!(
                        "{}'{}'",
                        &trimmed[..=eq_pos],
                        &value[..value.len() - 1]
                    ));
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
    fn test_env_repairer_creation() {
        let _repairer = EnvRepairer::new();
    }

    #[test]
    fn test_env_repairer_default() {
        let _repairer = EnvRepairer::default();
    }

    #[test]
    fn test_valid_env() {
        let content = r#"# Comment
DATABASE_URL=postgresql://localhost/mydb
API_KEY=secret_key
"#;
        let repairer = EnvRepairer::new();
        assert!(repairer.needs_repair(content) == false);
    }

    #[test]
    fn test_fix_missing_equals() {
        let content = "DATABASE_URL postgresql://localhost/mydb\nAPI_KEY secret_key";
        let mut repairer = EnvRepairer::new();
        let result = repairer.repair(content).unwrap();
        assert!(result.contains("DATABASE_URL="));
        assert!(result.contains("API_KEY="));
    }

    #[test]
    fn test_fix_whitespace_around_equals() {
        let content = "DATABASE_URL = postgresql://localhost/mydb\nAPI_KEY=secret_key";
        let mut repairer = EnvRepairer::new();
        let result = repairer.repair(content).unwrap();
        assert!(result.contains("DATABASE_URL="));
        assert!(result.contains("postgresql://localhost/mydb"));
        // Should remove whitespace around equals
        assert!(!result.contains("DATABASE_URL ="));
    }

    #[test]
    fn test_fix_empty_keys() {
        let content = "=postgresql://localhost/mydb\nAPI_KEY=secret_key";
        let mut repairer = EnvRepairer::new();
        let result = repairer.repair(content).unwrap();
        assert!(!result.starts_with('='));
    }

    #[test]
    fn test_env_validator() {
        let validator = EnvValidator;

        // Valid .env
        let valid = "DATABASE_URL=postgresql://localhost/mydb\nAPI_KEY=secret_key";
        assert!(validator.is_valid(valid));

        // Invalid - missing equals
        let invalid = "DATABASE_URL postgresql://localhost/mydb\nAPI_KEY=secret_key";
        assert!(!validator.is_valid(invalid));

        // Invalid - empty key
        let invalid2 = "=postgresql://localhost/mydb\nAPI_KEY=secret_key";
        assert!(!validator.is_valid(invalid2));
    }

    #[test]
    fn test_env_confidence() {
        let repairer = EnvRepairer::new();

        // High confidence - uppercase keys with underscores
        let high_conf = "DATABASE_URL=postgresql://localhost/mydb\nAPI_KEY=secret_key";
        assert!(repairer.confidence(high_conf) >= 0.5);

        // Low confidence
        let low_conf = "some random text";
        assert!(repairer.confidence(low_conf) < 0.5);
    }

    #[test]
    fn test_fix_quoted_values() {
        let content = "DATABASE_URL=\"postgresql://localhost/mydb\nAPI_KEY='secret_key";
        let mut repairer = EnvRepairer::new();
        let result = repairer.repair(content).unwrap();
        // Should fix mismatched quotes by removing them
        assert!(result.contains("postgresql://localhost/mydb"));
        assert!(result.contains("secret_key"));
    }
}
