//! # AnyRepair
//!
//! A Rust crate for repairing malformed structured data including JSON, YAML,
//! XML, TOML, CSV, INI, Markdown, and Diff with format auto-detection.

pub mod csv;
pub mod diff;
pub mod error;
pub mod format_detection;
pub mod json;
pub mod key_value;
pub mod markdown;
pub mod mcp_server;
pub mod repairer_base;
pub mod streaming;
pub mod toml;
pub mod traits;
pub mod xml;
pub mod yaml;

pub use diff::DiffRepairer;
pub use error::{RepairError, Result};
pub use json::JsonRepairer;
pub use key_value::{EnvRepairer, IniRepairer, PropertiesRepairer};
pub use mcp_server::AnyrepairMcpServer;
pub use streaming::StreamingRepair;
pub use traits::Repair;

pub const SUPPORTED_FORMATS: &[&str] = &[
    "json",
    "yaml",
    "markdown",
    "xml",
    "toml",
    "csv",
    "ini",
    "diff",
    "properties",
    "env",
];

pub fn normalize_format(format: &str) -> &str {
    if format.eq_ignore_ascii_case("yml") {
        return "yaml";
    }
    if format.eq_ignore_ascii_case("md") {
        return "markdown";
    }
    for &fmt in SUPPORTED_FORMATS {
        if format.eq_ignore_ascii_case(fmt) {
            return fmt;
        }
    }
    format
}

fn parse_supported_format(format: &str) -> Result<&'static str> {
    let n = normalize_format(format);
    SUPPORTED_FORMATS
        .iter()
        .find(|&&fmt| fmt == n)
        .copied()
        .ok_or_else(|| RepairError::FormatDetection(format!("Unknown format: {}", n)))
}

pub fn create_repairer(format: &str) -> Result<Box<dyn Repair>> {
    match parse_supported_format(format)? {
        "json" => Ok(Box::new(json::JsonRepairer::new())),
        "yaml" => Ok(Box::new(yaml::YamlRepairer::new())),
        "markdown" => Ok(Box::new(markdown::MarkdownRepairer::new())),
        "xml" => Ok(Box::new(xml::XmlRepairer::new())),
        "toml" => Ok(Box::new(toml::TomlRepairer::new())),
        "csv" => Ok(Box::new(csv::CsvRepairer::new())),
        "ini" => Ok(Box::new(key_value::IniRepairer::new())),
        "diff" => Ok(Box::new(diff::DiffRepairer::new())),
        "properties" => Ok(Box::new(key_value::PropertiesRepairer::new())),
        "env" => Ok(Box::new(key_value::EnvRepairer::new())),
        other => Err(RepairError::FormatDetection(format!(
            "Unknown format: {}",
            other
        ))),
    }
}

pub fn create_validator(format: &str) -> Result<Box<dyn traits::Validator>> {
    match parse_supported_format(format)? {
        "json" => Ok(Box::new(json::JsonValidator)),
        "yaml" => Ok(Box::new(yaml::YamlValidator)),
        "markdown" => Ok(Box::new(markdown::MarkdownValidator)),
        "xml" => Ok(Box::new(xml::XmlValidator)),
        "toml" => Ok(Box::new(toml::TomlValidator)),
        "csv" => Ok(Box::new(csv::CsvValidator)),
        "ini" => Ok(Box::new(key_value::IniValidator)),
        "diff" => Ok(Box::new(diff::DiffValidator)),
        "properties" => Ok(Box::new(key_value::PropertiesValidator)),
        "env" => Ok(Box::new(key_value::EnvValidator)),
        other => Err(RepairError::FormatDetection(format!(
            "Unknown format: {}",
            other
        ))),
    }
}

pub fn repair_with_format(content: &str, format: &str) -> Result<String> {
    let mut repairer = create_repairer(format)?;
    repairer.repair(content)
}

pub fn repair(content: &str) -> Result<String> {
    let trimmed = content.trim();
    if let Some(fmt) = detect_format(trimmed) {
        let mut repairer = create_repairer(fmt)?;
        repairer.repair(trimmed)
    } else {
        let mut repairer = markdown::MarkdownRepairer::new();
        repairer.repair(trimmed)
    }
}

pub fn detect_format(content: &str) -> Option<&'static str> {
    format_detection::detect_format(content)
}

pub fn jsonrepair(json_str: &str) -> Result<String> {
    let mut repairer = json::JsonRepairer::new();
    repairer.repair(json_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(detect_format(r#"{"key": "value"}"#), Some("json"));
        assert_eq!(detect_format(r#"[1, 2, 3]"#), Some("json"));
        assert_eq!(detect_format("key: value"), Some("yaml"));
        assert_eq!(detect_format("---\nkey: value"), Some("yaml"));
        assert_eq!(
            detect_format("<?xml version=\"1.0\"?><root></root>"),
            Some("xml")
        );
        assert_eq!(detect_format("name,age\nJohn,30"), Some("csv"));
        assert_eq!(detect_format("# Header\n**bold**"), Some("markdown"));
    }

    #[test]
    fn test_repair_function() {
        let json_input = r#"{"name": "John", "age": 30,}"#;
        let result = repair(json_input).unwrap();
        assert!(result.contains("John"));
        assert!(!result.ends_with(','));

        let yaml_input = "name: John\nage: 30";
        let result = repair(yaml_input).unwrap();
        assert!(result.contains("name: John"));
    }

    #[test]
    fn test_jsonrepair_function() {
        let malformed = r#"{"name": "John", age: 30,}"#;
        let repaired = jsonrepair(malformed).unwrap();
        assert!(repaired.contains("\"age\""));
        assert!(!repaired.ends_with(','));
    }

    #[test]
    fn test_repair_error_handling() {
        let result = repair("");
        assert!(result.is_ok());
    }
}
