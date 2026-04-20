//! Generic repair loop: validator gate + ordered `RepairStrategy` pipeline.

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};

/// Composes a `Validator` with strategy objects (sorted by `priority`, high first).
pub struct GenericRepairer {
    strategies: Vec<Box<dyn RepairStrategy>>,
    validator: Box<dyn Validator>,
}

impl GenericRepairer {
    /// Create a new generic repairer with validator and strategies
    pub fn new(
        validator: Box<dyn Validator>,
        mut strategies: Vec<Box<dyn RepairStrategy>>,
    ) -> Self {
        // Sort strategies by priority (higher priority first)
        strategies.sort_by_key(|s| std::cmp::Reverse(s.priority()));

        Self {
            strategies,
            validator,
        }
    }

    /// Apply all repair strategies to the content
    fn apply_strategies_internal(&mut self, content: &str) -> Result<String> {
        let mut repaired = content.to_string();

        for strategy in self.strategies.iter() {
            if let Ok(result) = strategy.apply(&repaired) {
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

        // Handle empty content
        if trimmed.is_empty() {
            return Ok(String::new());
        }

        // If already valid, return as-is
        if self.validator.is_valid(trimmed) {
            return Ok(trimmed.to_string());
        }

        // Apply repair strategies
        let repaired = self.apply_strategies_internal(trimmed)?;

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
