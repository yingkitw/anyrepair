//! Markdown repair module
//!
//! Provides comprehensive Markdown repair functionality with multiple strategies
//! for fixing common Markdown issues from LLM outputs.

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};

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
        let _italic_count = content.matches('*').count();
        let code_fence_count = content.matches("```").count();

        // Bold should be balanced
        if !bold_count.is_multiple_of(2) {
            return false;
        }

        // Code fences should be balanced
        if !code_fence_count.is_multiple_of(2) {
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
                    if let Some(ch) = trimmed.chars().nth(hash_count)
                        && ch != ' ' && ch != '\n' {
                            return false; // Malformed header
                        }
                }
            }
        }

        // Basic structure check
        

        !content.contains("[[") && !content.contains("]]")
    }

    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();

        if content.is_empty() {
            return errors;
        }

        // Check for unbalanced bold markers
        let bold_count = content.matches("**").count();
        if !bold_count.is_multiple_of(2) {
            errors.push("Unbalanced bold markers (**)".to_string());
        }

        // Check for unbalanced code fences
        let code_fence_count = content.matches("```").count();
        if !code_fence_count.is_multiple_of(2) {
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
// Pattern Helpers
// ============================================================================

/// Fix missing space after Markdown header hashes.
///
/// Mirrors the regex `(?m)^(#{1,6})([^#\s])` → `$1 $2`: at each line start,
/// 1 to 6 leading `#` followed by a non-`#`-non-whitespace char get a space
/// inserted between the hashes and that char.
fn fix_header_spacing(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;
    while i < n {
        let is_line_start = i == 0 || chars[i - 1] == '\n';
        if is_line_start && chars[i] == '#' {
            // Count leading '#' (capped at 6, matching {1,6} greedy).
            let leading = chars[i..].iter().take_while(|c| **c == '#').count();
            let max_hash = leading.min(6);
            // Greedy: try the largest hash count <= 6 whose following char
            // is a non-'#'-non-whitespace char.
            let mut matched = None;
            for k in (1..=max_hash).rev() {
                if let Some(&c) = chars.get(i + k)
                    && c != '#' && !c.is_whitespace() {
                        matched = Some((k, c));
                        break;
                    }
            }
            if let Some((k, c)) = matched {
                for _ in 0..k {
                    out.push('#');
                }
                out.push(' ');
                out.push(c);
                i += k + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Fix missing space after a numbered list marker.
///
/// Mirrors the regex `(?m)^(\s*)(\d+\.)([^ ])` → `$1$2 $3`: at each line
/// start, optional whitespace, then digits + `.`, then a non-space char get
/// a space inserted between the marker and that char.
fn fix_list_items(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;
    while i < n {
        let is_line_start = i == 0 || chars[i - 1] == '\n';
        if is_line_start
            && let Some((indent, marker, ch, end)) = match_list_item(&chars, i) {
                out.push_str(&indent);
                out.push_str(&marker);
                out.push(' ');
                out.push(ch);
                i = end;
                continue;
            }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Match `(\s*)(\d+\.)([^ ])` starting at `i`. Returns (indent, marker, ch, end).
fn match_list_item(chars: &[char], i: usize) -> Option<(String, String, char, usize)> {
    let n = chars.len();
    let mut j = i;
    // \s*
    let indent_start = j;
    while j < n && chars[j].is_whitespace() {
        j += 1;
    }
    let indent: String = chars[indent_start..j].iter().collect();
    // \d+
    let digits_start = j;
    while j < n && chars[j].is_ascii_digit() {
        j += 1;
    }
    if j == digits_start {
        return None;
    }
    // \.
    if j >= n || chars[j] != '.' {
        return None;
    }
    let marker: String = chars[digits_start..=j].iter().collect();
    j += 1;
    // [^ ] (any char except a single space, including newline)
    if j >= n {
        return None;
    }
    let ch = chars[j];
    if ch == ' ' {
        return None;
    }
    Some((indent, marker, ch, j + 1))
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
        Ok(fix_header_spacing(content))
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
        Ok(fix_list_items(content))
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
        if !bold_count.is_multiple_of(2) {
            result.push_str("**");
        }

        // Fix unmatched italic markers
        let italic_count = result.matches('*').count();
        if !italic_count.is_multiple_of(2) {
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
            if (line.trim().starts_with('#') || line.trim().starts_with("```"))
                && i < lines.len() - 1 && !lines[i + 1].is_empty() {
                    result.push('\n');
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

        for line in lines.iter() {
            if line.contains('|') {
                // Ensure proper spacing around pipes
                let fixed = line.replace("| ", "|").replace(" |", "|");
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
                let marker = trimmed.as_bytes()[0] as char;
                let content_part = trimmed.trim_start_matches([marker, ' ']);
                result.push_str(&format!(
                    "{}{} {}",
                    " ".repeat(indent),
                    marker,
                    content_part
                ));
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
///
/// Uses trait-based composition with GenericRepairer for better modularity
pub struct MarkdownRepairer {
    pub inner: crate::repairer_base::GenericRepairer,
}

impl MarkdownRepairer {
    /// Create a new Markdown repairer
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
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

        let validator: Box<dyn Validator> = Box::new(MarkdownValidator);
        let inner = crate::repairer_base::GenericRepairer::new(validator, strategies);

        Self { inner }
    }
}

impl Default for MarkdownRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for MarkdownRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        self.inner.repair(content)
    }

    fn needs_repair(&self, content: &str) -> bool {
        self.inner.needs_repair(content)
    }

    fn confidence(&self, content: &str) -> f64 {
        if self.inner.validator().is_valid(content) {
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
        assert!(!repairer.inner.strategies().is_empty());
    }

    #[test]
    fn test_markdown_repairer_default() {
        let repairer = MarkdownRepairer::default();
        assert!(!repairer.inner.strategies().is_empty());
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
