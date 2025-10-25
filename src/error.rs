//! Error types for the anyrepair crate

use thiserror::Error;

/// Main error type for repair operations
#[derive(Error, Debug)]
pub enum RepairError {
    #[error("JSON repair failed: {0}")]
    JsonRepair(String),
    
    #[error("YAML repair failed: {0}")]
    YamlRepair(String),
    
    #[error("Markdown repair failed: {0}")]
    MarkdownRepair(String),
    
    #[error("Format detection failed: {0}")]
    FormatDetection(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    
    #[error("CSV writer error: {0}")]
    CsvWriter(#[from] csv::IntoInnerError<csv::Writer<Vec<u8>>>),
    
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    
    #[error("Generic error: {0}")]
    Generic(String),
}

/// Result type alias for repair operations
pub type Result<T> = std::result::Result<T, RepairError>;

impl RepairError {
    /// Create a new JSON repair error
    pub fn json_repair(msg: impl Into<String>) -> Self {
        Self::JsonRepair(msg.into())
    }
    
    /// Create a new YAML repair error
    pub fn yaml_repair(msg: impl Into<String>) -> Self {
        Self::YamlRepair(msg.into())
    }
    
    /// Create a new Markdown repair error
    pub fn markdown_repair(msg: impl Into<String>) -> Self {
        Self::MarkdownRepair(msg.into())
    }
    
    /// Create a new format detection error
    pub fn format_detection(msg: impl Into<String>) -> Self {
        Self::FormatDetection(msg.into())
    }
    
    /// Create a new generic error
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = RepairError::json_repair("test error");
        assert_eq!(err.to_string(), "JSON repair failed: test error");

        let err = RepairError::yaml_repair("yaml error");
        assert_eq!(err.to_string(), "YAML repair failed: yaml error");

        let err = RepairError::markdown_repair("markdown error");
        assert_eq!(err.to_string(), "Markdown repair failed: markdown error");

        let err = RepairError::format_detection("detection error");
        assert_eq!(err.to_string(), "Format detection failed: detection error");

        let err = RepairError::generic("generic error");
        assert_eq!(err.to_string(), "Generic error: generic error");
    }

    #[test]
    fn test_error_display() {
        let err = RepairError::json_repair("invalid json");
        let display_str = format!("{}", err);
        assert!(display_str.contains("JSON repair failed"));
        assert!(display_str.contains("invalid json"));
    }

    #[test]
    fn test_error_debug() {
        let err = RepairError::generic("test");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Generic"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_result_type() {
        let ok_result: Result<String> = Ok("success".to_string());
        assert!(ok_result.is_ok());

        let err_result: Result<String> = Err(RepairError::generic("failed"));
        assert!(err_result.is_err());
    }
}
