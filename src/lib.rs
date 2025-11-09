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
}
