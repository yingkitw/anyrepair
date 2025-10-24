use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for custom repair rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairConfig {
    /// Global settings
    pub global: GlobalConfig,
    /// Format-specific configurations
    pub formats: HashMap<String, FormatConfig>,
    /// Custom repair rules
    pub custom_rules: Vec<CustomRule>,
}

/// Global configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Maximum number of repair attempts
    pub max_attempts: u32,
    /// Enable parallel processing
    pub parallel_processing: bool,
    /// Minimum confidence threshold for repairs
    pub min_confidence: f64,
    /// Enable verbose logging
    pub verbose: bool,
}

/// Format-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    /// Enable specific strategies for this format
    pub enabled_strategies: Vec<String>,
    /// Disable specific strategies for this format
    pub disabled_strategies: Vec<String>,
    /// Custom validation rules
    pub validation_rules: Vec<ValidationRule>,
    /// Repair-specific settings
    pub repair_settings: RepairSettings,
}

/// Custom repair rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    /// Unique identifier for the rule
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this rule does
    pub description: String,
    /// Target format (json, yaml, markdown, etc.)
    pub target_format: String,
    /// Priority (higher numbers = higher priority)
    pub priority: u8,
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Pattern to match
    pub pattern: String,
    /// Replacement pattern
    pub replacement: String,
    /// Conditions for applying the rule
    pub conditions: Vec<RuleCondition>,
}

/// Validation rule for custom validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule name
    pub name: String,
    /// Pattern to validate
    pub pattern: String,
    /// Error message if validation fails
    pub error_message: String,
    /// Whether to treat as warning or error
    pub severity: ValidationSeverity,
}

/// Severity level for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Warning,
    Error,
}

/// Repair-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairSettings {
    /// Maximum line length for formatting
    pub max_line_length: Option<usize>,
    /// Indentation style
    pub indentation: IndentationStyle,
    /// Quote style preference
    pub quote_style: QuoteStyle,
    /// Whether to preserve original formatting where possible
    pub preserve_formatting: bool,
}

/// Indentation style options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndentationStyle {
    Spaces(usize),
    Tabs,
}

/// Quote style options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuoteStyle {
    Single,
    Double,
    Auto,
}

/// Rule condition for custom rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    /// Field to check
    pub field: String,
    /// Operator for comparison
    pub operator: ConditionOperator,
    /// Value to compare against
    pub value: String,
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Matches,
    NotMatches,
}

impl Default for RepairConfig {
    fn default() -> Self {
        Self {
            global: GlobalConfig::default(),
            formats: HashMap::new(),
            custom_rules: Vec::new(),
        }
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            parallel_processing: true,
            min_confidence: 0.5,
            verbose: false,
        }
    }
}

impl Default for RepairSettings {
    fn default() -> Self {
        Self {
            max_line_length: Some(120),
            indentation: IndentationStyle::Spaces(2),
            quote_style: QuoteStyle::Auto,
            preserve_formatting: true,
        }
    }
}

impl RepairConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: RepairConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add a custom rule
    pub fn add_custom_rule(&mut self, rule: CustomRule) {
        self.custom_rules.push(rule);
    }

    /// Get configuration for a specific format
    pub fn get_format_config(&self, format: &str) -> Option<&FormatConfig> {
        self.formats.get(format)
    }

    /// Get enabled custom rules for a format
    pub fn get_enabled_rules_for_format(&self, format: &str) -> Vec<&CustomRule> {
        self.custom_rules
            .iter()
            .filter(|rule| rule.enabled && rule.target_format == format)
            .collect()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate global config
        if self.global.max_attempts == 0 {
            errors.push("max_attempts must be greater than 0".to_string());
        }

        if self.global.min_confidence < 0.0 || self.global.min_confidence > 1.0 {
            errors.push("min_confidence must be between 0.0 and 1.0".to_string());
        }

        // Validate custom rules
        for rule in &self.custom_rules {
            if rule.id.is_empty() {
                errors.push("Custom rule ID cannot be empty".to_string());
            }

            if rule.pattern.is_empty() {
                errors.push(format!("Custom rule '{}' pattern cannot be empty", rule.id));
            }

            if rule.priority > 10 {
                errors.push(format!("Custom rule '{}' priority must be between 0 and 10", rule.id));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Configuration builder for easy setup
pub struct RepairConfigBuilder {
    config: RepairConfig,
}

impl RepairConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: RepairConfig::new(),
        }
    }

    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.config.global.max_attempts = max_attempts;
        self
    }

    pub fn with_parallel_processing(mut self, enabled: bool) -> Self {
        self.config.global.parallel_processing = enabled;
        self
    }

    pub fn with_min_confidence(mut self, confidence: f64) -> Self {
        self.config.global.min_confidence = confidence;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.config.global.verbose = verbose;
        self
    }

    pub fn add_custom_rule(mut self, rule: CustomRule) -> Self {
        self.config.add_custom_rule(rule);
        self
    }

    pub fn add_format_config(mut self, format: String, config: FormatConfig) -> Self {
        self.config.formats.insert(format, config);
        self
    }

    pub fn build(self) -> RepairConfig {
        self.config
    }
}

impl Default for RepairConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RepairConfig::new();
        assert_eq!(config.global.max_attempts, 3);
        assert!(config.global.parallel_processing);
        assert_eq!(config.global.min_confidence, 0.5);
        assert!(!config.global.verbose);
    }

    #[test]
    fn test_config_builder() {
        let config = RepairConfigBuilder::new()
            .with_max_attempts(5)
            .with_parallel_processing(false)
            .with_min_confidence(0.8)
            .with_verbose(true)
            .build();

        assert_eq!(config.global.max_attempts, 5);
        assert!(!config.global.parallel_processing);
        assert_eq!(config.global.min_confidence, 0.8);
        assert!(config.global.verbose);
    }

    #[test]
    fn test_custom_rule() {
        let mut config = RepairConfig::new();
        let rule = CustomRule {
            id: "test_rule".to_string(),
            name: "Test Rule".to_string(),
            description: "A test rule".to_string(),
            target_format: "json".to_string(),
            priority: 5,
            enabled: true,
            pattern: r#"(\w+)"#.to_string(),
            replacement: r#""$1""#.to_string(),
            conditions: vec![],
        };

        config.add_custom_rule(rule);
        assert_eq!(config.custom_rules.len(), 1);
        assert_eq!(config.custom_rules[0].id, "test_rule");
    }

    #[test]
    fn test_config_validation() {
        let mut config = RepairConfig::new();
        config.global.max_attempts = 0; // Invalid
        config.global.min_confidence = 1.5; // Invalid

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 2);
    }
}
