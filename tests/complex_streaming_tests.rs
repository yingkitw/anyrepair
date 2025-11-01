//! Complex streaming repair tests for large files with multiple damage patterns
//! Tests realistic scenarios with streaming processing and complex damage

use anyrepair::StreamingRepair;
use std::io::Cursor;

#[test]
fn test_streaming_complex_json_large_nested_structure() {
    // Simulate a large JSON file with deeply nested structures and multiple errors
    let mut input = String::from("{\n  \"data\": [\n");
    
    for i in 0..50 {
        input.push_str(&format!(
            r#"    {{
      "id": {},
      "user": {{
        "name": "User{}",
        "email": "user{}@example.com",
        "profile": {{
          "age": {},
          "active": true,
          "tags": ["tag1", "tag2",],
        }},
      }},
      "metadata": {{
        "created": "2024-01-01",
        "updated": "2024-01-15",
      }},
    }}"#,
            i, i, i, 20 + i
        ));
        
        if i < 49 {
            input.push(',');
        }
        input.push('\n');
    }
    
    input.push_str("  ]\n}");

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(2048);

    let result = processor.process(reader, &mut output, "json");
    assert!(result.is_ok());
    
    let bytes = result.unwrap();
    assert!(bytes > 0);
    
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("User0"));
    assert!(output_str.contains("User49"));
}

#[test]
fn test_streaming_complex_yaml_large_config_with_errors() {
    // Simulate a large YAML config file with indentation and formatting errors
    let mut input = String::from("application:\n  name: MyApp\n  version: 1.0.0\n  services:\n");
    
    for i in 0..30 {
        input.push_str(&format!(
            "    - name: service{}\n      port: {}\n      endpoints:\n",
            i,
            8000 + i
        ));
        
        for j in 0..3 {
            input.push_str(&format!("        - /endpoint{}{}\n", i, j));
        }
        
        input.push_str(&format!(
            "      config:\n        timeout: {}\n        retries: 3\n",
            30 + i
        ));
    }

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(4096);

    let result = processor.process(reader, &mut output, "yaml");
    assert!(result.is_ok());
    
    let bytes = result.unwrap();
    assert!(bytes > 0);
    
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("service0"));
    assert!(output_str.contains("service29"));
}

#[test]
fn test_streaming_complex_markdown_large_document() {
    // Simulate a large markdown document with various formatting issues
    let mut input = String::from("# Main Documentation\n\n");
    
    for section in 0..20 {
        input.push_str(&format!("## Section {}\n\n", section));
        input.push_str(&format!("This is section {} with some content.\n\n", section));
        
        input.push_str("### Subsection\n\n");
        input.push_str("- Item 1\n");
        input.push_str("- Item 2\n");
        input.push_str("- Item 3\n\n");
        
        input.push_str("```rust\n");
        input.push_str("fn example() {\n");
        input.push_str("    println!(\"Section {}\");\n", );
        input.push_str("}\n");
        input.push_str("```\n\n");
    }

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(3072);

    let result = processor.process(reader, &mut output, "markdown");
    assert!(result.is_ok());
    
    let bytes = result.unwrap();
    assert!(bytes > 0);
    
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Main Documentation"));
    assert!(output_str.contains("Section 0"));
    assert!(output_str.contains("Section 19"));
}

#[test]
fn test_streaming_complex_csv_large_dataset() {
    // Simulate a large CSV with quoted fields and special characters
    let mut input = String::from("id,name,email,phone,address,notes\n");
    
    for i in 0..100 {
        input.push_str(&format!(
            r#"{},"{}, User{}","user{}@example.com","+1-555-{:04}","123 Main St, Apt {}, City, ST {}","Note for user {}""#,
            i,
            "User",
            i,
            i,
            i,
            i,
            10000 + i,
            i
        ));
        input.push('\n');
    }

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(2048);

    let result = processor.process(reader, &mut output, "csv");
    assert!(result.is_ok());
    
    let bytes = result.unwrap();
    assert!(bytes > 0);
}

#[test]
fn test_streaming_complex_xml_large_nested() {
    // Simulate a large XML with deeply nested structures
    let mut input = String::from("<?xml version=\"1.0\"?>\n<root>\n");
    
    for i in 0..25 {
        input.push_str(&format!("  <item id=\"{}\">\n", i));
        input.push_str(&format!("    <name>Item {}</name>\n", i));
        input.push_str(&format!("    <value>{}</value>\n", i * 100));
        input.push_str("    <properties>\n");
        
        for j in 0..3 {
            input.push_str(&format!(
                "      <property key=\"prop{}\" value=\"val{}\"/>\n",
                j, i * 10 + j
            ));
        }
        
        input.push_str("    </properties>\n");
        input.push_str("  </item>\n");
    }
    
    input.push_str("</root>");

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(2048);

    let result = processor.process(reader, &mut output, "xml");
    assert!(result.is_ok());
    
    let bytes = result.unwrap();
    assert!(bytes > 0);
    
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Item 0"));
    assert!(output_str.contains("Item 24"));
}

#[test]
fn test_streaming_complex_toml_large_config() {
    // Simulate a large TOML configuration file
    let mut input = String::from("[package]\nname = \"myapp\"\n\n");
    
    for i in 0..20 {
        input.push_str(&format!("[section{}]\n", i));
        input.push_str(&format!("name = \"Section {}\"\n", i));
        input.push_str(&format!("value = {}\n", i * 100));
        input.push_str(&format!("enabled = {}\n", if i % 2 == 0 { "true" } else { "false" }));
        input.push_str(&format!("items = [\"item1\", \"item2\", \"item3\"]\n\n"));
    }

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(2048);

    let result = processor.process(reader, &mut output, "toml");
    assert!(result.is_ok());
    
    let bytes = result.unwrap();
    assert!(bytes > 0);
}

#[test]
fn test_streaming_complex_ini_large_config() {
    // Simulate a large INI configuration file
    let mut input = String::from("; Configuration File\n\n");
    
    for i in 0..30 {
        input.push_str(&format!("[section{}]\n", i));
        input.push_str(&format!("key1 = value{}\n", i));
        input.push_str(&format!("key2 = {}\n", i * 100));
        input.push_str(&format!("key3 = enabled\n\n"));
    }

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(2048);

    let result = processor.process(reader, &mut output, "ini");
    assert!(result.is_ok());
    
    let bytes = result.unwrap();
    assert!(bytes > 0);
}

#[test]
fn test_streaming_very_small_buffer_with_complex_json() {
    // Test with very small buffer to force many iterations
    let mut input = String::from("[\n");
    
    for i in 0..100 {
        input.push_str(&format!(r#"{{"id": {}, "value": "item{}",}}"#, i, i));
        if i < 99 {
            input.push(',');
        }
        input.push('\n');
    }
    input.push(']');

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(256); // Very small buffer

    let result = processor.process(reader, &mut output, "json");
    assert!(result.is_ok());
    assert!(result.unwrap() > 0);
}

#[test]
fn test_streaming_large_buffer_with_complex_yaml() {
    // Test with large buffer for efficiency
    let mut input = String::from("data:\n");
    
    for i in 0..50 {
        input.push_str(&format!("  item{}:\n", i));
        input.push_str(&format!("    id: {}\n", i));
        input.push_str(&format!("    name: Item {}\n", i));
        input.push_str(&format!("    values: [1, 2, 3]\n"));
    }

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(65536); // 64KB buffer

    let result = processor.process(reader, &mut output, "yaml");
    assert!(result.is_ok());
    assert!(result.unwrap() > 0);
}

#[test]
fn test_streaming_mixed_damage_json_large() {
    // Complex JSON with multiple damage types
    let mut input = String::from("{\n  \"users\": [\n");
    
    for i in 0..40 {
        input.push_str(&format!(
            r#"    {{
      "id": {},
      "name": "User{}",
      "email": 'user{}@example.com',
      "active": true,
      "roles": ["admin", "user",],
      "settings": {{
        "notifications": true,
        "theme": "dark",
      }},
    }}"#,
            i, i, i
        ));
        
        if i < 39 {
            input.push(',');
        }
        input.push('\n');
    }
    
    input.push_str("  ]\n}");

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(3072);

    let result = processor.process(reader, &mut output, "json");
    assert!(result.is_ok());
    
    let bytes = result.unwrap();
    assert!(bytes > 0);
    
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("User0"));
    assert!(output_str.contains("User39"));
}

#[test]
fn test_streaming_performance_many_small_chunks() {
    // Test performance with many small chunks
    let mut input = String::new();
    
    for i in 0..500 {
        input.push_str(&format!("line {}: content\n", i));
    }

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(512);

    let result = processor.process(reader, &mut output, "markdown");
    assert!(result.is_ok());
    assert!(result.unwrap() > 0);
}

#[test]
fn test_streaming_auto_detect_large_json() {
    // Test auto-detection with large JSON
    let mut input = String::from("{\n");
    
    for i in 0..30 {
        input.push_str(&format!(r#"  "key{}": "value{}","#, i, i));
        input.push('\n');
    }
    
    input.push('}');

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "auto");
    assert!(result.is_ok());
    assert!(result.unwrap() > 0);
}

#[test]
fn test_streaming_auto_detect_large_yaml() {
    // Test auto-detection with large YAML
    let mut input = String::from("config:\n");
    
    for i in 0..30 {
        input.push_str(&format!("  key{}: value{}\n", i, i));
    }

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::new();

    let result = processor.process(reader, &mut output, "auto");
    assert!(result.is_ok());
    assert!(result.unwrap() > 0);
}

#[test]
fn test_streaming_unicode_large_json() {
    // Test streaming with unicode content
    let mut input = String::from("{\n");
    
    let languages = vec!["Hello", "世界", "مرحبا", "Привет", "🚀"];
    
    for (i, lang) in languages.iter().enumerate() {
        for j in 0..10 {
            input.push_str(&format!(r#"  "item{}_{}": "{}","#, i, j, lang));
            input.push('\n');
        }
    }
    
    input.push('}');

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(2048);

    let result = processor.process(reader, &mut output, "json");
    assert!(result.is_ok());
    assert!(result.unwrap() > 0);
}

#[test]
fn test_streaming_multiline_content_large_markdown() {
    // Test streaming with multiline content
    let mut input = String::from("# Documentation\n\n");
    
    for i in 0..15 {
        input.push_str(&format!("## Section {}\n\n", i));
        input.push_str("This is a multi-line\n");
        input.push_str("paragraph that spans\n");
        input.push_str("multiple lines.\n\n");
        
        input.push_str("```\n");
        input.push_str("code block\n");
        input.push_str("```\n\n");
    }

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(2048);

    let result = processor.process(reader, &mut output, "markdown");
    assert!(result.is_ok());
    assert!(result.unwrap() > 0);
}

#[test]
fn test_streaming_complex_csv_with_escaping() {
    // Test streaming CSV with complex escaping
    let mut input = String::from("id,name,description\n");
    
    for i in 0..50 {
        input.push_str(&format!(
            r#"{},"{}, User{}","Description with ""quotes"" and, commas""#,
            i, "User", i
        ));
        input.push('\n');
    }

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(2048);

    let result = processor.process(reader, &mut output, "csv");
    assert!(result.is_ok());
    assert!(result.unwrap() > 0);
}

#[test]
fn test_streaming_buffer_boundary_alignment() {
    // Test that buffer boundaries don't break repairs
    let input = r#"[
{"id": 1, "data": "value1",},
{"id": 2, "data": "value2",},
{"id": 3, "data": "value3",},
{"id": 4, "data": "value4",},
{"id": 5, "data": "value5",}
]"#;

    // Test with buffer size that might split at awkward places
    for buffer_size in &[64, 128, 256, 512, 1024] {
        let reader = Cursor::new(input);
        let mut output = Vec::new();
        let processor = StreamingRepair::with_buffer_size(*buffer_size);

        let result = processor.process(reader, &mut output, "json");
        assert!(result.is_ok(), "Failed with buffer size {}", buffer_size);
    }
}

#[test]
fn test_streaming_large_nested_xml() {
    // Test streaming with deeply nested XML
    let mut input = String::from("<?xml version=\"1.0\"?>\n<root>\n");
    
    for i in 0..20 {
        input.push_str(&format!("  <level1 id=\"{}\">\n", i));
        
        for j in 0..3 {
            input.push_str(&format!("    <level2 id=\"{}.{}\">\n", i, j));
            
            for k in 0..2 {
                input.push_str(&format!(
                    "      <level3 id=\"{}.{}.{}\">Value</level3>\n",
                    i, j, k
                ));
            }
            
            input.push_str("    </level2>\n");
        }
        
        input.push_str("  </level1>\n");
    }
    
    input.push_str("</root>");

    let reader = Cursor::new(input);
    let mut output = Vec::new();
    let processor = StreamingRepair::with_buffer_size(2048);

    let result = processor.process(reader, &mut output, "xml");
    assert!(result.is_ok());
    assert!(result.unwrap() > 0);
}
