//! # AnyRepair
//!
//! A Rust crate for repairing malformed structured data including JSON, YAML,
//! XML, TOML, CSV, INI, Markdown, and Diff with format auto-detection.

pub mod audio;
pub mod binary_detection;
pub mod binary_traits;
pub mod csv;
pub mod diff;
pub mod error;
pub mod image;
pub mod json_util;
pub mod format_detection;
pub mod json;
pub mod key_value;
pub mod markdown;
pub mod mcp_server;
mod pattern;
pub mod repairer_base;
pub mod streaming;
pub mod toml;
pub mod traits;
pub mod video;
pub mod xml;
pub mod yaml;

pub use audio::{Mp3Repairer, WavRepairer};
pub use binary_detection::{detect_binary_format, is_binary_format, BinaryDetectionResult};
pub use binary_traits::{BinaryRepair, BinaryValidator};
pub use diff::DiffRepairer;
pub use error::{RepairError, Result};
pub use image::{JpegRepairer, PngRepairer};
pub use json::JsonRepairer;
pub use key_value::{EnvRepairer, IniRepairer, PropertiesRepairer};
pub use mcp_server::AnyrepairMcpServer;
pub use streaming::StreamingRepair;
pub use traits::Repair;
pub use video::{MovRepairer, Mp4Repairer};

/// All text format identifiers supported by anyrepair.
/// Used by `create_repairer`, `create_validator`, and `repair_with_format`.
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

/// All binary (media) format identifiers supported by anyrepair.
pub const BINARY_FORMATS: &[&str] = &["png", "jpeg", "wav", "mp3", "mp4", "mov"];

/// Normalize a format alias to its canonical name.
/// Accepts case-insensitive matches and common aliases (`yml` → `yaml`, `md` → `markdown`, `jpg` → `jpeg`).
/// Returns the original string unchanged if no match is found.
pub fn normalize_format(format: &str) -> &str {
    if format.eq_ignore_ascii_case("yml") {
        return "yaml";
    }
    if format.eq_ignore_ascii_case("md") {
        return "markdown";
    }
    if format.eq_ignore_ascii_case("jpg") {
        return "jpeg";
    }
    for &fmt in SUPPORTED_FORMATS {
        if format.eq_ignore_ascii_case(fmt) {
            return fmt;
        }
    }
    for &fmt in BINARY_FORMATS {
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

/// Generate `create_repairer`, `create_validator`, and `repair_with_explanations`
/// from a single list of `(format, repairer, validator)`. New text formats are
/// registered here in exactly one place.
macro_rules! gen_text_format_dispatch {
    ($(($fmt:ident, $repairer:ty, $validator:path)),* $(,)?) => {
        /// Create a boxed [`Repair`] instance for the given format.
        /// Accepts canonical names and aliases (e.g. `yml`, `md`).
        /// Returns `RepairError::FormatDetection` if the format is unknown.
        pub fn create_repairer(format: &str) -> Result<Box<dyn Repair>> {
            match parse_supported_format(format)? {
                $(stringify!($fmt) => Ok(Box::new(<$repairer>::new())),)*
                other => Err(RepairError::FormatDetection(format!(
                    "Unknown format: {}",
                    other
                ))),
            }
        }

        /// Create a boxed [`Validator`](traits::Validator) for the given format.
        /// Accepts canonical names and aliases (e.g. `yml`, `md`).
        /// Returns `RepairError::FormatDetection` if the format is unknown.
        pub fn create_validator(format: &str) -> Result<Box<dyn traits::Validator>> {
            match parse_supported_format(format)? {
                $(stringify!($fmt) => Ok(Box::new($validator)),)*
                other => Err(RepairError::FormatDetection(format!(
                    "Unknown format: {}",
                    other
                ))),
            }
        }

        /// Repair content with a specific format and return the list of strategies that changed it.
        /// Returns `(repaired_content, applied_strategy_names)`.
        pub fn repair_with_explanations(
            content: &str,
            format: &str,
        ) -> Result<(String, Vec<String>)> {
            let trimmed = content.trim();
            match parse_supported_format(format)? {
                $(stringify!($fmt) => <$repairer>::new().inner.repair_with_explanations(trimmed),)*
                other => Err(RepairError::FormatDetection(format!(
                    "Unknown format: {}",
                    other
                ))),
            }
        }
    };
}

gen_text_format_dispatch!(
    (json, json::JsonRepairer, json::JsonValidator),
    (yaml, yaml::YamlRepairer, yaml::YamlValidator),
    (markdown, markdown::MarkdownRepairer, markdown::MarkdownValidator),
    (xml, xml::XmlRepairer, xml::XmlValidator),
    (toml, toml::TomlRepairer, toml::TomlValidator),
    (csv, csv::CsvRepairer, csv::CsvValidator),
    (ini, key_value::IniRepairer, key_value::IniValidator),
    (diff, diff::DiffRepairer, diff::DiffValidator),
    (properties, key_value::PropertiesRepairer, key_value::PropertiesValidator),
    (env, key_value::EnvRepairer, key_value::EnvValidator),
);

/// Repair content using an explicit format.
/// Convenience wrapper around `create_repairer` + `Repair::repair`.
pub fn repair_with_format(content: &str, format: &str) -> Result<String> {
    let mut repairer = create_repairer(format)?;
    repairer.repair(content)
}

/// Repair content with automatic format detection.
/// Falls back to the Markdown repairer if no format is detected.
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

/// Detect the format of the given content.
/// Returns `None` if no known format matches.
/// See [`format_detection`] for the heuristic order.
pub fn detect_format(content: &str) -> Option<&'static str> {
    format_detection::detect_format(content)
}

/// Detect format with a confidence score (`0.0..=1.0`).
///
/// Same heuristics as [`detect_format`], but also returns how strong the match is.
pub fn detect_format_with_confidence(
    content: &str,
) -> Option<format_detection::DetectionResult> {
    format_detection::detect_format_with_confidence(content)
}

pub use format_detection::DetectionResult;

/// Create a boxed [`BinaryRepair`] instance for the given media format.
/// Accepts canonical names and aliases (e.g. `jpg` → `jpeg`).
/// Returns `RepairError::FormatDetection` if the format is unknown.
pub fn create_binary_repairer(format: &str) -> Result<Box<dyn BinaryRepair>> {
    let n = normalize_format(format);
    match n {
        "png" => Ok(Box::new(image::PngRepairer::new())),
        "jpeg" => Ok(Box::new(image::JpegRepairer::new())),
        "wav" => Ok(Box::new(audio::WavRepairer::new())),
        "mp3" => Ok(Box::new(audio::Mp3Repairer::new())),
        "mp4" => Ok(Box::new(video::Mp4Repairer::new())),
        "mov" => Ok(Box::new(video::MovRepairer::new())),
        other => Err(RepairError::FormatDetection(format!(
            "Unknown binary format: {}",
            other
        ))),
    }
}

/// Create a boxed [`BinaryValidator`] for the given media format.
pub fn create_binary_validator(format: &str) -> Result<Box<dyn BinaryValidator>> {
    let n = normalize_format(format);
    match n {
        "png" => Ok(Box::new(image::PngValidator)),
        "jpeg" => Ok(Box::new(image::JpegValidator)),
        "wav" => Ok(Box::new(audio::WavValidator)),
        "mp3" => Ok(Box::new(audio::Mp3Validator)),
        "mp4" => Ok(Box::new(video::Mp4Validator)),
        "mov" => Ok(Box::new(video::Mp4Validator)),
        other => Err(RepairError::FormatDetection(format!(
            "Unknown binary format: {}",
            other
        ))),
    }
}

/// Repair binary content using an explicit format.
pub fn repair_binary_with_format(data: &[u8], format: &str) -> Result<Vec<u8>> {
    let mut repairer = create_binary_repairer(format)?;
    repairer.repair(data)
}

/// Repair binary content with automatic format detection.
/// Falls back to returning the original data if no format is detected.
pub fn repair_binary(data: &[u8]) -> Result<Vec<u8>> {
    if let Some(detected) = binary_detection::detect_binary_format(data) {
        let mut repairer = create_binary_repairer(detected.format)?;
        repairer.repair(data)
    } else {
        Ok(data.to_vec())
    }
}

/// Repair a JSON string (Python-compatible convenience function).
/// Equivalent to `create_repairer("json")?.repair(json_str)`.
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
