//! Error types for the anyrepair crate

use std::fmt;

/// Main error type for repair operations
#[derive(Debug)]
pub enum RepairError {
    JsonRepair(String),
    YamlRepair(String),
    MarkdownRepair(String),
    FormatDetection(String),
    Io(std::io::Error),
    Utf8(std::string::FromUtf8Error),
    Generic(String),
    BinaryRepair(String),
}

impl fmt::Display for RepairError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepairError::JsonRepair(msg) => write!(f, "JSON repair failed: {msg}"),
            RepairError::YamlRepair(msg) => write!(f, "YAML repair failed: {msg}"),
            RepairError::MarkdownRepair(msg) => write!(f, "Markdown repair failed: {msg}"),
            RepairError::FormatDetection(msg) => write!(f, "Format detection failed: {msg}"),
            RepairError::Io(err) => write!(f, "IO error: {err}"),
            RepairError::Utf8(err) => write!(f, "UTF-8 error: {err}"),
            RepairError::Generic(msg) => write!(f, "Generic error: {msg}"),
            RepairError::BinaryRepair(msg) => write!(f, "Binary repair failed: {msg}"),
        }
    }
}

impl std::error::Error for RepairError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RepairError::Io(err) => Some(err),
            RepairError::Utf8(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for RepairError {
    fn from(err: std::io::Error) -> Self {
        RepairError::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for RepairError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        RepairError::Utf8(err)
    }
}

/// Result type alias for repair operations
pub type Result<T> = std::result::Result<T, RepairError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = RepairError::JsonRepair("test error".to_string());
        assert_eq!(err.to_string(), "JSON repair failed: test error");

        let err = RepairError::YamlRepair("yaml error".to_string());
        assert_eq!(err.to_string(), "YAML repair failed: yaml error");

        let err = RepairError::MarkdownRepair("markdown error".to_string());
        assert_eq!(err.to_string(), "Markdown repair failed: markdown error");

        let err = RepairError::FormatDetection("detection error".to_string());
        assert_eq!(err.to_string(), "Format detection failed: detection error");

        let err = RepairError::Generic("generic error".to_string());
        assert_eq!(err.to_string(), "Generic error: generic error");
    }

    #[test]
    fn test_error_display() {
        let err = RepairError::JsonRepair("invalid json".to_string());
        let display_str = format!("{}", err);
        assert!(display_str.contains("JSON repair failed"));
        assert!(display_str.contains("invalid json"));
    }

    #[test]
    fn test_error_debug() {
        let err = RepairError::Generic("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Generic"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_result_type() {
        let ok_result: Result<String> = Ok("success".to_string());
        assert!(ok_result.is_ok());

        let err_result: Result<String> = Err(RepairError::Generic("failed".to_string()));
        assert!(err_result.is_err());
    }
}
