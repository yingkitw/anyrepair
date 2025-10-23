//! # AnyRepair
//! 
//! A Rust crate for repairing LLM responses including JSON, YAML, and Markdown.
//! 
//! This crate provides robust repair mechanisms for common issues found in LLM-generated content,
//! such as malformed JSON, incomplete YAML, and broken Markdown formatting.

pub mod error;
pub mod json;
pub mod yaml;
pub mod markdown;
pub mod traits;

pub use error::{RepairError, Result};
pub use traits::Repair;

/// Main repair function that automatically detects format and repairs content
pub fn repair(content: &str) -> Result<String> {
    let trimmed = content.trim();
    
    // Try to detect format and repair accordingly
    if is_json_like(trimmed) {
        json::JsonRepairer::new().repair(trimmed)
    } else if is_yaml_like(trimmed) {
        yaml::YamlRepairer::new().repair(trimmed)
    } else if is_markdown_like(trimmed) {
        markdown::MarkdownRepairer::new().repair(trimmed)
    } else {
        // Default to markdown repair for unknown content
        markdown::MarkdownRepairer::new().repair(trimmed)
    }
}

fn is_json_like(content: &str) -> bool {
    let trimmed = content.trim();
    (trimmed.starts_with('{') && (trimmed.ends_with('}') || trimmed.contains(':'))) ||
    (trimmed.starts_with('[') && (trimmed.ends_with(']') || trimmed.contains(',')))
}

fn is_yaml_like(content: &str) -> bool {
    let trimmed = content.trim();
    trimmed.contains("---") || 
    (trimmed.contains(":") && !trimmed.starts_with('{') && !trimmed.starts_with('[')) ||
    trimmed.lines().any(|line| line.contains(":") && !line.trim().starts_with('"') && !line.trim().starts_with('{'))
}

fn is_markdown_like(content: &str) -> bool {
    let trimmed = content.trim();
    trimmed.starts_with('#') ||
    trimmed.contains("```") ||
    trimmed.contains("**") ||
    trimmed.contains("*") ||
    trimmed.contains("`")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert!(is_json_like(r#"{"key": "value"}"#));
        assert!(is_json_like(r#"[1, 2, 3]"#));
        assert!(!is_json_like("# Header\nContent"));
        
        assert!(is_yaml_like("key: value"));
        assert!(is_yaml_like("---\nkey: value"));
        assert!(!is_yaml_like(r#"{"key": "value"}"#));
        
        assert!(is_markdown_like("# Header"));
        assert!(is_markdown_like("**bold**"));
        assert!(is_markdown_like("```code```"));
        assert!(!is_markdown_like(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_repair_function() {
        // Test JSON repair
        let json_input = r#"{"name": "John", "age": 30,}"#;
        let result = repair(json_input).unwrap();
        assert!(result.contains("John"));
        assert!(!result.ends_with(','));

        // Test YAML repair
        let yaml_input = "name: John\nage: 30";
        let result = repair(yaml_input).unwrap();
        assert!(result.contains("name: John"));

        // Test Markdown repair
        let markdown_input = "#Header\nSome **bold** text";
        let result = repair(markdown_input).unwrap();
        assert!(result.contains("Header"));
    }

    #[test]
    fn test_format_detection_edge_cases() {
        // Test empty content
        assert!(!is_json_like(""));
        assert!(!is_yaml_like(""));
        assert!(!is_markdown_like(""));

        // Test whitespace only
        assert!(!is_json_like("   \n\t  "));
        assert!(!is_yaml_like("   \n\t  "));
        assert!(!is_markdown_like("   \n\t  "));

        // Test mixed content
        assert!(is_json_like(r#"{"key": "value", "nested": {"inner": "value"}}"#));
        assert!(is_yaml_like("key: value\nnested:\n  inner: value"));
        assert!(is_markdown_like("# Header\n\nSome **bold** text with `code`"));

        // Test malformed content
        assert!(is_json_like(r#"{"key": "value""#)); // Missing closing brace
        assert!(is_yaml_like("key: value\n  invalid: indentation"));
        assert!(is_markdown_like("#Header\n**bold")); // Missing closing bold
    }

    #[test]
    fn test_repair_error_handling() {
        // Test with empty input
        let result = repair("");
        assert!(result.is_ok());

        // Test with very long input
        let long_input = "a".repeat(10000);
        let result = repair(&long_input);
        assert!(result.is_ok());
    }
}
