//! Custom validation rules module for enterprise validation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation rule type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleType {
    /// Regex pattern matching
    Regex,
    /// Length constraint
    Length,
    /// Format constraint
    Format,
    /// Custom constraint
    Custom,
}

/// Validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule name
    pub name: String,
    /// Rule type
    pub rule_type: RuleType,
    /// Rule pattern or constraint
    pub pattern: String,
    /// Error message
    pub error_message: String,
    /// Whether rule is enabled
    pub enabled: bool,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Violations found
    pub violations: Vec<String>,
}

/// Custom validation rules engine
pub struct ValidationRulesEngine {
    rules: HashMap<String, ValidationRule>,
}

impl ValidationRulesEngine {
    /// Create a new validation rules engine
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    /// Add a validation rule
    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.insert(rule.name.clone(), rule);
    }

    /// Remove a validation rule
    pub fn remove_rule(&mut self, name: &str) -> Option<ValidationRule> {
        self.rules.remove(name)
    }

    /// Get a validation rule
    pub fn get_rule(&self, name: &str) -> Option<&ValidationRule> {
        self.rules.get(name)
    }

    /// Validate content against all enabled rules
    pub fn validate(&self, content: &str) -> ValidationResult {
        let mut violations = Vec::new();

        for rule in self.rules.values() {
            if !rule.enabled {
                continue;
            }

            match rule.rule_type {
                RuleType::Regex => {
                    if let Ok(regex) = regex::Regex::new(&rule.pattern) {
                        if !regex.is_match(content) {
                            violations.push(rule.error_message.clone());
                        }
                    }
                }
                RuleType::Length => {
                    if let Ok(max_len) = rule.pattern.parse::<usize>() {
                        if content.len() > max_len {
                            violations.push(rule.error_message.clone());
                        }
                    }
                }
                RuleType::Format => {
                    if !self.validate_format(content, &rule.pattern) {
                        violations.push(rule.error_message.clone());
                    }
                }
                RuleType::Custom => {
                    // Custom rules would be handled by external validators
                    violations.push(format!("Custom rule '{}' not implemented", rule.name));
                }
            }
        }

        ValidationResult {
            passed: violations.is_empty(),
            violations,
        }
    }

    /// Validate specific format
    fn validate_format(&self, content: &str, format: &str) -> bool {
        match format {
            "json" => serde_json::from_str::<serde_json::Value>(content).is_ok(),
            "xml" => {
                // Basic XML validation
                content.trim().starts_with('<') && content.trim().ends_with('>')
            }
            "yaml" => {
                // Basic YAML validation - just check it's not empty
                !content.trim().is_empty()
            }
            _ => true,
        }
    }

    /// Get all rules
    pub fn get_all_rules(&self) -> Vec<&ValidationRule> {
        self.rules.values().collect()
    }

    /// Clear all rules
    pub fn clear_rules(&mut self) {
        self.rules.clear();
    }

    /// Enable a rule
    pub fn enable_rule(&mut self, name: &str) -> bool {
        if let Some(rule) = self.rules.get_mut(name) {
            rule.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable a rule
    pub fn disable_rule(&mut self, name: &str) -> bool {
        if let Some(rule) = self.rules.get_mut(name) {
            rule.enabled = false;
            true
        } else {
            false
        }
    }
}

impl Default for ValidationRulesEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_rules_engine_creation() {
        let engine = ValidationRulesEngine::new();
        assert_eq!(engine.get_all_rules().len(), 0);
    }

    #[test]
    fn test_add_rule() {
        let mut engine = ValidationRulesEngine::new();
        let rule = ValidationRule {
            name: "test_rule".to_string(),
            rule_type: RuleType::Length,
            pattern: "100".to_string(),
            error_message: "Content too long".to_string(),
            enabled: true,
        };

        engine.add_rule(rule);
        assert_eq!(engine.get_all_rules().len(), 1);
    }

    #[test]
    fn test_validate_length() {
        let mut engine = ValidationRulesEngine::new();
        let rule = ValidationRule {
            name: "max_length".to_string(),
            rule_type: RuleType::Length,
            pattern: "10".to_string(),
            error_message: "Content exceeds max length".to_string(),
            enabled: true,
        };

        engine.add_rule(rule);

        let result = engine.validate("short");
        assert!(result.passed);

        let result = engine.validate("this is a very long string");
        assert!(!result.passed);
    }

    #[test]
    fn test_validate_format() {
        let mut engine = ValidationRulesEngine::new();
        let rule = ValidationRule {
            name: "json_format".to_string(),
            rule_type: RuleType::Format,
            pattern: "json".to_string(),
            error_message: "Invalid JSON format".to_string(),
            enabled: true,
        };

        engine.add_rule(rule);

        let result = engine.validate(r#"{"key": "value"}"#);
        assert!(result.passed);

        let result = engine.validate("not json");
        assert!(!result.passed);
    }

    #[test]
    fn test_enable_disable_rule() {
        let mut engine = ValidationRulesEngine::new();
        let rule = ValidationRule {
            name: "test_rule".to_string(),
            rule_type: RuleType::Length,
            pattern: "10".to_string(),
            error_message: "Too long".to_string(),
            enabled: true,
        };

        engine.add_rule(rule);
        assert!(engine.disable_rule("test_rule"));
        assert!(engine.enable_rule("test_rule"));
    }
}
