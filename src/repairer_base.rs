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
    fn apply_strategies_with_explanations(&self, content: &str) -> Result<(String, Vec<String>)> {
        let mut repaired = content.to_string();
        let mut applied = Vec::new();

        for strategy in self.strategies.iter() {
            if let Ok(result) = strategy.apply(&repaired)
                && result != repaired {
                    applied.push(strategy.name().to_string());
                    repaired = result;
                }
        }

        Ok((repaired, applied))
    }

    /// Apply all repair strategies to the content
    fn apply_strategies_internal(&self, content: &str) -> Result<String> {
        let (repaired, _) = self.apply_strategies_with_explanations(content)?;
        Ok(repaired)
    }

    /// Repair content and return the list of strategy names that changed it.
    /// Returns `(repaired_content, applied_strategy_names)`.
    /// If the content is already valid, returns `(content, [])`.
    pub fn repair_with_explanations(&self, content: &str) -> Result<(String, Vec<String>)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple validator that considers content valid if it equals "ok".
    struct MockValidator;
    impl Validator for MockValidator {
        fn is_valid(&self, content: &str) -> bool {
            content == "ok"
        }
        fn validate(&self, content: &str) -> Vec<String> {
            if content == "ok" {
                Vec::new()
            } else {
                vec!["not ok".to_string()]
            }
        }
    }

    /// No-op strategy that returns content unchanged.
    struct NoopStrategy;
    impl RepairStrategy for NoopStrategy {
        fn apply(&self, content: &str) -> Result<String> {
            Ok(content.to_string())
        }
        fn priority(&self) -> u8 {
            0
        }
        fn name(&self) -> &str {
            "Noop"
        }
    }

    fn make_repairer() -> GenericRepairer {
        GenericRepairer::new(
            Box::new(MockValidator),
            vec![Box::new(NoopStrategy)],
        )
    }

    #[test]
    fn test_confidence_valid_content() {
        let repairer = make_repairer();
        assert_eq!(repairer.confidence("ok"), 1.0);
    }

    #[test]
    fn test_confidence_invalid_content() {
        let repairer = make_repairer();
        assert_eq!(repairer.confidence("broken"), 0.0);
    }

    #[test]
    fn test_needs_repair_valid() {
        let repairer = make_repairer();
        assert!(!repairer.needs_repair("ok"));
    }

    #[test]
    fn test_needs_repair_invalid() {
        let repairer = make_repairer();
        assert!(repairer.needs_repair("broken"));
    }

    #[test]
    fn test_repair_valid_passthrough() {
        let mut repairer = make_repairer();
        let result = repairer.repair("ok").unwrap();
        assert_eq!(result, "ok");
    }

    #[test]
    fn test_repair_empty_input() {
        let mut repairer = make_repairer();
        let result = repairer.repair("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_repair_with_explanations_valid() {
        let repairer = make_repairer();
        let (result, applied) = repairer.repair_with_explanations("ok").unwrap();
        assert_eq!(result, "ok");
        assert!(applied.is_empty());
    }

    #[test]
    fn test_repair_with_explanations_empty() {
        let repairer = make_repairer();
        let (result, applied) = repairer.repair_with_explanations("").unwrap();
        assert_eq!(result, "");
        assert!(applied.is_empty());
    }
}
