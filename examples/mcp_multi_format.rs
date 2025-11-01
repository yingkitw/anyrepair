//! Example: Using AnyRepair MCP server with multiple formats
//!
//! This example demonstrates how to use the AnyRepair MCP server
//! to repair content in different formats: JSON, YAML, Markdown, XML, TOML, CSV, INI.
//!
//! Run with: cargo run --example mcp_multi_format

use anyrepair::AnyrepairMcpServer;
use serde_json::json;

fn main() {
    println!("=== AnyRepair MCP Server - Multi-Format Example ===\n");

    let server = AnyrepairMcpServer::new();

    // Example 1: Repair YAML
    println!("Example 1: Repair YAML");
    println!("----------------------");
    let malformed_yaml = "name: Alice\n  age: 30\n  city: New York";
    println!("Input:\n{}\n", malformed_yaml);

    let input = json!({
        "content": malformed_yaml
    });

    match server.process_tool_call("repair_yaml", &input) {
        Ok(result) => {
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            println!("Output: {}\n", parsed["repaired"]);
        }
        Err(e) => eprintln!("Error: {}\n", e),
    }

    // Example 2: Repair Markdown
    println!("Example 2: Repair Markdown");
    println!("---------------------------");
    let malformed_markdown = "#Header\n##Subheader\nSome content";
    println!("Input:\n{}\n", malformed_markdown);

    let input = json!({
        "content": malformed_markdown
    });

    match server.process_tool_call("repair_markdown", &input) {
        Ok(result) => {
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            println!("Output:\n{}\n", parsed["repaired"]);
        }
        Err(e) => eprintln!("Error: {}\n", e),
    }

    // Example 3: Repair XML
    println!("Example 3: Repair XML");
    println!("---------------------");
    let malformed_xml = "<root><item>value</root>";
    println!("Input: {}\n", malformed_xml);

    let input = json!({
        "content": malformed_xml
    });

    match server.process_tool_call("repair_xml", &input) {
        Ok(result) => {
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            println!("Output: {}\n", parsed["repaired"]);
        }
        Err(e) => eprintln!("Error: {}\n", e),
    }

    // Example 4: Repair TOML
    println!("Example 4: Repair TOML");
    println!("----------------------");
    let toml_content = "name = \"myapp\"\nversion = \"1.0\"\n[database]\nhost = \"localhost\"";
    println!("Input:\n{}\n", toml_content);

    let input = json!({
        "content": toml_content
    });

    match server.process_tool_call("repair_toml", &input) {
        Ok(result) => {
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            println!("Output:\n{}\n", parsed["repaired"]);
        }
        Err(e) => eprintln!("Error: {}\n", e),
    }

    // Example 5: Repair CSV
    println!("Example 5: Repair CSV");
    println!("---------------------");
    let csv_content = "name,age,city\nAlice,30,New York\nBob,25,San Francisco";
    println!("Input:\n{}\n", csv_content);

    let input = json!({
        "content": csv_content
    });

    match server.process_tool_call("repair_csv", &input) {
        Ok(result) => {
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            println!("Output:\n{}\n", parsed["repaired"]);
        }
        Err(e) => eprintln!("Error: {}\n", e),
    }

    // Example 6: Repair INI
    println!("Example 6: Repair INI");
    println!("---------------------");
    let ini_content = "[section1]\nkey1=value1\n[section2]\nkey2=value2";
    println!("Input:\n{}\n", ini_content);

    let input = json!({
        "content": ini_content
    });

    match server.process_tool_call("repair_ini", &input) {
        Ok(result) => {
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            println!("Output:\n{}\n", parsed["repaired"]);
        }
        Err(e) => eprintln!("Error: {}\n", e),
    }

    // Example 7: Validate multiple formats
    println!("Example 7: Validate multiple formats");
    println!("------------------------------------");

    let formats = vec![
        ("json", r#"{"key": "value"}"#),
        ("yaml", "key: value"),
        ("markdown", "# Header"),
        ("xml", "<root></root>"),
        ("toml", "key = \"value\""),
        ("csv", "name,age\nAlice,30"),
        ("ini", "[section]\nkey=value"),
    ];

    for (format, content) in formats {
        let input = json!({
            "content": content,
            "format": format
        });

        match server.process_tool_call("validate", &input) {
            Ok(result) => {
                let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
                let valid = parsed["valid"].as_bool().unwrap_or(false);
                println!("{}: {}", format, if valid { "✓ Valid" } else { "✗ Invalid" });
            }
            Err(e) => eprintln!("{}: Error - {}", format, e),
        }
    }

    println!("\n=== Examples completed ===");
}
