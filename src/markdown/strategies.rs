//! Markdown repair strategies
//! Contains all strategy implementations for Markdown repair

use crate::error::Result;
use crate::traits::RepairStrategy;
use regex::Regex;
use std::sync::OnceLock;

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
