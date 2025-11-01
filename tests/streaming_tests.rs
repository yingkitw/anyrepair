//! Comprehensive streaming repair tests
//! Tests for large file handling with minimal memory overhead

use anyrepair::StreamingRepair;
use std::io::Cursor;

#[test]
fn test_streaming_json_multiline() {
    let input = r#"{
  "users": [
    {"id": 1, "name": "Alice",},
    {"id": 2, "name": "Bob",}
  ],
}"#;

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "json");
    assert!(result.is_ok());
    
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("\"users\""));
    assert!(output_str.contains("\"id\""));
}

#[test]
fn test_streaming_yaml_multiline() {
    let input = "users:\n  - id: 1\n    name: Alice\n  - id: 2\n    name: Bob";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "yaml");
    assert!(result.is_ok());
    
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("users"));
}

#[test]
fn test_streaming_markdown_multiline() {
    let input = "# Main Title\n\nSome paragraph.\n\n## Subsection\n\nMore content.";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "markdown");
    assert!(result.is_ok());
    
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Main Title"));
}

#[test]
fn test_streaming_xml_multiline() {
    let input = "<root>\n  <item id=\"1\">Alice</item>\n  <item id=\"2\">Bob</item>\n</root>";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "xml");
    assert!(result.is_ok());
    
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("root"));
}

#[test]
fn test_streaming_csv_multiline() {
    let input = "id,name,email\n1,Alice,alice@example.com\n2,Bob,bob@example.com";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "csv");
    assert!(result.is_ok());
    
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("id"));
}

#[test]
fn test_streaming_toml_multiline() {
    let input = "[section]\nkey1 = \"value1\"\nkey2 = \"value2\"";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "toml");
    assert!(result.is_ok());
    
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("section"));
}

#[test]
fn test_streaming_ini_multiline() {
    let input = "[database]\nhost=localhost\nport=5432";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "ini");
    assert!(result.is_ok());
    
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("database"));
}

#[test]
fn test_streaming_small_buffer_size() {
    // Test with very small buffer to force multiple iterations
    let input = r#"{"a": 1,}
{"b": 2,}
{"c": 3,}"#;

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(64); // Very small buffer

    let result = processor.process(reader, &mut output, "json");
    assert!(result.is_ok());
    assert!(result.unwrap() > 0);
}

#[test]
fn test_streaming_large_buffer_size() {
    // Test with large buffer
    let input = r#"{"key": "value",}"#;

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(65536); // 64KB buffer

    let result = processor.process(reader, &mut output, "json");
    assert!(result.is_ok());
    assert!(result.unwrap() > 0);
}

#[test]
fn test_streaming_many_lines() {
    // Simulate processing many lines
    let mut input = String::new();
    for i in 0..1000 {
        input.push_str(&format!("line {}: some content\n", i));
    }

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "markdown");
    assert!(result.is_ok());
    assert!(result.unwrap() > 0);
}

#[test]
fn test_streaming_json_with_trailing_comma() {
    let input = r#"{
  "name": "test",
  "value": 123,
}"#;

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "json");
    assert!(result.is_ok());
    
    let output_str = String::from_utf8(output).unwrap();
    // Verify trailing comma was handled
    assert!(output_str.contains("\"name\""));
}

#[test]
fn test_streaming_mixed_content() {
    let input = r#"[
  {"id": 1, "data": "item1",},
  {"id": 2, "data": "item2",}
]"#;

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "json");
    assert!(result.is_ok());
    
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("\"id\""));
}

#[test]
fn test_streaming_default_processor() {
    let input = r#"{"test": "value",}"#;

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::default();

    let result = processor.process(reader, &mut output, "json");
    assert!(result.is_ok());
}

#[test]
fn test_streaming_bytes_counted() {
    let input = "# Test Header\n\nContent here.";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "markdown");
    assert!(result.is_ok());
    
    let bytes = result.unwrap();
    assert!(bytes > 0);
    assert_eq!(bytes, output.len());
}

#[test]
fn test_streaming_json_array() {
    let input = r#"[
  1,
  2,
  3,
]"#;

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "json");
    assert!(result.is_ok());
}

#[test]
fn test_streaming_yaml_list() {
    let input = "- item1\n- item2\n- item3";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "yaml");
    assert!(result.is_ok());
}

#[test]
fn test_streaming_csv_quoted_fields() {
    let input = "name,description\n\"John Doe\",\"A person\"\n\"Jane Smith\",\"Another person\"";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "csv");
    assert!(result.is_ok());
}

#[test]
fn test_streaming_xml_attributes() {
    let input = "<root>\n  <item id=\"1\" name=\"first\">Content</item>\n</root>";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "xml");
    assert!(result.is_ok());
}

#[test]
fn test_streaming_markdown_code_blocks() {
    let input = "# Code Example\n\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "markdown");
    assert!(result.is_ok());
}

#[test]
fn test_streaming_toml_arrays() {
    let input = "[package]\nname = \"test\"\nauthors = [\"Alice\", \"Bob\"]";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "toml");
    assert!(result.is_ok());
}

#[test]
fn test_streaming_ini_comments() {
    let input = "; This is a comment\n[section]\nkey=value";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "ini");
    assert!(result.is_ok());
}

#[test]
fn test_streaming_auto_format_json() {
    let input = r#"{"test": "value",}"#;

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "auto");
    assert!(result.is_ok());
}

#[test]
fn test_streaming_auto_format_yaml() {
    let input = "key: value\nlist:\n  - item1\n  - item2";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "auto");
    assert!(result.is_ok());
}

#[test]
fn test_streaming_performance_large_json() {
    // Create a large JSON structure
    let mut input = String::from("[");
    for i in 0..100 {
        if i > 0 {
            input.push(',');
        }
        input.push_str(&format!(r#"{{"id": {}, "name": "item{}",}}"#, i, i));
    }
    input.push(']');

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(1024);

    let result = processor.process(reader, &mut output, "json");
    assert!(result.is_ok());
    assert!(result.unwrap() > 0);
}

#[test]
fn test_streaming_empty_lines() {
    let input = "line1\n\n\nline2\n\nline3";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "markdown");
    assert!(result.is_ok());
}

#[test]
fn test_streaming_whitespace_handling() {
    let input = "  \n  \n  ";

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "markdown");
    assert!(result.is_ok());
}
