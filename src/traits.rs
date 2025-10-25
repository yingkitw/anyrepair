//! Core traits for repair functionality

use crate::error::Result;

/// Trait for repairing content of various formats
pub trait Repair {
    /// Repair the given content and return the repaired version
    fn repair(&mut self, content: &str) -> Result<String>;
    
    /// Check if the content needs repair
    fn needs_repair(&self, content: &str) -> bool;
    
    /// Get the confidence score for repair (0.0 to 1.0)
    fn confidence(&self, content: &str) -> f64;
}

/// Base trait for format-specific repairers to reduce code duplication
pub trait BaseRepairer: Repair {
    /// Get the validator for this format
    fn validator(&self) -> &dyn Validator;
    
    /// Get the strategies for this format
    fn strategies(&self) -> &[Box<dyn RepairStrategy>];
    
    /// Default implementation of repair logic
    fn repair_impl(&self, content: &str) -> Result<String> {
        let trimmed = content.trim();
        
        // Handle empty content
        if trimmed.is_empty() {
            return Ok(String::new());
        }
        
        // If already valid, return as-is
        if self.validator().is_valid(trimmed) {
            return Ok(trimmed.to_string());
        }
        
        // Apply repair strategies
        let mut repaired = trimmed.to_string();
        for strategy in self.strategies() {
            if let Ok(result) = strategy.apply(&repaired) {
                repaired = result;
            }
        }
        
        Ok(repaired)
    }
}

/// Trait for format-specific repair strategies
pub trait RepairStrategy {
    /// Apply the repair strategy to the content
    fn apply(&self, content: &str) -> Result<String>;

    /// Get the priority of this strategy (higher = more important)
    fn priority(&self) -> u8;

    /// Get the name of this strategy
    fn name(&self) -> &str;
}

/// Trait for parallel-safe repair strategies
pub trait ParallelRepairStrategy: RepairStrategy + Send + Sync {
    /// Apply the repair strategy to the content (parallel-safe version)
    fn apply_parallel(&self, content: &str) -> Result<String> {
        self.apply(content)
    }
}

/// Trait for content validation
pub trait Validator {
    /// Validate the content and return true if valid
    fn is_valid(&self, content: &str) -> bool;
    
    /// Get validation errors if any
    fn validate(&self, content: &str) -> Vec<String>;
}
