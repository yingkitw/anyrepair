//! Error types for the anyrepair crate

use thiserror::Error;

/// Main error type for repair operations
#[derive(Error, Debug)]
pub enum RepairError {
    #[error("Format repair failed: {0}")]
    FormatRepair(String),

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
    CsvWriter(#[from] Box<csv::IntoInnerError<csv::Writer<Vec<u8>>>>),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// Result type alias for repair operations
pub type Result<T> = std::result::Result<T, RepairError>;

impl RepairError {
    /// Create a new format repair error
    pub fn format_repair(format: &str, msg: impl Into<String>) -> Self {
        Self::FormatRepair(format!("{}: {}", format, msg.into()))
    }

    /// Create a new format detection error
    pub fn format_detection(msg: impl Into<String>) -> Self {
        Self::FormatDetection(msg.into())
    }
}

impl From<csv::IntoInnerError<csv::Writer<Vec<u8>>>> for RepairError {
    fn from(err: csv::IntoInnerError<csv::Writer<Vec<u8>>>) -> Self {
        Self::CsvWriter(Box::new(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = RepairError::format_repair("json", "test error");
        assert!(err.to_string().contains("json"));
        assert!(err.to_string().contains("test error"));

        let err = RepairError::format_detection("detection error");
        assert_eq!(err.to_string(), "Format detection failed: detection error");
    }

    #[test]
    fn test_result_type() {
        let ok_result: Result<String> = Ok("success".to_string());
        assert!(ok_result.is_ok());

        let err_result: Result<String> = Err(RepairError::format_repair("json", "failed"));
        assert!(err_result.is_err());
    }
}
