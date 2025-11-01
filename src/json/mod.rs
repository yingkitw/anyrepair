//! JSON repair module
//! 
//! Provides comprehensive JSON repair functionality with multiple strategies
//! for fixing common JSON issues from LLM outputs.

pub mod strategies;
pub mod validator;

pub use strategies::{
    StripTrailingContentStrategy, FixTrailingCommasStrategy, FixSingleQuotesStrategy,
    AddMissingQuotesStrategy, FixMalformedNumbersStrategy, FixBooleanNullStrategy,
    AddMissingBracesStrategy, FixAgenticAiResponseStrategy, get_regex_cache,
};
pub use validator::JsonValidator;

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use serde_json::Value;

/// JSON repairer that can fix common JSON issues
pub struct JsonRepairer {
    strategies: Vec<Box<dyn RepairStrategy>>,
    validator: JsonValidator,
    logging: bool,
    repair_log: Vec<String>,
}

impl JsonRepairer {
    /// Create a new JSON repairer
    pub fn new() -> Self {
        let mut strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(StripTrailingContentStrategy),
            Box::new(AddMissingQuotesStrategy),
            Box::new(FixTrailingCommasStrategy),
            Box::new(AddMissingBracesStrategy),
            Box::new(FixSingleQuotesStrategy),
            Box::new(FixMalformedNumbersStrategy),
            Box::new(FixBooleanNullStrategy),
            Box::new(FixAgenticAiResponseStrategy),
        ];
        
        // Sort strategies by priority (higher priority first)
        strategies.sort_by_key(|b| std::cmp::Reverse(b.priority()));
        
        Self {
            strategies,
            validator: JsonValidator,
            logging: false,
            repair_log: Vec::new(),
        }
    }

    /// Create a new JSON repairer with logging enabled
    pub fn with_logging(logging: bool) -> Self {
        let mut repairer = Self::new();
        repairer.logging = logging;
        repairer
    }

    /// Get the repair log
    pub fn get_repair_log(&self) -> &[String] {
        &self.repair_log
    }

    /// Clear the repair log
    pub fn clear_log(&mut self) {
        self.repair_log.clear();
    }

    /// Log a repair action
    fn log(&mut self, message: &str) {
        if self.logging {
            self.repair_log.push(message.to_string());
        }
    }
    
    /// Apply all repair strategies to the content
    fn apply_strategies(&mut self, content: &str) -> Result<String> {
        let mut repaired = content.to_string();
        let logging = self.logging;
        
        for strategy in &self.strategies {
            if let Ok(result) = strategy.apply(&repaired) {
                if result != repaired && logging {
                    let strategy_name = strategy.name().to_string();
                    self.repair_log.push(format!("Applied strategy: {}", strategy_name));
                }
                repaired = result;
            }
        }
        
        Ok(repaired)
    }
}

impl Default for JsonRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for JsonRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        let trimmed = content.trim();
        
        // Clear previous log
        self.repair_log.clear();
        
        // If already valid, return as-is
        if self.validator.is_valid(trimmed) {
            self.log("JSON was already valid, no repairs needed");
            return Ok(trimmed.to_string());
        }
        
        self.log("Starting JSON repair process");
        
        // Apply repair strategies
        let repaired = self.apply_strategies(trimmed)?;
        
        self.log("JSON repair completed");
        
        // Return the repaired content even if validation fails
        Ok(repaired)
    }
    
    fn needs_repair(&self, content: &str) -> bool {
        !self.validator.is_valid(content)
    }
    
    fn confidence(&self, content: &str) -> f64 {
        if self.validator.is_valid(content) {
            return 1.0;
        }
        
        // Calculate confidence based on how close we are to valid JSON
        let mut score: f64 = 0.0;
        
        // Check for basic JSON structure
        if content.contains('{') || content.contains('[') {
            score += 0.3;
        }
        
        // Check for key-value pairs
        if content.contains(':') {
            score += 0.2;
        }
        
        // Check for quotes
        if content.contains('"') {
            score += 0.2;
        }
        
        // Check for commas
        if content.contains(',') {
            score += 0.1;
        }
        
        // Check for balanced braces/brackets
        let open_braces = content.matches('{').count();
        let close_braces = content.matches('}').count();
        let open_brackets = content.matches('[').count();
        let close_brackets = content.matches(']').count();
        
        if open_braces == close_braces && open_brackets == close_brackets {
            score += 0.2;
        }
        
        score.min(1.0_f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_repairer_creation() {
        let repairer = JsonRepairer::new();
        assert!(!repairer.strategies.is_empty());
    }

    #[test]
    fn test_json_repairer_default() {
        let repairer = JsonRepairer::default();
        assert!(!repairer.strategies.is_empty());
    }

    #[test]
    fn test_json_repairer_with_logging() {
        let repairer = JsonRepairer::with_logging(true);
        assert!(repairer.logging);
    }

    #[test]
    fn test_json_confidence_valid() {
        let repairer = JsonRepairer::new();
        let confidence = repairer.confidence(r#"{"key": "value"}"#);
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_json_confidence_invalid() {
        let repairer = JsonRepairer::new();
        let confidence = repairer.confidence(r#"{"key": value}"#);
        assert!(confidence < 1.0);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_json_needs_repair() {
        let repairer = JsonRepairer::new();
        assert!(!repairer.needs_repair(r#"{"key": "value"}"#));
        assert!(repairer.needs_repair(r#"{"key": "value",}"#));
    }
}
