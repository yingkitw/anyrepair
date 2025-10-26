//! Batch processing module for multi-format repair operations

use crate::error::Result;
use crate::traits::Repair;
use crate::analytics::{AnalyticsTracker, PerformanceMonitor};
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Batch processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// Total files processed
    pub total_files: usize,
    /// Successfully repaired files
    pub successful_files: usize,
    /// Failed files
    pub failed_files: usize,
    /// Files that were skipped
    pub skipped_files: usize,
    /// Results per file
    pub file_results: Vec<FileResult>,
}

/// Individual file result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    /// File path
    pub file_path: String,
    /// Whether repair was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// File format detected
    pub format: String,
    /// Time taken in milliseconds
    pub time_ms: u64,
}

/// Batch processor for handling multiple files
pub struct BatchProcessor {
    analytics: AnalyticsTracker,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new() -> Self {
        Self {
            analytics: AnalyticsTracker::new(),
        }
    }

    /// Process all files in a directory
    pub fn process_directory(
        &self,
        directory: &Path,
        recursive: bool,
        extensions: Option<&[&str]>,
    ) -> Result<BatchResult> {
        let mut results = BatchResult {
            total_files: 0,
            successful_files: 0,
            failed_files: 0,
            skipped_files: 0,
            file_results: Vec::new(),
        };

        self.process_dir_recursive(directory, recursive, extensions, &mut results)?;
        Ok(results)
    }

    /// Process a list of files
    pub fn process_files(&self, files: &[PathBuf]) -> Result<BatchResult> {
        let mut results = BatchResult {
            total_files: files.len(),
            successful_files: 0,
            failed_files: 0,
            skipped_files: 0,
            file_results: Vec::new(),
        };

        for file_path in files {
            if file_path.is_file() {
                self.process_single_file(file_path, &mut results);
            } else {
                results.skipped_files += 1;
            }
        }

        Ok(results)
    }

    fn process_dir_recursive(
        &self,
        directory: &Path,
        recursive: bool,
        extensions: Option<&[&str]>,
        results: &mut BatchResult,
    ) -> Result<()> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() && recursive {
                self.process_dir_recursive(&path, recursive, extensions, results)?;
            } else if path.is_file() {
                // Check if file should be processed based on extension
                if let Some(exts) = extensions {
                    if let Some(ext) = path.extension() {
                        if let Some(ext_str) = ext.to_str() {
                            if exts.contains(&ext_str) {
                                results.total_files += 1;
                                self.process_single_file(&path, results);
                            } else {
                                results.skipped_files += 1;
                            }
                        }
                    }
                } else {
                    results.total_files += 1;
                    self.process_single_file(&path, results);
                }
            }
        }

        Ok(())
    }

    fn process_single_file(&self, file_path: &Path, results: &mut BatchResult) {
        let monitor = PerformanceMonitor::new(file_path.to_string_lossy().as_ref());

        match fs::read_to_string(file_path) {
            Ok(content) => {
                let format = detect_format(&content);
                match crate::repair(&content) {
                    Ok(repaired) => {
                        // Try to write back the repaired content
                        match fs::write(file_path, &repaired) {
                            Ok(_) => {
                                results.successful_files += 1;
                                self.analytics.record_repair(&format, true, monitor.elapsed(), 1.0);
                                results.file_results.push(FileResult {
                                    file_path: file_path.to_string_lossy().to_string(),
                                    success: true,
                                    error: None,
                                    format,
                                    time_ms: monitor.elapsed_ms(),
                                });
                            }
                            Err(e) => {
                                results.failed_files += 1;
                                self.analytics.record_repair(&format, false, monitor.elapsed(), 0.0);
                                results.file_results.push(FileResult {
                                    file_path: file_path.to_string_lossy().to_string(),
                                    success: false,
                                    error: Some(format!("Failed to write file: {}", e)),
                                    format,
                                    time_ms: monitor.elapsed_ms(),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        results.failed_files += 1;
                        let format = detect_format(&content);
                        self.analytics.record_repair(&format, false, monitor.elapsed(), 0.0);
                        results.file_results.push(FileResult {
                            file_path: file_path.to_string_lossy().to_string(),
                            success: false,
                            error: Some(e.to_string()),
                            format,
                            time_ms: monitor.elapsed_ms(),
                        });
                    }
                }
            }
            Err(e) => {
                results.failed_files += 1;
                results.file_results.push(FileResult {
                    file_path: file_path.to_string_lossy().to_string(),
                    success: false,
                    error: Some(format!("Failed to read file: {}", e)),
                    format: "unknown".to_string(),
                    time_ms: monitor.elapsed_ms(),
                });
            }
        }
    }

    /// Get analytics tracker
    pub fn analytics(&self) -> &AnalyticsTracker {
        &self.analytics
    }
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect format from content
fn detect_format(content: &str) -> String {
    let trimmed = content.trim();

    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        "json".to_string()
    } else if trimmed.starts_with("<?xml") || trimmed.starts_with('<') {
        "xml".to_string()
    } else if trimmed.contains("---") || (trimmed.contains(':') && !trimmed.starts_with('{')) {
        "yaml".to_string()
    } else if trimmed.starts_with('[') || trimmed.contains('=') {
        "toml".to_string()
    } else if trimmed.contains(',') && trimmed.lines().count() > 1 {
        "csv".to_string()
    } else if trimmed.contains('=') && trimmed.contains('[') {
        "ini".to_string()
    } else if trimmed.starts_with('#') || trimmed.contains("```") {
        "markdown".to_string()
    } else {
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_processor_creation() {
        let processor = BatchProcessor::new();
        assert!(processor.analytics().get_metrics().total_repairs == 0);
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(detect_format(r#"{"key": "value"}"#), "json");
        assert_eq!(detect_format("key: value"), "yaml");
        assert_eq!(detect_format("<?xml version=\"1.0\"?>"), "xml");
        assert_eq!(detect_format("# Header"), "markdown");
    }
}
