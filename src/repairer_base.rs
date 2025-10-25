//! Common base implementation for all repairers to follow DRY principle

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};

/// Generic repairer base that implements common repair logic
pub struct GenericRepairer {
    pub strategies: Vec<Box<dyn RepairStrategy>>,
    pub validator: Box<dyn Validator>,
}

impl GenericRepairer {
    /// Create a new generic repairer
    pub fn new(validator: Box<dyn Validator>, mut strategies: Vec<Box<dyn RepairStrategy>>) -> Self {
        // Sort strategies by priority (higher priority first)
        strategies.sort_by_key(|s| std::cmp::Reverse(s.priority()));
        
        Self { strategies, validator }
    }
    
    /// Apply all repair strategies to the content
    pub fn apply_strategies(&self, content: &str) -> Result<String> {
        let mut repaired = content.to_string();
        
        for strategy in &self.strategies {
            if let Ok(result) = strategy.apply(&repaired) {
                repaired = result;
            }
        }
        
        Ok(repaired)
    }
    
    /// Standard repair implementation
    pub fn repair_impl(&self, content: &str) -> Result<String> {
        let trimmed = content.trim();
        
        // Handle empty content
        if trimmed.is_empty() {
            return Ok(String::new());
        }
        
        // If already valid, return as-is
        if self.validator.is_valid(trimmed) {
            return Ok(trimmed.to_string());
        }
        
        // Apply repair strategies
        self.apply_strategies(trimmed)
    }
    
    /// Standard needs_repair implementation
    pub fn needs_repair_impl(&self, content: &str) -> bool {
        !self.validator.is_valid(content)
    }
    
    /// Standard confidence implementation
    pub fn confidence_impl(&self, content: &str, is_valid: bool) -> f64 {
        if is_valid {
            1.0
        } else {
            0.0
        }
    }
}
