//! Markdown repair module
//!
//! Provides comprehensive Markdown repair functionality with multiple strategies
//! for fixing common Markdown issues from LLM outputs.

pub mod strategies;
pub mod validator;

pub use strategies::{
    FixHeaderSpacingStrategy, FixCodeBlockFencesStrategy, FixListFormattingStrategy,
    FixLinkFormattingStrategy, FixBoldItalicStrategy, AddMissingNewlinesStrategy,
    FixTableFormattingStrategy, FixNestedListsStrategy, FixImageSyntaxStrategy,
    get_markdown_regex_cache,
};
pub use validator::MarkdownValidator;

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};

/// Markdown repairer that can fix common Markdown issues
pub struct MarkdownRepairer {
    strategies: Vec<Box<dyn RepairStrategy>>,
    validator: MarkdownValidator,
}

impl MarkdownRepairer {
    /// Create a new Markdown repairer
    pub fn new() -> Self {
        let mut strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(FixHeaderSpacingStrategy),
            Box::new(FixCodeBlockFencesStrategy),
            Box::new(FixListFormattingStrategy),
            Box::new(FixLinkFormattingStrategy),
            Box::new(FixBoldItalicStrategy),
            Box::new(AddMissingNewlinesStrategy),
            Box::new(FixTableFormattingStrategy),
            Box::new(FixNestedListsStrategy),
            Box::new(FixImageSyntaxStrategy),
        ];
        
        // Sort strategies by priority (higher priority first)
        strategies.sort_by_key(|b| std::cmp::Reverse(b.priority()));
        
        Self {
            strategies,
            validator: MarkdownValidator,
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

impl Default for MarkdownRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for MarkdownRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        let trimmed = content.trim();
        
        // If already valid, return as-is
        if self.validator.is_valid(trimmed) {
            return Ok(trimmed.to_string());
        }
        
        // Apply repair strategies
        let repaired = self.apply_strategies(trimmed)?;
        
        // Return the repaired content
        Ok(repaired)
    }
    
    fn needs_repair(&self, content: &str) -> bool {
        !self.validator.is_valid(content)
    }
    
    fn confidence(&self, content: &str) -> f64 {
        if self.validator.is_valid(content) {
            return 1.0;
        }
        
        let mut score: f64 = 0.0;
        
        // Check for markdown structure
        if content.contains('#') {
            score += 0.2;
        }
        
        if content.contains("```") {
            score += 0.2;
        }
        
        if content.contains('[') && content.contains(']') {
            score += 0.2;
        }
        
        if content.contains('*') || content.contains('_') {
            score += 0.2;
        }
        
        if content.contains('-') || content.contains('+') {
            score += 0.1;
        }
        
        score.min(1.0_f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_repairer_creation() {
        let repairer = MarkdownRepairer::new();
        assert!(!repairer.strategies.is_empty());
    }

    #[test]
    fn test_markdown_repairer_default() {
        let repairer = MarkdownRepairer::default();
        assert!(!repairer.strategies.is_empty());
    }

    #[test]
    fn test_markdown_confidence_valid() {
        let repairer = MarkdownRepairer::new();
        let confidence = repairer.confidence("# Header\n\nContent");
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_markdown_confidence_invalid() {
        let repairer = MarkdownRepairer::new();
        let confidence = repairer.confidence("**bold text");
        assert!(confidence < 1.0);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_markdown_needs_repair() {
        let repairer = MarkdownRepairer::new();
        assert!(!repairer.needs_repair("# Header\n\nContent"));
        assert!(repairer.needs_repair("**bold text"));
    }
}
