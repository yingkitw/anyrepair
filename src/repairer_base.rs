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

    /// Apply all repair strategies to the content, tracking which ones changed it.
    fn apply_strategies_with_explanations(&mut self, content: &str) -> Result<(String, Vec<String>)> {
        let mut repaired = content.to_string();
        let mut applied = Vec::new();

        for strategy in self.strategies.iter() {
            if let Ok(result) = strategy.apply(&repaired) {
                if result != repaired {
                    applied.push(strategy.name().to_string());
                    repaired = result;
                }
            }
        }

        Ok((repaired, applied))
    }

    /// Apply all repair strategies to the content
    fn apply_strategies_internal(&mut self, content: &str) -> Result<String> {
        let (repaired, _) = self.apply_strategies_with_explanations(content)?;
        Ok(repaired)
    }

    /// Repair content and return the list of strategy names that changed it.
    /// Returns `(repaired_content, applied_strategy_names)`.
    /// If the content is already valid, returns `(content, [])`.
    pub fn repair_with_explanations(&mut self, content: &str) -> Result<(String, Vec<String>)> {
        let trimmed = content.trim();

        if trimmed.is_empty() {
            return Ok((String::new(), Vec::new()));
        }

        if self.validator.is_valid(trimmed) {
            return Ok((trimmed.to_string(), Vec::new()));
        }

        self.apply_strategies_with_explanations(trimmed)
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
