//! Markdown repair functionality

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use regex::Regex;
use std::sync::OnceLock;

/// Cached regex patterns for Markdown performance optimization
#[allow(dead_code)]
struct MarkdownRegexCache {
    header_spacing: Regex,
    code_block_fences: Regex,
    list_items: Regex,
    link_formatting: Regex,
    bold_italic: Regex,
}

impl MarkdownRegexCache {
    fn new() -> Result<Self> {
        Ok(Self {
            header_spacing: Regex::new(r#"^(#{1,6})([^#\s])"#)?,
            code_block_fences: Regex::new(r#"^```(\w+)?$"#)?,
            list_items: Regex::new(r#"^(\s*)(\d+\.)([^ ])"#)?,
            link_formatting: Regex::new(r#"\[([^\]]+)\]\(([^)]+)\)"#)?,
            bold_italic: Regex::new(r#"\*\*([^*]+)\*\*|\*([^*]+)\*"#)?,
        })
    }
}

static MARKDOWN_REGEX_CACHE: OnceLock<MarkdownRegexCache> = OnceLock::new();

fn get_markdown_regex_cache() -> &'static MarkdownRegexCache {
    MARKDOWN_REGEX_CACHE.get_or_init(|| MarkdownRegexCache::new().expect("Failed to initialize Markdown regex cache"))
}

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
        
        // Return the repaired content even if validation fails
        // (some repairs might not be perfect but are still improvements)
        Ok(repaired)
    }
    
    fn needs_repair(&self, content: &str) -> bool {
        !self.validator.is_valid(content)
    }
    
    fn confidence(&self, content: &str) -> f64 {
        if self.validator.is_valid(content) {
            return 1.0;
        }
        
        // Calculate confidence based on Markdown-like patterns
        let mut score: f64 = 0.0;
        
        // Check for headers
        if content.contains('#') {
            score += 0.2;
        }
        
        // Check for code blocks
        if content.contains("```") {
            score += 0.2;
        }
        
        // Check for bold/italic
        if content.contains("**") || content.contains("*") {
            score += 0.15;
        }
        
        // Check for links
        if content.contains("[") && content.contains("]") && content.contains("(") {
            score += 0.15;
        }
        
        // Check for lists
        if content.contains("- ") || content.contains("* ") || content.contains("1. ") {
            score += 0.15;
        }
        
        // Check for proper line breaks
        if content.contains("\n\n") {
            score += 0.1;
        }
        
        // Check for inline code
        if content.contains("`") {
            score += 0.05;
        }
        
        score.min(1.0_f64)
    }
}

/// Markdown validator
pub struct MarkdownValidator;

impl Validator for MarkdownValidator {
    fn is_valid(&self, content: &str) -> bool {
        // Check if content has any markdown-like features
        let has_markdown_features = content.contains('#') || 
                                   content.contains("```") || 
                                   content.contains("**") || 
                                   content.contains("*") || 
                                   content.contains("`") ||
                                   content.contains("[") && content.contains("]") && content.contains("(");
        
        if !has_markdown_features {
            return false;
        }
        
        // Check for malformed headers (hash without space)
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with('#') && !trimmed.starts_with("# ") {
                return false;
            }
        }
        
        // Additional checks for common Markdown issues
        self.validate(content).is_empty()
    }
    
    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        // Check for unmatched code block fences
        let code_fence_count = content.matches("```").count();
        if code_fence_count % 2 != 0 {
            errors.push("Unmatched code block fences".to_string());
        }
        
        // Check for unmatched bold/italic markers
        let bold_count = content.matches("**").count();
        if bold_count % 2 != 0 {
            errors.push("Unmatched bold markers (**)".to_string());
        }
        
        let italic_count = content.matches("*").count();
        if italic_count % 2 != 0 {
            errors.push("Unmatched italic markers (*)".to_string());
        }
        
        // Check for malformed links
        let link_re = Regex::new(r"\[([^\]]*)\]\(([^)]*)\)").unwrap();
        for cap in link_re.captures_iter(content) {
            let text = &cap[1];
            let url = &cap[2];
            if text.is_empty() || url.is_empty() {
                errors.push("Empty link text or URL".to_string());
            }
        }
        
        errors
    }
}

/// Strategy to fix header spacing
struct FixHeaderSpacingStrategy;

impl RepairStrategy for FixHeaderSpacingStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_markdown_regex_cache();
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        
        for line in lines {
            if cache.header_spacing.is_match(line) {
                let fixed = cache.header_spacing.replace(line, "$1 $2");
                result.push(fixed.to_string());
            } else {
                result.push(line.to_string());
            }
        }
        
        Ok(result.join("\n"))
    }
    
    fn priority(&self) -> u8 {
        6
    }

    fn name(&self) -> &str {
        "FixHeaderSpacingStrategy"
    }
}

/// Strategy to fix code block fences
struct FixCodeBlockFencesStrategy;

impl RepairStrategy for FixCodeBlockFencesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut in_code_block = false;
        let mut _code_block_count = 0;
        
        for line in lines {
            if line.trim().starts_with("```") {
                _code_block_count += 1;
                in_code_block = !in_code_block;
                
                // Ensure proper fence format
                if line.trim() == "```" {
                    result.push("```".to_string());
                } else {
                    let lang = line.trim().strip_prefix("```").unwrap_or("").trim();
                    result.push(format!("```{}", lang));
                }
            } else if in_code_block {
                result.push(line.to_string());
            } else {
                result.push(line.to_string());
            }
        }
        
        // If we ended in a code block, close it
        if in_code_block {
            result.push("```".to_string());
        }
        
        Ok(result.join("\n"))
    }
    
    fn priority(&self) -> u8 {
        5
    }

    fn name(&self) -> &str {
        "FixCodeBlockFencesStrategy"
    }
}

/// Strategy to fix list formatting
struct FixListFormattingStrategy;

impl RepairStrategy for FixListFormattingStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_markdown_regex_cache();
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("-") && !trimmed.starts_with("- ") {
                let fixed = format!("- {}", trimmed[1..].trim());
                result.push(fixed);
            } else if trimmed.starts_with("*") && !trimmed.starts_with("* ") {
                let fixed = format!("* {}", trimmed[1..].trim());
                result.push(fixed);
            } else if cache.list_items.is_match(line) {
                let fixed = cache.list_items.replace(line, "$1$2 $3");
                result.push(fixed.to_string());
            } else {
                result.push(line.to_string());
            }
        }
        
        Ok(result.join("\n"))
    }
    
    fn priority(&self) -> u8 {
        4
    }

    fn name(&self) -> &str {
        "FixListFormattingStrategy"
    }
}

/// Strategy to fix link formatting
struct FixLinkFormattingStrategy;

impl RepairStrategy for FixLinkFormattingStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        // Fix malformed links
        let re = Regex::new(r"\[([^\]]*)\]\(([^)]*)\)")?;
        let result = re.replace_all(content, |caps: &regex::Captures| {
            let text = caps.get(1).unwrap().as_str();
            let url = caps.get(2).unwrap().as_str();
            if text.is_empty() || url.is_empty() {
                format!("[{}]({})", text, url)
            } else {
                format!("[{}]({})", text, url)
            }
        });
        
        Ok(result.to_string())
    }
    
    fn priority(&self) -> u8 {
        3
    }

    fn name(&self) -> &str {
        "FixLinkFormattingStrategy"
    }
}

/// Strategy to fix bold/italic formatting
struct FixBoldItalicStrategy;

impl RepairStrategy for FixBoldItalicStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_markdown_regex_cache();
        
        // Fix unmatched bold and italic markers
        let result = cache.bold_italic.replace_all(content, |caps: &regex::Captures| {
            if let Some(bold) = caps.get(1) {
                format!("**{}**", bold.as_str())
            } else if let Some(italic) = caps.get(2) {
                format!("*{}*", italic.as_str())
            } else {
                caps[0].to_string()
            }
        });
        
        Ok(result.to_string())
    }
    
    fn priority(&self) -> u8 {
        2
    }

    fn name(&self) -> &str {
        "FixBoldItalicStrategy"
    }
}

/// Strategy to add missing newlines
struct AddMissingNewlinesStrategy;

impl RepairStrategy for AddMissingNewlinesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            result.push(line.to_string());
            
            // Add newline after headers
            if line.trim().starts_with('#') && i < lines.len() - 1 {
                if !lines[i + 1].trim().is_empty() {
                    result.push("".to_string());
                }
            }
            
            // Add newline after code blocks
            if line.trim().starts_with("```") && i < lines.len() - 1 {
                if !lines[i + 1].trim().is_empty() {
                    result.push("".to_string());
                }
            }
        }
        
        Ok(result.join("\n"))
    }
    
    fn priority(&self) -> u8 {
        1
    }

    fn name(&self) -> &str {
        "AddMissingNewlinesStrategy"
    }
}

/// Strategy to fix table formatting
struct FixTableFormattingStrategy;

impl RepairStrategy for FixTableFormattingStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result: Vec<String> = Vec::new();
        let mut in_table = false;
        let mut table_lines: Vec<String> = Vec::new();
        
        for line in lines {
            let trimmed = line.trim();
            
            // Detect table rows (contain |)
            if trimmed.contains('|') && !trimmed.starts_with('|') && !trimmed.ends_with('|') {
                if !in_table {
                    in_table = true;
                    // Add empty line before table
                    if !result.is_empty() && !result.last().unwrap().trim().is_empty() {
                        result.push(String::new());
                    }
                }
                
                // Fix table row formatting
                let fixed = format!("|{}|", trimmed);
                table_lines.push(fixed);
            } else if trimmed.starts_with('|') && trimmed.ends_with('|') {
                // Already formatted table row
                if !in_table {
                    in_table = true;
                    if !result.is_empty() && !result.last().unwrap().trim().is_empty() {
                        result.push(String::new());
                    }
                }
                table_lines.push(line.to_string());
            } else if trimmed.starts_with("|---") {
                // Table separator
                if in_table {
                    table_lines.push(line.to_string());
                } else {
                    result.push(line.to_string());
                }
            } else {
                // End of table
                if in_table {
                    // Add table to result
                    result.extend(table_lines.clone());
                    result.push(String::new()); // Empty line after table
                    table_lines.clear();
                    in_table = false;
                }
                result.push(line.to_string());
            }
        }
        
        // Add any remaining table lines
        if in_table {
            result.extend(table_lines.clone());
        }
        
        Ok(result.join("\n"))
    }
    
    fn priority(&self) -> u8 {
        3
    }

    fn name(&self) -> &str {
        "FixTableFormattingStrategy"
    }
}

/// Strategy to fix nested lists
struct FixNestedListsStrategy;

impl RepairStrategy for FixNestedListsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result: Vec<String> = Vec::new();
        let mut list_stack: Vec<char> = Vec::new();
        
        for line in lines {
            if line.trim().is_empty() {
                result.push(line.to_string());
                continue;
            }
            
            let trimmed = line.trim();
            let indent = line.chars().take_while(|c| c.is_whitespace()).count();
            
            if trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with('+') {
                // List item
                let list_char = trimmed.chars().next().unwrap();
                let content = trimmed[1..].trim();
                
                // Determine nesting level
                let expected_indent = list_stack.len() * 2;
                let actual_indent = indent;
                
                if actual_indent < expected_indent {
                    // Reduce nesting
                    while list_stack.len() > actual_indent / 2 {
                        list_stack.pop();
                    }
                } else if actual_indent > expected_indent {
                    // Increase nesting
                    while list_stack.len() < actual_indent / 2 {
                        list_stack.push(list_char);
                    }
                }
                
                let fixed_indent = list_stack.len() * 2;
                let fixed = format!("{}{} {}", " ".repeat(fixed_indent), list_char, content);
                result.push(fixed);
            } else if trimmed.starts_with("1.") || trimmed.starts_with("2.") || trimmed.starts_with("3.") {
                // Numbered list item
                let parts: Vec<&str> = trimmed.splitn(2, '.').collect();
                if parts.len() == 2 {
                    let number = parts[0].trim();
                    let content = parts[1].trim();
                    let fixed_indent = list_stack.len() * 2;
                    let fixed = format!("{}{}. {}", " ".repeat(fixed_indent), number, content);
                    result.push(fixed);
                } else {
                    result.push(line.to_string());
                }
            } else {
                // Non-list content - reset list stack
                list_stack.clear();
                result.push(line.to_string());
            }
        }
        
        Ok(result.join("\n"))
    }
    
    fn priority(&self) -> u8 {
        4
    }

    fn name(&self) -> &str {
        "FixNestedListsStrategy"
    }
}

/// Strategy to fix image syntax
struct FixImageSyntaxStrategy;

impl RepairStrategy for FixImageSyntaxStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();
        
        // Fix image syntax: ![alt](url)
        let image_re = Regex::new(r"!\[([^\]]*)\]\(([^)]*)\)")?;
        result = image_re.replace_all(&result, |caps: &regex::Captures| {
            let alt = caps.get(1).map_or("", |m| m.as_str());
            let url = caps.get(2).map_or("", |m| m.as_str());
            format!("![{}]({})", alt, url)
        }).to_string();
        
        Ok(result)
    }
    
    fn priority(&self) -> u8 {
        2
    }

    fn name(&self) -> &str {
        "FixImageSyntaxStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_markdown_repair_headers() {
        let mut repairer = MarkdownRepairer::new();
        
        let input = "#Header\n##Subheader";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        # Header

        ## Subheader
        ");
    }
    
    #[test]
    fn test_markdown_repair_code_blocks() {
        let mut repairer = MarkdownRepairer::new();
        
        let input = "```rust\nfn main() {\n    println!(\"Hello\");\n```";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        ```rust
        fn main() {
            println!("Hello");
        ```
        "#);
    }
    
    #[test]
    fn test_markdown_repair_lists() {
        let mut repairer = MarkdownRepairer::new();
        
        let input = "-item1\n-item2\n1.item3";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        - item1
        - item2
        1. item3
        ");
    }
    
    #[test]
    fn test_markdown_confidence() {
        let mut repairer = MarkdownRepairer::new();
        
        // Valid Markdown should have confidence 1.0
        let valid = "# Header\n\nSome **bold** text";
        assert_eq!(repairer.confidence(valid), 1.0);
        
        // Invalid Markdown should have lower confidence
        let invalid = "not markdown at all";
        let conf = repairer.confidence(invalid);
        println!("Confidence for 'not markdown at all': {}", conf);
        assert!(conf < 1.0);
    }
    
    #[test]
    fn test_needs_repair() {
        let mut repairer = MarkdownRepairer::new();
        
        assert!(!repairer.needs_repair("# Header\n\nSome text"));
        assert!(repairer.needs_repair("not markdown at all"));
    }

    #[test]
    fn test_markdown_repair_edge_cases() {
        let mut repairer = MarkdownRepairer::new();
        
        // Test empty markdown
        let input = "";
        let result = repairer.repair(input).unwrap();
        assert_eq!(result, "");
        
        // Test single line
        let input = "Just plain text";
        let result = repairer.repair(input).unwrap();
        assert_eq!(result, "Just plain text");
        
        // Test only headers
        let input = "#Header\n##Subheader";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        # Header

        ## Subheader
        ");
    }

    #[test]
    fn test_markdown_repair_complex_structures() {
        let mut repairer = MarkdownRepairer::new();
        
        // Test complex document
        let input = "#Title\n\nSome **bold** and *italic* text.\n\n##Subsection\n\n- item1\n- item2\n\n```code\nblock\n```";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        # Title

        Some **bold** and *italic* text.

        ## Subsection

        - item1
        - item2

        ```code

        block
        ```
        ");
        
        // Test tables
        let input = "|Header1|Header2|\n|-------|-------|\n|Cell1|Cell2|";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r###"
        |Header1|Header2|
        |-------|-------|
        |Cell1|Cell2|
        "###);
        
        // Test links and images
        let input = "[Link](https://example.com)\n![Image](image.png)";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r###"
        [Link](https://example.com)
        ![Image](image.png)
        "###);
    }

    #[test]
    fn test_markdown_repair_code_blocks_advanced() {
        let mut repairer = MarkdownRepairer::new();
        
        // Test unclosed code block
        let input = "```rust\nfn main() {\n    println!(\"Hello\");\n";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r#"
        ```rust

        fn main() {
            println!("Hello");
        ```
        "#);
        
        // Test code block with language
        let input = "```python\ndef hello():\n    print('Hello')\n```";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r###"
        ```python
        def hello():
            print('Hello')
        ```
        "###);
        
        // Test inline code
        let input = "Use `console.log()` to print";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r###"
        Use `console.log()` to print
        "###);
    }

    #[test]
    fn test_markdown_repair_lists_advanced() {
        let mut repairer = MarkdownRepairer::new();
        
        // Test unordered lists
        let input = "-item1\n-item2\n-item3";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r###"
        - item1
        - item2
        - item3
        "###);
        
        // Test ordered lists
        let input = "1.item1\n2.item2\n3.item3";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        1. item1
        2. item2
        3. item3
        ");
        
        // Test mixed lists
        let input = "-item1\n1.item2\n-item3";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        - item1
        1. item2
        - item3
        ");
        
        // Test nested lists
        let input = "-item1\n  -nested1\n  -nested2\n-item2";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        - item1
        - nested1
        - nested2
        - item2
        ");
    }

    #[test]
    fn test_markdown_repair_formatting() {
        let mut repairer = MarkdownRepairer::new();
        
        // Test bold and italic
        let input = "**bold** and *italic* and ***both***";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r###"
        **bold** and *italic* and ***both***
        "###);
        
        // Test strikethrough
        let input = "~~strikethrough~~ text";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r###"
        ~~strikethrough~~ text
        "###);
        
        // Test blockquotes
        let input = "> This is a quote\n> Multiple lines";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r###"
        > This is a quote
        > Multiple lines
        "###);
    }

    #[test]
    fn test_markdown_repair_malformed_cases() {
        let mut repairer = MarkdownRepairer::new();
        
        // Test unmatched bold
        let input = "**bold text without closing";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @"* *bold text without closing");
        
        // Test malformed links
        let input = "[Link]()\n[Empty text](https://example.com)";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r###"
        [Link]()
        [Empty text](https://example.com)
        "###);
        
        // Test malformed headers
        let input = "##Header without space\n###Another header";
        let result = repairer.repair(input).unwrap();
        assert_snapshot!(result, @r"
        ## Header without space

        ### Another header
        ");
    }

    #[test]
    fn test_markdown_confidence_edge_cases() {
        let mut repairer = MarkdownRepairer::new();
        
        // Test empty string
        assert_eq!(repairer.confidence(""), 0.0);
        
        // Test whitespace only
        assert_eq!(repairer.confidence("   \n\t  "), 0.0);
        
        // Test partial markdown
        let partial = "# Header\nSome text";
        let conf = repairer.confidence(partial);
        assert!(conf > 0.0);
        
        // Test non-markdown text
        let text = "This is just plain text";
        let conf = repairer.confidence(text);
        assert_eq!(conf, 0.0);
    }

    #[test]
    fn test_markdown_validator() {
        let validator = MarkdownValidator;
        
        // Test valid markdown
        assert!(validator.is_valid("# Header\n\nSome **bold** text"));
        assert!(validator.is_valid("```code\nblock\n```"));
        assert!(validator.is_valid("[Link](https://example.com)"));
        
        // Test invalid markdown
        assert!(!validator.is_valid("not markdown at all"));
        assert!(!validator.is_valid("```unclosed code block"));
        assert!(!validator.is_valid("**unmatched bold"));
        
        // Test validation errors
        let errors = validator.validate("```unclosed code block");
        assert!(!errors.is_empty());
        assert!(errors[0].contains("Unmatched code block fences"));
    }

    #[test]
    fn test_markdown_strategies_individual() {
        // Test FixHeaderSpacingStrategy
        let strategy = FixHeaderSpacingStrategy;
        let input = "#Header\n##Subheader";
        let result = strategy.apply(input).unwrap();
        assert_eq!(result, "# Header\n## Subheader");
        
        // Test FixListFormattingStrategy
        let strategy = FixListFormattingStrategy;
        let input = "-item1\n-item2\n1.item3";
        let result = strategy.apply(input).unwrap();
        assert_eq!(result, "- item1\n- item2\n1. item3");
        
        // Test FixBoldItalicStrategy
        let strategy = FixBoldItalicStrategy;
        let input = "**bold** and *italic*";
        let result = strategy.apply(input).unwrap();
        assert_eq!(result, "**bold** and *italic*");
        
        // Test AddMissingNewlinesStrategy
        let strategy = AddMissingNewlinesStrategy;
        let input = "#Header\nSome text\n```code\nblock\n```";
        let result = strategy.apply(input).unwrap();
        assert_snapshot!(result, @r"
        #Header

        Some text
        ```code

        block
        ```
        ");
    }
}
