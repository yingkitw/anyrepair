//! Common base implementation for all repairers to follow DRY principle
//! 
//! This module provides trait-based generic implementations that eliminate
//! code duplication across all format-specific repairers.

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};

/// Generic trait-based repairer that implements common repair logic
/// 
/// This struct uses composition with trait objects to provide a unified
/// implementation for all format-specific repairers, following the DRY principle.
pub struct GenericRepairer {
    strategies: Vec<Box<dyn RepairStrategy>>,
    validator: Box<dyn Validator>,
    repair_log: Vec<String>,
    logging_enabled: bool,
}

impl GenericRepairer {
    /// Create a new generic repairer with validator and strategies
    pub fn new(validator: Box<dyn Validator>, mut strategies: Vec<Box<dyn RepairStrategy>>) -> Self {
        // Sort strategies by priority (higher priority first)
        strategies.sort_by_key(|s| std::cmp::Reverse(s.priority()));
        
        Self { 
            strategies, 
            validator,
            repair_log: Vec::new(),
            logging_enabled: false,
        }
    }
    
    /// Enable or disable logging
    pub fn with_logging(mut self, enabled: bool) -> Self {
        self.logging_enabled = enabled;
        self
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
    fn log(&mut self, message: impl Into<String>) {
        if self.logging_enabled {
            self.repair_log.push(message.into());
        }
    }
    
    /// Apply all repair strategies to the content
    fn apply_strategies_internal(&mut self, content: &str) -> Result<String> {
        let mut repaired = content.to_string();
        
        for strategy in self.strategies.iter() {
            if let Ok(result) = strategy.apply(&repaired) {
                if self.logging_enabled && result != repaired {
                    let strategy_name = strategy.name().to_string();
                    self.repair_log.push(format!("Applied strategy: {}", strategy_name));
                }
                repaired = result;
            }
        }
        
        Ok(repaired)
    }
    
    /// Get the validator
    pub fn validator(&self) -> &dyn Validator {
        self.validator.as_ref()
    }
    
    /// Get the strategies
    pub fn strategies(&self) -> &[Box<dyn RepairStrategy>] {
        &self.strategies
    }
}

impl Repair for GenericRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        let trimmed = content.trim();
        
        // Clear previous log
        self.clear_log();
        
        // Handle empty content
        if trimmed.is_empty() {
            self.log("Content is empty");
            return Ok(String::new());
        }
        
        // If already valid, return as-is
        if self.validator.is_valid(trimmed) {
            self.log("Content is already valid, no repairs needed");
            return Ok(trimmed.to_string());
        }
        
        self.log("Starting repair process");
        
        // Apply repair strategies
        let repaired = self.apply_strategies_internal(trimmed)?;
        
        self.log("Repair process completed");
        
        Ok(repaired)
    }
    
    fn needs_repair(&self, content: &str) -> bool {
        !self.validator.is_valid(content)
    }
    
    fn confidence(&self, content: &str) -> f64 {
        if self.validator.is_valid(content) {
            1.0
        } else {
            0.0
        }
    }
}
