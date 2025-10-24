//! Core traits for repair functionality

use crate::error::Result;

/// Trait for repairing content of various formats
pub trait Repair {
    /// Repair the given content and return the repaired version
    fn repair(&self, content: &str) -> Result<String>;
    
    /// Check if the content needs repair
    fn needs_repair(&self, content: &str) -> bool;
    
    /// Get the confidence score for repair (0.0 to 1.0)
    fn confidence(&self, content: &str) -> f64;
}

/// Trait for format-specific repair strategies
pub trait RepairStrategy {
    /// Apply the repair strategy to the content
    fn apply(&self, content: &str) -> Result<String>;
    
    /// Get the priority of this strategy (higher = more important)
    fn priority(&self) -> u8;
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
