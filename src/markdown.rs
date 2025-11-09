//! Markdown repair module
//!
//! Provides comprehensive Markdown repair functionality with multiple strategies
//! for fixing common Markdown issues from LLM outputs.

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use regex::Regex;
use std::sync::OnceLock;

// ============================================================================
// Markdown Validator
// ============================================================================

/// Markdown validator
pub struct MarkdownValidator;

impl Validator for MarkdownValidator {
    fn is_valid(&self, content: &str) -> bool {
        // Basic markdown validation
        if content.is_empty() {
            return true;
        }
        
        // Check for balanced markers
        let bold_count = content.matches("**").count();
        let italic_count = content.matches('*').count();
        let code_fence_count = content.matches("```").count();
        
        // Bold should be balanced
        if bold_count % 2 != 0 {
            return false;
        }
        
        // Code fences should be balanced
        if code_fence_count % 2 != 0 {
            return false;
        }
        
        // Check for malformed headers (# without space)
        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                // Count leading #
                let hash_count = trimmed.chars().take_while(|c| *c == '#').count();
                if hash_count <= 6 {
                    // Check if there's a space after the hashes
                    if let Some(ch) = trimmed.chars().nth(hash_count) {
                        if ch != ' ' && ch != '\n' {
                            return false; // Malformed header
                        }
                    }
                }
            }
        }
        
        // Basic structure check
        let has_valid_structure = !content.contains("[[") && !content.contains("]]");
        
        has_valid_structure
    }
    
    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        if content.is_empty() {
            return errors;
        }
        
        // Check for unbalanced bold markers
        let bold_count = content.matches("**").count();
        if bold_count % 2 != 0 {
            errors.push("Unbalanced bold markers (**)".to_string());
        }
        
        // Check for unbalanced code fences
        let code_fence_count = content.matches("```").count();
        if code_fence_count % 2 != 0 {
            errors.push("Unbalanced code block fences (```)".to_string());
        }
        
        // Check for malformed links
        if content.contains("[[") || content.contains("]]") {
            errors.push("Malformed link syntax".to_string());
        }
        
        errors
    }
}

#[cfg(test)]
mod validator_tests {
    use super::*;

    #[test]
    fn test_valid_markdown() {
        let validator = MarkdownValidator;
        assert!(validator.is_valid("# Header\n\nSome content"));
    }

    #[test]
    fn test_invalid_markdown_unbalanced_bold() {
        let validator = MarkdownValidator;
        assert!(!validator.is_valid("**bold text"));
    }

    #[test]
    fn test_invalid_markdown_unbalanced_code() {
        let validator = MarkdownValidator;
        assert!(!validator.is_valid("```\ncode"));
    }

    #[test]
    fn test_validate_errors() {
        let validator = MarkdownValidator;
        let errors = validator.validate("**bold text");
        assert!(!errors.is_empty());
    }
}

// ============================================================================
// Regex Cache
// ============================================================================

/// Cached regex patterns for Markdown performance optimization
pub struct MarkdownRegexCache {
    pub header_spacing: Regex,
    pub code_block_fences: Regex,
    pub list_items: Regex,
    pub link_formatting: Regex,
    pub bold_italic: Regex,
}

impl MarkdownRegexCache {
    pub fn new() -> Result<Self> {
        Ok(Self {
            header_spacing: Regex::new(r#"(?m)^(#{1,6})([^#\s])"#)?,
            code_block_fences: Regex::new(r#"(?m)^```(\w+)?$"#)?,
            list_items: Regex::new(r#"(?m)^(\s*)(\d+\.)([^ ])"#)?,
            link_formatting: Regex::new(r#"\[([^\]]+)\]\(([^)]+)\)"#)?,
            bold_italic: Regex::new(r#"\*\*([^*]+)\*\*|\*([^*]+)\*"#)?,
        })
    }
}

static MARKDOWN_REGEX_CACHE: OnceLock<MarkdownRegexCache> = OnceLock::new();

pub fn get_markdown_regex_cache() -> &'static MarkdownRegexCache {
    MARKDOWN_REGEX_CACHE.get_or_init(|| MarkdownRegexCache::new().expect("Failed to initialize Markdown regex cache"))
}

// ============================================================================
// Repair Strategies
// ============================================================================

/// Strategy to fix header spacing
pub struct FixHeaderSpacingStrategy;

impl RepairStrategy for FixHeaderSpacingStrategy {
    fn name(&self) -> &str {
        "FixHeaderSpacing"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_markdown_regex_cache();
        Ok(cache.header_spacing.replace_all(content, "$1 $2").to_string())
    }
    
    fn priority(&self) -> u8 {
        100
    }
}

/// Strategy to fix code block fences
pub struct FixCodeBlockFencesStrategy;

impl RepairStrategy for FixCodeBlockFencesStrategy {
    fn name(&self) -> &str {
        "FixCodeBlockFences"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = String::new();
        let mut in_code_block = false;
        
        for line in lines {
            if line.trim().starts_with("```") {
                in_code_block = !in_code_block;
                result.push_str(line);
            } else {
                result.push_str(line);
            }
            result.push('\n');
        }
        
        Ok(result.trim_end().to_string())
    }
    
    fn priority(&self) -> u8 {
        90
    }
}

/// Strategy to fix list formatting
pub struct FixListFormattingStrategy;

impl RepairStrategy for FixListFormattingStrategy {
    fn name(&self) -> &str {
        "FixListFormatting"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_markdown_regex_cache();
        Ok(cache.list_items.replace_all(content, "$1$2 $3").to_string())
    }
    
    fn priority(&self) -> u8 {
        85
    }
}

/// Strategy to fix link formatting
pub struct FixLinkFormattingStrategy;

impl RepairStrategy for FixLinkFormattingStrategy {
    fn name(&self) -> &str {
        "FixLinkFormatting"
    }

    fn apply(&self, content: &str) -> Result<String> {
        // Validate and fix link syntax
        let mut result = content.to_string();
        
        // Fix common link issues
        result = result.replace("[ ", "[");
        result = result.replace(" ]", "]");
        result = result.replace("( ", "(");
        result = result.replace(" )", ")");
        
        Ok(result)
    }
    
    fn priority(&self) -> u8 {
        80
    }
}

/// Strategy to fix bold and italic formatting
pub struct FixBoldItalicStrategy;

impl RepairStrategy for FixBoldItalicStrategy {
    fn name(&self) -> &str {
        "FixBoldItalic"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();
        
        // Fix unmatched bold markers
        let bold_count = result.matches("**").count();
        if bold_count % 2 != 0 {
            result.push_str("**");
        }
        
        // Fix unmatched italic markers
        let italic_count = result.matches('*').count();
        if italic_count % 2 != 0 {
            result.push('*');
        }
        
        Ok(result)
    }
    
    fn priority(&self) -> u8 {
        75
    }
}

/// Strategy to add missing newlines
pub struct AddMissingNewlinesStrategy;

impl RepairStrategy for AddMissingNewlinesStrategy {
    fn name(&self) -> &str {
        "AddMissingNewlines"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = String::new();
        
        for (i, line) in lines.iter().enumerate() {
            result.push_str(line);
            
            // Add newline after headers and code blocks
            if line.trim().starts_with('#') || line.trim().starts_with("```") {
                if i < lines.len() - 1 && !lines[i + 1].is_empty() {
                    result.push('\n');
                }
            }
            
            result.push('\n');
        }
        
        Ok(result.trim_end().to_string())
    }
    
    fn priority(&self) -> u8 {
        70
    }
}

/// Strategy to fix table formatting
pub struct FixTableFormattingStrategy;

impl RepairStrategy for FixTableFormattingStrategy {
    fn name(&self) -> &str {
        "FixTableFormatting"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = String::new();
        
        for (i, line) in lines.iter().enumerate() {
            if line.contains('|') {
                // Ensure proper spacing around pipes
                let fixed = line
                    .replace("| ", "|")
                    .replace(" |", "|");
                let fixed = fixed.replace("|", " | ");
                result.push_str(&fixed);
            } else {
                result.push_str(line);
            }
            result.push('\n');
        }
        
        Ok(result.trim_end().to_string())
    }
    
    fn priority(&self) -> u8 {
        65
    }
}

/// Strategy to fix nested lists
pub struct FixNestedListsStrategy;

impl RepairStrategy for FixNestedListsStrategy {
    fn name(&self) -> &str {
        "FixNestedLists"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = String::new();
        
        for line in lines {
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();
            
            // Fix list item formatting
            if trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with('+') {
                let marker = trimmed.chars().next().unwrap();
                let content_part = trimmed.trim_start_matches(|c| c == marker || c == ' ');
                result.push_str(&format!("{}{} {}", " ".repeat(indent), marker, content_part));
            } else {
                result.push_str(line);
            }
            result.push('\n');
        }
        
        Ok(result.trim_end().to_string())
    }
    
    fn priority(&self) -> u8 {
        60
    }
}

/// Strategy to fix image syntax
pub struct FixImageSyntaxStrategy;

impl RepairStrategy for FixImageSyntaxStrategy {
    fn name(&self) -> &str {
        "FixImageSyntax"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();
        
        // Fix common image syntax issues
        result = result.replace("![ ", "![");
        result = result.replace(" ]", "]");
        result = result.replace("( ", "(");
        result = result.replace(" )", ")");
        
        Ok(result)
    }
    
    fn priority(&self) -> u8 {
        55
    }
}

// ============================================================================
// Markdown Repairer
// ============================================================================

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

