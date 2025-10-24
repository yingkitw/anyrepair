//! Example plugin for the anyrepair crate
//! 
//! This demonstrates how to create a custom plugin that extends the repair functionality.

use anyrepair::{
    plugin::{Plugin, PluginMetadata, PluginConfig},
    traits::{Repair, RepairStrategy, Validator},
    error::Result,
};

/// Example plugin that provides additional repair strategies
pub struct ExamplePlugin {
    metadata: PluginMetadata,
    enabled: bool,
}

impl ExamplePlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "example_plugin".to_string(),
                name: "Example Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "An example plugin demonstrating custom repair strategies".to_string(),
                author: "AnyRepair Team".to_string(),
                supported_formats: vec!["json".to_string(), "yaml".to_string()],
                dependencies: vec![],
                config_schema: None,
            },
            enabled: false,
        }
    }
}

impl Plugin for ExamplePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn initialize(&mut self, config: &PluginConfig) -> Result<()> {
        self.enabled = config.enabled;
        println!("Example plugin initialized with enabled={}", self.enabled);
        Ok(())
    }
    
    fn get_strategies(&self) -> Vec<Box<dyn RepairStrategy>> {
        if !self.enabled {
            return vec![];
        }
        
        vec![
            Box::new(ExampleStrategy::new()),
        ]
    }
    
    fn get_validator(&self) -> Option<Box<dyn Validator>> {
        if !self.enabled {
            return None;
        }
        
        Some(Box::new(ExampleValidator::new()))
    }
    
    fn get_repairer(&self) -> Option<Box<dyn Repair>> {
        if !self.enabled {
            return None;
        }
        
        Some(Box::new(ExampleRepairer::new()))
    }
    
    fn cleanup(&mut self) -> Result<()> {
        println!("Example plugin cleanup");
        Ok(())
    }
}

/// Example repair strategy
pub struct ExampleStrategy {
    _name: String,
}

impl ExampleStrategy {
    pub fn new() -> Self {
        Self {
            _name: "Example Strategy".to_string(),
        }
    }
}

impl RepairStrategy for ExampleStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        // Simple example: add a comment to JSON content
        if content.trim().starts_with('{') {
            Ok(format!("/* Repaired by Example Plugin */\n{}", content))
        } else {
            Ok(content.to_string())
        }
    }
    
    fn priority(&self) -> u8 {
        3
    }
}

/// Example validator
pub struct ExampleValidator;

impl ExampleValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Validator for ExampleValidator {
    fn is_valid(&self, content: &str) -> bool {
        // Simple example: check if content is not empty
        !content.trim().is_empty()
    }
    
    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        if content.trim().is_empty() {
            errors.push("Content is empty".to_string());
        }
        
        if content.len() > 10000 {
            errors.push("Content is too long".to_string());
        }
        
        errors
    }
}

/// Example repairer
pub struct ExampleRepairer;

impl ExampleRepairer {
    pub fn new() -> Self {
        Self
    }
}

impl Repair for ExampleRepairer {
    fn repair(&self, content: &str) -> Result<String> {
        // Simple example: add a header comment
        Ok(format!("<!-- Repaired by Example Plugin -->\n{}", content))
    }
    
    fn confidence(&self, content: &str) -> f64 {
        // Simple confidence scoring
        if content.trim().is_empty() {
            0.0
        } else if content.len() > 1000 {
            0.8
        } else {
            0.5
        }
    }
    
    fn needs_repair(&self, content: &str) -> bool {
        // Simple check: needs repair if content is not empty
        !content.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_plugin() {
        let mut plugin = ExamplePlugin::new();
        let config = PluginConfig {
            metadata: plugin.metadata().clone(),
            settings: std::collections::HashMap::new(),
            enabled: true,
            priority: 5,
        };
        
        plugin.initialize(&config).unwrap();
        assert!(plugin.enabled);
        
        let strategies = plugin.get_strategies();
        assert_eq!(strategies.len(), 1);
        
        let validator = plugin.get_validator();
        assert!(validator.is_some());
        
        let repairer = plugin.get_repairer();
        assert!(repairer.is_some());
    }
    
    #[test]
    fn test_example_strategy() {
        let strategy = ExampleStrategy::new();
        let result = strategy.apply(r#"{"key": "value"}"#).unwrap();
        assert!(result.contains("/* Repaired by Example Plugin */"));
    }
    
    #[test]
    fn test_example_validator() {
        let validator = ExampleValidator::new();
        assert!(validator.is_valid("valid content"));
        assert!(!validator.is_valid(""));
        
        let errors = validator.validate("");
        assert!(!errors.is_empty());
    }
    
    #[test]
    fn test_example_repairer() {
        let repairer = ExampleRepairer::new();
        let result = repairer.repair("test content").unwrap();
        assert!(result.contains("<!-- Repaired by Example Plugin -->"));
        
        let confidence = repairer.confidence("test");
        assert!(confidence > 0.0);
    }
}

fn main() {
    println!("Example Plugin for AnyRepair");
    println!("This is a demonstration of how to create a custom plugin.");
    println!("See the Plugin Development Guide for more information.");
}
