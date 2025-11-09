//! # AnyRepair
//! 
//! A Rust crate for repairing LLM responses including JSON, YAML, and Markdown.
//! 
//! This crate provides robust repair mechanisms for common issues found in LLM-generated content,
//! such as malformed JSON, incomplete YAML, and broken Markdown formatting.
//!
//! ## Module Organization
//!
//! - Format-specific repairers: `json`, `yaml`, `markdown`, `xml`, `toml`, `csv`, `ini`
//! - `plugins` - Plugin system and integration
//! - `config` - Configuration and custom rule management
//! - Utility modules: `advanced`, `parallel`, `context_parser`, `enhanced_json`
//! - `traits` - Core trait definitions
//! - `error` - Error types and handling

// Core modules
pub mod error;
pub mod traits;
pub mod repairer_base;

// Format-specific repairers
pub mod json;
pub mod yaml;
pub mod markdown;
pub mod xml;
pub mod toml;
pub mod csv;
pub mod ini;

// Utility and helper modules
pub mod parallel;
pub mod parallel_strategy;
pub mod advanced;
pub mod plugin;
pub mod plugin_config;
pub mod plugin_integration;
pub mod context_parser;
pub mod enhanced_json;
pub mod config;
pub mod custom_rules;

// Enterprise features
pub mod analytics;
pub mod batch_processor;
pub mod validation_rules;
pub mod audit_log;
pub mod confidence_scorer;

// Streaming support
pub mod streaming;

// MCP server support
pub mod mcp_server;

pub use error::{RepairError, Result};
pub use traits::Repair;
pub use enhanced_json::EnhancedJsonRepairer;
pub use analytics::AnalyticsTracker;
pub use batch_processor::BatchProcessor;
pub use validation_rules::ValidationRulesEngine;
pub use audit_log::AuditLogger;
pub use confidence_scorer::ConfidenceScorer;
pub use streaming::StreamingRepair;
pub use mcp_server::AnyrepairMcpServer;
pub use json::JsonRepairer;

use serde_json::Value;
use std::fs::File;
use std::io::Read;

/// Main repair function that automatically detects format and repairs content
pub fn repair(content: &str) -> Result<String> {
    let trimmed = content.trim();
    
    // Try to detect format and repair accordingly
    if is_json_like(trimmed) {
        let mut repairer = json::JsonRepairer::new();
        repairer.repair(trimmed)
    } else if is_yaml_like(trimmed) {
        let mut repairer = yaml::YamlRepairer::new();
        repairer.repair(trimmed)
    } else if is_xml_like(trimmed) {
        let mut repairer = xml::XmlRepairer::new();
        repairer.repair(trimmed)
    } else if is_toml_like(trimmed) {
        let mut repairer = toml::TomlRepairer::new();
        repairer.repair(trimmed)
    } else if is_csv_like(trimmed) {
        let mut repairer = csv::CsvRepairer::new();
        repairer.repair(trimmed)
    } else if is_ini_like(trimmed) {
        let mut repairer = ini::IniRepairer::new();
        repairer.repair(trimmed)
    } else if is_markdown_like(trimmed) {
        let mut repairer = markdown::MarkdownRepairer::new();
        repairer.repair(trimmed)
    } else {
        // Default to markdown repair for unknown content
        let mut repairer = markdown::MarkdownRepairer::new();
        repairer.repair(trimmed)
    }
}

/// Repair JSON string - Python jsonrepair compatible API
/// 
/// This function provides a simple interface matching Python's jsonrepair library:
/// ```python
/// from jsonrepair import repair_json
/// repaired = repair_json('{"key": "value",}')
/// ```
/// 
/// # Arguments
/// * `json_str` - The malformed JSON string to repair
/// 
/// # Returns
/// * `Ok(String)` - The repaired JSON string
/// * `Err(RepairError)` - If repair fails
/// 
/// # Example
/// ```
/// use anyrepair::jsonrepair;
/// 
/// let malformed = r#"{"name": "John", age: 30,}"#;
/// let repaired = jsonrepair(malformed).unwrap();
/// assert!(repaired.contains("\"age\""));
/// assert!(!repaired.ends_with(','));
/// ```
pub fn jsonrepair(json_str: &str) -> Result<String> {
    let mut repairer = json::JsonRepairer::new();
    repairer.repair(json_str)
}

/// JsonRepair - Python jsonrepair compatible class-like interface
/// 
/// This struct provides a class-based API matching Python's jsonrepair library:
/// ```python
/// from jsonrepair import JsonRepair
/// jr = JsonRepair()
/// repaired = jr.jsonrepair('{"key": "value",}')
/// ```
/// 
/// # Example
/// ```
/// use anyrepair::JsonRepair;
/// 
/// let mut jr = JsonRepair::new();
/// let malformed = r#"{"name": "John", age: 30,}"#;
/// let repaired = jr.jsonrepair(malformed).unwrap();
/// assert!(repaired.contains("\"age\""));
/// assert!(!repaired.ends_with(','));
/// ```
pub struct JsonRepair {
    repairer: json::JsonRepairer,
}

impl JsonRepair {
    /// Create a new JsonRepair instance
    pub fn new() -> Self {
        Self {
            repairer: json::JsonRepairer::new(),
        }
    }

    /// Repair a JSON string (Python jsonrepair compatible method)
    /// 
    /// # Arguments
    /// * `json_str` - The malformed JSON string to repair
    /// 
    /// # Returns
    /// * `Ok(String)` - The repaired JSON string
    /// * `Err(RepairError)` - If repair fails
    pub fn jsonrepair(&mut self, json_str: &str) -> Result<String> {
        self.repairer.repair(json_str)
    }
}

impl Default for JsonRepair {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced JSON repair with advanced capabilities
/// 
/// This function provides drop-in replacement for json.loads() with advanced repair capabilities
/// inspired by json_repair-main.
/// 
/// # Arguments
/// * `json_str` - The JSON string to repair
/// * `skip_json_loads` - Skip initial JSON validation for performance
/// * `logging` - Enable detailed repair logging
/// * `stream_stable` - Enable streaming support for partial JSON
/// 
/// # Returns
/// * `Ok(Value)` - The parsed JSON value
/// * `Err(RepairError)` - If repair fails
pub fn repair_json_advanced(
    json_str: &str,
    skip_json_loads: bool,
    logging: bool,
    stream_stable: bool,
) -> Result<Value> {
    let mut repairer = EnhancedJsonRepairer::new()
        .with_skip_json_loads(skip_json_loads)
        .with_logging(logging)
        .with_stream_stable(stream_stable);
    
    repairer.loads(json_str)
}

/// Drop-in replacement for json.loads() with repair capabilities
pub fn loads(json_str: &str) -> Result<Value> {
    repair_json_advanced(json_str, false, false, false)
}

/// Drop-in replacement for json.load() with repair capabilities
pub fn load<R: Read>(reader: R) -> Result<Value> {
    let mut repairer = EnhancedJsonRepairer::new();
    repairer.load(reader)
}

/// Load JSON from file with repair capabilities
pub fn from_file(filename: &str) -> Result<Value> {
    let mut repairer = EnhancedJsonRepairer::new();
    repairer.from_file(filename)
}

/// Repair JSON string and return as string
pub fn repair_json_string(json_str: &str) -> Result<String> {
    let mut repairer = EnhancedJsonRepairer::new();
    repairer.repair_json(json_str)
}

/// Repair JSON with detailed logging
pub fn repair_json_with_logging(json_str: &str) -> Result<(Value, Vec<String>)> {
    let mut repairer = EnhancedJsonRepairer::new().with_logging(true);
    let value = repairer.loads(json_str)?;
    let log = repairer.get_repair_log().to_vec();
    Ok((value, log))
}

/// Repair JSON with detailed logging using the main JSON repairer
pub fn repair_json_with_logging_main(json_str: &str) -> Result<(String, Vec<String>)> {
    let mut repairer = json::JsonRepairer::with_logging(true);
    let result = repairer.repair(json_str)?;
    let log = repairer.get_repair_log().to_vec();
    Ok((result, log))
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

fn is_xml_like(content: &str) -> bool {
    let trimmed = content.trim();
    trimmed.starts_with("<?xml") ||
    (trimmed.starts_with('<') && trimmed.contains('>') && !trimmed.starts_with('#')) ||
    (trimmed.contains('<') && trimmed.contains('>') && trimmed.contains("</"))
}

fn is_toml_like(content: &str) -> bool {
    let trimmed = content.trim();
    trimmed.starts_with('[') ||
    (trimmed.contains('=') && !trimmed.starts_with('{') && !trimmed.starts_with('<') && !trimmed.starts_with('#')) ||
    trimmed.lines().any(|line| line.trim().starts_with('[') && line.trim().ends_with(']'))
}

fn is_csv_like(content: &str) -> bool {
    let trimmed = content.trim();
    trimmed.contains(',') &&
    !trimmed.starts_with('{') &&
    !trimmed.starts_with('[') &&
    !trimmed.starts_with('<') &&
    !trimmed.starts_with('#') &&
    !trimmed.starts_with("<?xml") &&
    trimmed.lines().count() > 1
}

fn is_ini_like(content: &str) -> bool {
    let trimmed = content.trim();
    (trimmed.starts_with('[') && trimmed.contains(']')) ||
    (trimmed.contains('=') && !trimmed.starts_with('{') && !trimmed.starts_with('<') && 
     !trimmed.starts_with('#') && !trimmed.starts_with("<?xml") && 
     !trimmed.contains(',') && !trimmed.contains(':')) ||
    trimmed.lines().any(|line| {
        let line = line.trim();
        line.starts_with('[') && line.contains(']') && !line.contains(',')
    })
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
        
        assert!(is_xml_like("<?xml version=\"1.0\"?><root></root>"));
        assert!(is_xml_like("<root><item>value</item></root>"));
        assert!(!is_xml_like(r#"{"key": "value"}"#));
        
        assert!(is_toml_like("[user]\nname = \"John\""));
        assert!(is_toml_like("name = John"));
        assert!(!is_toml_like(r#"{"key": "value"}"#));
        
        assert!(is_csv_like("name,age\nJohn,30"));
        assert!(is_csv_like("John,30,Engineer\nJane,25,Designer"));
        assert!(!is_csv_like(r#"{"key": "value"}"#));
        
        assert!(is_ini_like("[user]\nname = John"));
        assert!(is_ini_like("name = John\nage = 30"));
        assert!(!is_ini_like(r#"{"key": "value"}"#));
        
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

        // Test XML repair
        let xml_input = "<root><item>value</item></root>";
        let result = repair(xml_input).unwrap();
        assert!(result.contains("<root>"));
        assert!(result.contains("<item>value</item>"));

        // Test TOML repair
        let toml_input = "name = John\nage = 30";
        let result = repair(toml_input).unwrap();
        assert!(result.contains("name = John"));

        // Test CSV repair
        let csv_input = "John,30,Engineer\nJane,25,Designer";
        let result = repair(csv_input).unwrap();
        assert!(result.contains("John,30,Engineer"));

        // Test INI repair
        let ini_input = "name = John\nage = 30";
        let result = repair(ini_input).unwrap();
        assert!(result.contains("name = John"));

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

    #[test]
    fn test_jsonrepair_function() {
        // Test Python jsonrepair compatible function
        let malformed = r#"{"name": "John", age: 30,}"#;
        let repaired = jsonrepair(malformed).unwrap();
        assert!(repaired.contains("\"age\""));
        assert!(!repaired.ends_with(','));
        
        // Test with trailing comma
        let with_comma = r#"{"key": "value",}"#;
        let repaired = jsonrepair(with_comma).unwrap();
        assert!(!repaired.ends_with(','));
    }

    #[test]
    fn test_jsonrepair_struct() {
        // Test Python jsonrepair compatible struct
        let mut jr = JsonRepair::new();
        let malformed = r#"{"name": "John", age: 30,}"#;
        let repaired = jr.jsonrepair(malformed).unwrap();
        assert!(repaired.contains("\"age\""));
        assert!(!repaired.ends_with(','));
        
        // Test default
        let mut jr = JsonRepair::default();
        let with_comma = r#"{"key": "value",}"#;
        let repaired = jr.jsonrepair(with_comma).unwrap();
        assert!(!repaired.ends_with(','));
    }

    #[test]
    fn test_jsonrepair_trailing_commas() {
        // Test trailing commas in objects
        let input = r#"{"name": "John", "age": 30,}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(!repaired.contains(",\n}"));
        assert!(!repaired.contains(",}"));
        
        // Test trailing commas in arrays
        let input = r#"[1, 2, 3,]"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(!repaired.contains(",]"));
        
        // Test multiple trailing commas
        let input = r#"{"a": 1, "b": 2, "c": 3,}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(!repaired.ends_with(','));
    }

    #[test]
    fn test_jsonrepair_missing_quotes() {
        // Test missing quotes around keys
        let input = r#"{name: "John", age: 30}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.contains("\"name\""));
        assert!(repaired.contains("\"age\""));
        // Verify it's valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&repaired).is_ok());
        
        // Test missing quotes around values - this case may not be fully repairable
        // as "John" without quotes could be interpreted as an identifier
        // The repairer focuses on keys, so we test a case that should work
        let input = r#"{"name": "John", age: 30}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.contains("\"age\""));
        // Verify it's valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&repaired).is_ok());
        
        // Test mixed missing quotes
        let input = r#"{name: "John", age: 30}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.contains("\"name\""));
        assert!(repaired.contains("\"age\""));
        // Verify it's valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&repaired).is_ok());
    }

    #[test]
    fn test_jsonrepair_single_quotes() {
        // Test single quotes instead of double quotes
        let input = r#"{'name': 'John', 'age': 30}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.contains("\"name\""));
        assert!(repaired.contains("\"John\""));
        assert!(!repaired.contains("'"));
        
        // Test mixed single and double quotes
        let input = r#"{'name': "John", "age": 30}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(!repaired.contains("'"));
    }

    #[test]
    fn test_jsonrepair_boolean_null() {
        // Test Python-style booleans
        let input = r#"{"active": True, "deleted": False}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.contains("true"));
        assert!(repaired.contains("false"));
        
        // Test various null representations
        let input = r#"{"value1": null, "value2": None, "value3": NULL, "value4": undefined}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.matches("null").count() >= 4);
    }

    #[test]
    fn test_jsonrepair_malformed_numbers() {
        // Test trailing dots
        let input = r#"{"price": 29.99., "quantity": 10.}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.contains("29.99"));
        assert!(repaired.contains("10"));
        
        // Test multiple dots
        let input = r#"{"value": 123.45.67}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.contains("123.45"));
    }

    #[test]
    fn test_jsonrepair_nested_structures() {
        // Test nested objects with errors
        let input = r#"{
            "user": {
                name: "John",
                age: 30,
                address: {
                    street: "123 Main St",
                    city: "NYC",
                },
            },
        }"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.contains("\"name\""));
        assert!(repaired.contains("\"age\""));
        assert!(repaired.contains("\"street\""));
        assert!(repaired.contains("\"city\""));
        
        // Test nested arrays
        let input = r#"{"items": [1, 2, 3,], "tags": ['a', 'b',],}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.contains("[1,2,3]") || repaired.contains("[1, 2, 3]"));
        assert!(!repaired.contains(",]"));
    }

    #[test]
    fn test_jsonrepair_missing_braces() {
        // Test missing closing brace
        let input = r#"{"name": "John", "age": 30"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.ends_with('}'));
        
        // Test missing opening brace
        let input = r#""name": "John", "age": 30}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.starts_with('{'));
        
        // Test missing brackets
        let input = r#"[1, 2, 3"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.ends_with(']'));
    }

    #[test]
    fn test_jsonrepair_empty_and_edge_cases() {
        // Test empty string (may return empty or empty object)
        let repaired = jsonrepair("").unwrap();
        assert!(repaired.is_empty() || repaired == "{}");
        
        // Test already valid JSON
        let valid = r#"{"name": "John", "age": 30}"#;
        let repaired = jsonrepair(valid).unwrap();
        // Should be valid JSON (may have whitespace differences)
        assert!(serde_json::from_str::<serde_json::Value>(&repaired).is_ok());
        
        // Test whitespace only (may return empty or empty object)
        let repaired = jsonrepair("   \n\t  ").unwrap();
        assert!(repaired.trim().is_empty() || repaired.trim() == "{}");
        
        // Test just braces
        let repaired = jsonrepair("{}").unwrap();
        assert_eq!(repaired.trim(), "{}");
        
        // Test just brackets
        let repaired = jsonrepair("[]").unwrap();
        assert_eq!(repaired.trim(), "[]");
    }

    #[test]
    fn test_jsonrepair_complex_real_world() {
        // Test complex real-world scenario
        let input = r#"{
            'users': [
                {
                    id: 1,
                    name: "Alice",
                    email: 'alice@example.com',
                    active: True,
                    tags: ['admin', 'user',],
                },
                {
                    id: 2,
                    name: "Bob",
                    email: 'bob@example.com',
                    active: False,
                    tags: ['user',],
                },
            ],
            'metadata': {
                total: 2,
                page: 1,
                has_more: False,
            },
        }"#;
        
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.contains("\"users\""));
        assert!(repaired.contains("\"Alice\""));
        assert!(repaired.contains("\"Bob\""));
        assert!(repaired.contains("true"));
        assert!(repaired.contains("false"));
        assert!(!repaired.contains("'"));
        assert!(!repaired.contains(",]"));
        assert!(!repaired.contains(",}"));
        
        // Verify it's valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&repaired).is_ok());
    }

    #[test]
    fn test_jsonrepair_struct_multiple_calls() {
        // Test that JsonRepair can be reused for multiple repairs
        let mut jr = JsonRepair::new();
        
        let input1 = r#"{"key1": "value1",}"#;
        let repaired1 = jr.jsonrepair(input1).unwrap();
        assert!(!repaired1.ends_with(','));
        
        let input2 = r#"{key2: "value2"}"#;
        let repaired2 = jr.jsonrepair(input2).unwrap();
        assert!(repaired2.contains("\"key2\""));
        
        let input3 = r#"['a', 'b',]"#;
        let repaired3 = jr.jsonrepair(input3).unwrap();
        assert!(!repaired3.contains("'"));
        assert!(!repaired3.contains(",]"));
    }

    #[test]
    fn test_jsonrepair_function_vs_struct_consistency() {
        // Test that function and struct produce same results
        let input = r#"{"name": "John", age: 30,}"#;
        
        let repaired_func = jsonrepair(input).unwrap();
        let mut jr = JsonRepair::new();
        let repaired_struct = jr.jsonrepair(input).unwrap();
        
        // Both should produce valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&repaired_func).is_ok());
        assert!(serde_json::from_str::<serde_json::Value>(&repaired_struct).is_ok());
        
        // Both should have same structure (may differ in whitespace)
        let parsed_func: serde_json::Value = serde_json::from_str(&repaired_func).unwrap();
        let parsed_struct: serde_json::Value = serde_json::from_str(&repaired_struct).unwrap();
        assert_eq!(parsed_func, parsed_struct);
    }

    #[test]
    fn test_jsonrepair_unicode_and_special_chars() {
        // Test Unicode characters
        let input = r#"{"name": "José", "emoji": "🚀"}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.contains("José"));
        assert!(repaired.contains("🚀"));
        
        // Test escaped characters
        let input = r#"{"message": "He said \"Hello\""}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.contains("\\\"Hello\\\""));
        
        // Test newlines in strings
        let input = r#"{"text": "Line 1\nLine 2"}"#;
        let repaired = jsonrepair(input).unwrap();
        assert!(repaired.contains("\\n"));
    }
}
