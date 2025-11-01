//! Streaming repair for large files with minimal memory overhead
//!
//! This module provides streaming repair capabilities for processing large files
//! without loading entire content into memory.

use crate::error::Result;
use crate::traits::Repair;
use std::io::{BufRead, Write};

/// Streaming repair processor for large files
pub struct StreamingRepair {
    buffer_size: usize,
}

impl StreamingRepair {
    /// Create a new streaming repair processor
    pub fn new() -> Self {
        Self {
            buffer_size: 8192, // 8KB default buffer
        }
    }

    /// Create with custom buffer size
    pub fn with_buffer_size(buffer_size: usize) -> Self {
        Self { buffer_size }
    }

    /// Process a reader and write repaired content to writer
    /// Returns number of bytes processed
    pub fn process<R: BufRead, W: Write>(
        &self,
        reader: R,
        writer: &mut W,
        format: &str,
    ) -> Result<usize> {
        let mut total_bytes = 0;
        let mut buffer = String::with_capacity(self.buffer_size);

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| {
                crate::error::RepairError::Generic(format!("IO error: {}", e))
            })?;

            buffer.push_str(&line);
            buffer.push('\n');

            // Process buffer when it reaches size threshold
            if buffer.len() >= self.buffer_size {
                let repaired = self.repair_chunk(&buffer, format)?;
                writer.write_all(repaired.as_bytes()).map_err(|e| {
                    crate::error::RepairError::Generic(format!("Write error: {}", e))
                })?;
                total_bytes += repaired.len();
                buffer.clear();
            }
        }

        // Process remaining buffer
        if !buffer.is_empty() {
            let repaired = self.repair_chunk(&buffer, format)?;
            writer.write_all(repaired.as_bytes()).map_err(|e| {
                crate::error::RepairError::Generic(format!("Write error: {}", e))
            })?;
            total_bytes += repaired.len();
        }

        Ok(total_bytes)
    }

    /// Repair a chunk of content
    fn repair_chunk(&self, chunk: &str, format: &str) -> Result<String> {
        match format.to_lowercase().as_str() {
            "json" => crate::json::JsonRepairer::new().repair(chunk),
            "yaml" | "yml" => crate::yaml::YamlRepairer::new().repair(chunk),
            "markdown" | "md" => crate::markdown::MarkdownRepairer::new().repair(chunk),
            "xml" => crate::xml::XmlRepairer::new().repair(chunk),
            "toml" => crate::toml::TomlRepairer::new().repair(chunk),
            "csv" => crate::csv::CsvRepairer::new().repair(chunk),
            "ini" => crate::ini::IniRepairer::new().repair(chunk),
            _ => crate::repair(chunk),
        }
    }
}

impl Default for StreamingRepair {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_streaming_json_repair() {
        let input = r#"{"name": "John",
"age": 30,
"city": "NYC",}"#;

        let reader = Cursor::new(input);
        let mut output = Vec::new();
        let processor = StreamingRepair::new();

        let result = processor.process(reader, &mut output, "json");
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("\"name\""));
        assert!(output_str.contains("\"age\""));
    }

    #[test]
    fn test_streaming_yaml_repair() {
        let input = "name: John\nage: 30\ncity: NYC";

        let reader = Cursor::new(input);
        let mut output = Vec::new();
        let processor = StreamingRepair::new();

        let result = processor.process(reader, &mut output, "yaml");
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("name"));
    }

    #[test]
    fn test_streaming_with_custom_buffer() {
        let input = r#"{"key": "value",}"#;

        let reader = Cursor::new(input);
        let mut output = Vec::new();
        let processor = StreamingRepair::with_buffer_size(256);

        let result = processor.process(reader, &mut output, "json");
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    #[test]
    fn test_streaming_large_file_simulation() {
        // Simulate large file with multiple chunks
        let mut input = String::new();
        for i in 0..100 {
            input.push_str(&format!(r#"{{"id": {}, "value": "item",}}"#, i));
            input.push('\n');
        }

        let reader = Cursor::new(input);
        let mut output = Vec::new();
        let processor = StreamingRepair::with_buffer_size(512);

        let result = processor.process(reader, &mut output, "json");
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    #[test]
    fn test_streaming_markdown_repair() {
        let input = "# Header\n\nSome content\n\n## Subheader";

        let reader = Cursor::new(input);
        let mut output = Vec::new();
        let processor = StreamingRepair::new();

        let result = processor.process(reader, &mut output, "markdown");
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Header"));
    }

    #[test]
    fn test_streaming_auto_detect() {
        let input = r#"{"test": "value",}"#;

        let reader = Cursor::new(input);
        let mut output = Vec::new();
        let processor = StreamingRepair::new();

        // Use "auto" to trigger auto-detection
        let result = processor.process(reader, &mut output, "auto");
        assert!(result.is_ok());
    }

    #[test]
    fn test_streaming_empty_input() {
        let input = "";

        let reader = Cursor::new(input);
        let mut output = Vec::new();
        let processor = StreamingRepair::new();

        let result = processor.process(reader, &mut output, "json");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_streaming_csv_repair() {
        let input = "name,age,city\nJohn,30,NYC\nJane,25,LA";

        let reader = Cursor::new(input);
        let mut output = Vec::new();
        let processor = StreamingRepair::new();

        let result = processor.process(reader, &mut output, "csv");
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("name"));
    }
}
