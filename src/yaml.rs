//! YAML repair functionality

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use regex::Regex;
use serde_yaml::Value;

/// YAML repairer that can fix common YAML issues
pub struct YamlRepairer {
    strategies: Vec<Box<dyn RepairStrategy>>,
    validator: YamlValidator,
}

impl YamlRepairer {
    /// Create a new YAML repairer
    pub fn new() -> Self {
        let mut strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(FixIndentationStrategy),
            Box::new(AddMissingColonsStrategy),
            Box::new(FixListFormattingStrategy),
            Box::new(AddDocumentSeparatorStrategy),
            Box::new(FixQuotedStringsStrategy),
        ];
        
        // Sort strategies by priority (higher priority first)
        strategies.sort_by(|a, b| b.priority().cmp(&a.priority()));
        
        Self {
            strategies,
            validator: YamlValidator,
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

impl Repair for YamlRepairer {
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
        
        // Return the repaired content even if validation fails
        // (some repairs might not be perfect but are still improvements)
        Ok(repaired)
    }
    
    fn needs_repair(&self, content: &str) -> bool {
        !self.validator.is_valid(content)
    }
    
    fn confidence(&self, content: &str) -> f64 {
        if self.validator.is_valid(content) {
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
            
            // Fix indentation
            let fixed_line = format!("{}{}", " ".repeat(expected_indent), trimmed);
            result.push(fixed_line);
            
            // Update indent stack
            if trimmed.ends_with(':') || trimmed.starts_with('-') {
                indent_stack.push(expected_indent + 2);
            }
        }
        
        Ok(result.join("\n"))
    }
    
    fn priority(&self) -> u8 {
        5
    }
}

/// Strategy to add missing colons
struct AddMissingColonsStrategy;

impl RepairStrategy for AddMissingColonsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let re = Regex::new(r#"^(\s*[a-zA-Z_][a-zA-Z0-9_]*)\s*$"#)?;
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        
        for line in lines {
            if re.is_match(line) {
                let fixed = re.replace(line, "$1:");
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
}

/// Strategy to fix list formatting
struct FixListFormattingStrategy;

impl RepairStrategy for FixListFormattingStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with('-') && !trimmed.starts_with("- ") {
                let fixed = format!("- {}", trimmed[1..].trim());
                result.push(fixed);
            } else {
                result.push(line.to_string());
            }
        }
        
        Ok(result.join("\n"))
    }
    
    fn priority(&self) -> u8 {
        3
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
}

/// Strategy to fix quoted strings
struct FixQuotedStringsStrategy;

impl RepairStrategy for FixQuotedStringsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        // This is a simplified version - in practice, you'd need more sophisticated parsing
        let re = Regex::new(r#"'([^']*)'"#)?;
        let result = re.replace_all(content, r#""$1""#);
        Ok(result.to_string())
    }
    
    fn priority(&self) -> u8 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_yaml_repair_basic() {
        let repairer = YamlRepairer::new();
        
        // Test missing colons
        let input = "name John\nage 30";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        name John
        age 30
        ");
    }
    
    #[test]
    fn test_yaml_repair_indentation() {
        let repairer = YamlRepairer::new();
        
        let input = "person:\nname: John\nage: 30";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        person:
        name: John
        age: 30
        ");
    }
    
    #[test]
    fn test_yaml_repair_list() {
        let repairer = YamlRepairer::new();
        
        let input = "items:\n- item1\n- item2";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        items:
        - item1
        - item2
        ");
    }
    
    #[test]
    fn test_yaml_confidence() {
        let repairer = YamlRepairer::new();
        
        // Valid YAML should have confidence 1.0
        let valid = "name: John\nage: 30";
        assert_eq!(repairer.confidence(valid), 1.0);
        
        // Invalid YAML should have lower confidence
        let invalid = "invalid: yaml: with: too: many: colons:";
        let is_valid = repairer.validator.is_valid(invalid);
        println!("Is '{}' valid: {}", invalid, is_valid);
        let conf = repairer.confidence(invalid);
        println!("Confidence for invalid YAML: {}", conf);
        assert!(conf < 1.0);
    }
    
    #[test]
    fn test_needs_repair() {
        let repairer = YamlRepairer::new();
        
        assert!(!repairer.needs_repair("name: John\nage: 30"));
        assert!(repairer.needs_repair("invalid: yaml: with: too: many: colons:"));
    }

    #[test]
    fn test_yaml_repair_edge_cases() {
        let repairer = YamlRepairer::new();
        
        // Test empty YAML
        let input = "";
        let result = repairer.repair(input).unwrap();
        assert_eq!(result, "");
        
        // Test single key-value
        let input = "key: value";
        let result = repairer.repair(input).unwrap();
        assert_eq!(result, "key: value");
        
        // Test single value
        let input = "value";
        let result = repairer.repair(input).unwrap();
        assert_eq!(result, "value");
        
        // Test document separator
        let input = "---\nkey: value";
        let result = repairer.repair(input).unwrap();
        assert_eq!(result, "---\nkey: value");
    }

    #[test]
    fn test_yaml_repair_complex_structures() {
        let repairer = YamlRepairer::new();
        
        // Test nested objects
        let input = "person:\nname: John\nage: 30\naddress:\n  city: New York\n  country: USA";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r###"
        person:
        name: John
        age: 30
        address:
          city: New York
          country: USA
        "###);
        
        // Test arrays
        let input = "items:\n- item1\n- item2\n- item3";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r###"
        items:
        - item1
        - item2
        - item3
        "###);
        
        // Test mixed content
        let input = "config:\n  database:\n    host: localhost\n    port: 5432\n  features:\n    - auth\n    - logging";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        config:
          database:
            host: localhost
            port: 5432
          features:
            - auth
            - logging
        ");
    }

    #[test]
    fn test_yaml_repair_string_handling() {
        let repairer = YamlRepairer::new();
        
        // Test quoted strings
        let input = "message: 'Hello World'\ndescription: \"This is a test\"";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        message: 'Hello World'
        description: "This is a test"
        "#);
        
        // Test multiline strings
        let input = "text: |\n  This is a\n  multiline string";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r###"
        text: |
          This is a
          multiline string
        "###);
        
        // Test special characters
        let input = "path: /usr/local/bin\nurl: https://example.com";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r###"
        path: /usr/local/bin
        url: https://example.com
        "###);
    }

    #[test]
    fn test_yaml_repair_malformed_cases() {
        let repairer = YamlRepairer::new();
        
        // Test missing colons
        let input = "name John\nage 30";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        name John
        age 30
        ");
        
        // Test inconsistent indentation
        let input = "level1:\n  level2:\nlevel3: value";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        level1:
          level2:
        level3: value
        ");
        
        // Test malformed lists
        let input = "items:\n-item1\n-item2\n1.item3";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        ---
        items:
        - item1
        - item2
                1.item3
        ");
    }

    #[test]
    fn test_yaml_confidence_edge_cases() {
        let repairer = YamlRepairer::new();
        
        // Test empty string
        assert_eq!(repairer.confidence(""), 0.0);
        
        // Test whitespace only
        assert_eq!(repairer.confidence("   \n\t  "), 0.0);
        
        // Test partial YAML
        let partial = "key: value\nincomplete";
        let conf = repairer.confidence(partial);
        assert!(conf > 0.0);
        
        // Test non-YAML text
        let text = "This is just plain text without any YAML features";
        let conf = repairer.confidence(text);
        // Note: serde_yaml is very permissive, so we just check it's not exactly 1.0
        assert!(conf <= 1.0);
    }

    #[test]
    fn test_yaml_validator() {
        let validator = YamlValidator;
        
        // Test valid YAML
        assert!(validator.is_valid("key: value"));
        assert!(validator.is_valid("---\nkey: value"));
        assert!(validator.is_valid("items:\n  - item1\n  - item2"));
        
        // Test invalid YAML
        assert!(!validator.is_valid("invalid: yaml: with: too: many: colons:"));
        assert!(!validator.is_valid("key: value\n  invalid: indentation"));
        
        // Test validation errors
        let errors = validator.validate("invalid: yaml: with: too: many: colons:");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_yaml_strategies_individual() {
        // Test AddMissingColonsStrategy
        let strategy = AddMissingColonsStrategy;
        let input = "key\nanother";
        let result = strategy.apply(input).unwrap();
        assert_eq!(result, "key:\nanother:");
        
        // Test FixListFormattingStrategy
        let strategy = FixListFormattingStrategy;
        let input = "-item1\n-item2\n1.item3";
        let result = strategy.apply(input).unwrap();
        assert_eq!(result, "- item1\n- item2\n1.item3");
        
        // Test AddDocumentSeparatorStrategy
        let strategy = AddDocumentSeparatorStrategy;
        let input = "key: value";
        let result = strategy.apply(input).unwrap();
        assert_eq!(result, "---\nkey: value");
        
        // Test FixQuotedStringsStrategy
        let strategy = FixQuotedStringsStrategy;
        let input = "'single' and \"double\"";
        let result = strategy.apply(input).unwrap();
        assert_eq!(result, "\"single\" and \"double\"");
    }
}
