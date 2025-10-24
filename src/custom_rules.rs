use crate::config::{CustomRule, RepairConfig, RuleCondition, ConditionOperator};
use crate::error::RepairError;
use regex::Regex;
use std::collections::HashMap;

/// Custom rule engine for applying user-defined repair rules
pub struct CustomRuleEngine {
    pub rules: HashMap<String, Vec<CustomRule>>,
    compiled_patterns: HashMap<String, Regex>,
}

impl CustomRuleEngine {
    /// Create a new custom rule engine
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            compiled_patterns: HashMap::new(),
        }
    }

    /// Load rules from configuration
    pub fn load_from_config(&mut self, config: &RepairConfig) -> Result<(), RepairError> {
        self.rules.clear();
        self.compiled_patterns.clear();

        for rule in &config.custom_rules {
            if !rule.enabled {
                continue;
            }

            // Compile regex pattern
            let regex = Regex::new(&rule.pattern)
                .map_err(|e| RepairError::generic(format!("Invalid regex pattern '{}': {}", rule.pattern, e)))?;
            
            self.compiled_patterns.insert(rule.id.clone(), regex);

            // Group rules by format
            self.rules
                .entry(rule.target_format.clone())
                .or_insert_with(Vec::new)
                .push(rule.clone());
        }

        // Sort rules by priority (higher priority first)
        for rules in self.rules.values_mut() {
            rules.sort_by_key(|r| std::cmp::Reverse(r.priority));
        }

        Ok(())
    }

    /// Apply custom rules to content
    pub fn apply_rules(&self, content: &str, format: &str) -> Result<String, RepairError> {
        let mut result = content.to_string();
        
        if let Some(rules) = self.rules.get(format) {
            for rule in rules {
                if let Some(regex) = self.compiled_patterns.get(&rule.id) {
                    // Check conditions
                    if self.evaluate_conditions(&rule.conditions, &result)? {
                        // Apply the rule
                        result = regex.replace_all(&result, &rule.replacement).to_string();
                        
                        if crate::config::GlobalConfig::default().verbose {
                            println!("Applied custom rule '{}' to {} content", rule.name, format);
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Evaluate rule conditions
    fn evaluate_conditions(&self, conditions: &[RuleCondition], content: &str) -> Result<bool, RepairError> {
        for condition in conditions {
            let field_value = self.extract_field_value(&condition.field, content)?;
            let matches = self.compare_values(&field_value, &condition.operator, &condition.value)?;
            
            if !matches {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Extract field value from content
    fn extract_field_value(&self, field: &str, content: &str) -> Result<String, RepairError> {
        match field {
            "content_length" => Ok(content.len().to_string()),
            "line_count" => Ok(content.lines().count().to_string()),
            "contains_newlines" => Ok(content.contains('\n').to_string()),
            "starts_with_whitespace" => Ok(content.starts_with(char::is_whitespace).to_string()),
            "ends_with_whitespace" => Ok(content.ends_with(char::is_whitespace).to_string()),
            _ => {
                // Try to extract using regex
                let regex = Regex::new(field)
                    .map_err(|e| RepairError::generic(format!("Invalid field pattern '{}': {}", field, e)))?;
                
                if let Some(captures) = regex.captures(content) {
                    Ok(captures.get(1).map_or("", |m| m.as_str()).to_string())
                } else {
                    Ok(String::new())
                }
            }
        }
    }

    /// Compare values based on operator
    fn compare_values(&self, field_value: &str, operator: &ConditionOperator, expected_value: &str) -> Result<bool, RepairError> {
        match operator {
            ConditionOperator::Equals => Ok(field_value == expected_value),
            ConditionOperator::NotEquals => Ok(field_value != expected_value),
            ConditionOperator::Contains => Ok(field_value.contains(expected_value)),
            ConditionOperator::NotContains => Ok(!field_value.contains(expected_value)),
            ConditionOperator::StartsWith => Ok(field_value.starts_with(expected_value)),
            ConditionOperator::EndsWith => Ok(field_value.ends_with(expected_value)),
            ConditionOperator::Matches => {
                let regex = Regex::new(expected_value)
                    .map_err(|e| RepairError::generic(format!("Invalid regex in condition: {}", e)))?;
                Ok(regex.is_match(field_value))
            }
            ConditionOperator::NotMatches => {
                let regex = Regex::new(expected_value)
                    .map_err(|e| RepairError::generic(format!("Invalid regex in condition: {}", e)))?;
                Ok(!regex.is_match(field_value))
            }
        }
    }

    /// Get available rules for a format
    pub fn get_rules_for_format(&self, format: &str) -> Vec<&CustomRule> {
        self.rules.get(format).map_or_else(Vec::new, |rules| rules.iter().collect())
    }

    /// Check if there are any rules for a format
    pub fn has_rules_for_format(&self, format: &str) -> bool {
        self.rules.get(format).map_or(false, |rules| !rules.is_empty())
    }

    /// Get rule statistics
    pub fn get_statistics(&self) -> RuleStatistics {
        let total_rules = self.rules.values().map(|rules| rules.len()).sum();
        let enabled_rules = self.rules.values()
            .flat_map(|rules| rules.iter())
            .filter(|rule| rule.enabled)
            .count();
        let format_count = self.rules.len();

        RuleStatistics {
            total_rules,
            enabled_rules,
            format_count,
        }
    }
}

/// Statistics about custom rules
#[derive(Debug, Clone)]
pub struct RuleStatistics {
    pub total_rules: usize,
    pub enabled_rules: usize,
    pub format_count: usize,
}

impl Default for CustomRuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined rule templates for common scenarios
pub struct RuleTemplates;

impl RuleTemplates {
    /// Create a rule for adding missing quotes around unquoted strings
    pub fn add_quotes_around_strings(format: &str) -> CustomRule {
        CustomRule {
            id: format!("add_quotes_{}", format),
            name: format!("Add quotes around unquoted strings ({})", format),
            description: "Automatically add quotes around unquoted string values".to_string(),
            target_format: format.to_string(),
            priority: 5,
            enabled: true,
            pattern: r#"(\w+)\s*:\s*([^",\s][^,]*[^",\s])\s*([,\n])"#.to_string(),
            replacement: r#"$1: "$2"$3"#.to_string(),
            conditions: vec![],
        }
    }

    /// Create a rule for fixing missing colons in key-value pairs
    pub fn add_missing_colons(format: &str) -> CustomRule {
        CustomRule {
            id: format!("add_colons_{}", format),
            name: format!("Add missing colons ({})", format),
            description: "Add missing colons in key-value pairs".to_string(),
            target_format: format.to_string(),
            priority: 6,
            enabled: true,
            pattern: r#"^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s+([^:\s].*)$"#.to_string(),
            replacement: r#"$1$2: $3"#.to_string(),
            conditions: vec![],
        }
    }

    /// Create a rule for fixing malformed headers
    pub fn fix_malformed_headers() -> CustomRule {
        CustomRule {
            id: "fix_malformed_headers".to_string(),
            name: "Fix malformed headers".to_string(),
            description: "Add spaces after hash symbols in headers".to_string(),
            target_format: "markdown".to_string(),
            priority: 7,
            enabled: true,
            pattern: r#"^(#{1,6})([^\s#])"#.to_string(),
            replacement: r#"$1 $2"#.to_string(),
            conditions: vec![],
        }
    }

    /// Create a rule for fixing missing equals signs
    pub fn add_missing_equals() -> CustomRule {
        CustomRule {
            id: "add_missing_equals".to_string(),
            name: "Add missing equals signs".to_string(),
            description: "Add missing equals signs in key-value pairs".to_string(),
            target_format: "ini".to_string(),
            priority: 6,
            enabled: true,
            pattern: r#"^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s+([^=\s].*)$"#.to_string(),
            replacement: r#"$1$2 = $3"#.to_string(),
            conditions: vec![],
        }
    }

    /// Create a rule for fixing missing quotes in CSV
    pub fn fix_csv_quotes() -> CustomRule {
        CustomRule {
            id: "fix_csv_quotes".to_string(),
            name: "Fix missing quotes in CSV".to_string(),
            description: "Add quotes around fields containing spaces or commas".to_string(),
            target_format: "csv".to_string(),
            priority: 5,
            enabled: true,
            pattern: r#"([^",\n]*[,\s][^",\n]*)"#.to_string(),
            replacement: r#""$1""#.to_string(),
            conditions: vec![],
        }
    }

    /// Get all predefined templates
    pub fn get_all_templates() -> Vec<CustomRule> {
        vec![
            Self::add_quotes_around_strings("json"),
            Self::add_quotes_around_strings("yaml"),
            Self::add_quotes_around_strings("toml"),
            Self::add_missing_colons("yaml"),
            Self::add_missing_colons("json"),
            Self::fix_malformed_headers(),
            Self::add_missing_equals(),
            Self::fix_csv_quotes(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RepairConfig;

    #[test]
    fn test_custom_rule_engine() {
        let mut engine = CustomRuleEngine::new();
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
        engine.load_from_config(&config).unwrap();
        
        let result = engine.apply_rules("hello world", "json").unwrap();
        assert_eq!(result, r#""hello" "world""#);
    }

    #[test]
    fn test_rule_conditions() {
        let mut engine = CustomRuleEngine::new();
        let mut config = RepairConfig::new();
        
        let rule = CustomRule {
            id: "conditional_rule".to_string(),
            name: "Conditional Rule".to_string(),
            description: "A rule with conditions".to_string(),
            target_format: "json".to_string(),
            priority: 5,
            enabled: true,
            pattern: r#"(\w+)"#.to_string(),
            replacement: r#""$1""#.to_string(),
            conditions: vec![
                RuleCondition {
                    field: "content_length".to_string(),
                    operator: ConditionOperator::NotEquals,
                    value: "2".to_string(),
                }
            ],
        };
        
        config.add_custom_rule(rule);
        engine.load_from_config(&config).unwrap();
        
        // Short content should not trigger the rule
        let result = engine.apply_rules("hi", "json").unwrap();
        assert_eq!(result, "hi");
        
        // Long content should trigger the rule
        let result = engine.apply_rules("hello world", "json").unwrap();
        assert_eq!(result, r#""hello" "world""#);
    }

    #[test]
    fn test_rule_templates() {
        let templates = RuleTemplates::get_all_templates();
        assert!(!templates.is_empty());
        
        let json_quote_rule = templates.iter()
            .find(|r| r.id == "add_quotes_json")
            .unwrap();
        assert_eq!(json_quote_rule.target_format, "json");
        assert_eq!(json_quote_rule.priority, 5);
    }
}
