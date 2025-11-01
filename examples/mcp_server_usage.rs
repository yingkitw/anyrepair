//! Example: MCP Server usage pattern
//!
//! This example demonstrates how to use the AnyRepair MCP server
//! in a real-world scenario with error handling and response parsing.
//!
//! Run with: cargo run --example mcp_server_usage

use anyrepair::AnyrepairMcpServer;
use serde_json::{json, Value};

/// Helper function to repair content and extract result
fn repair_content(server: &AnyrepairMcpServer, format: &str, content: &str) -> Result<String, String> {
    let tool_name = format!("repair_{}", format);
    let input = json!({
        "content": content
    });

    match server.process_tool_call(&tool_name, &input) {
        Ok(result) => {
            // Parse the JSON response
            match serde_json::from_str::<Value>(&result) {
                Ok(parsed) => {
                    if let Some(repaired) = parsed.get("repaired").and_then(|v| v.as_str()) {
                        Ok(repaired.to_string())
                    } else {
                        Err("No repaired content in response".to_string())
                    }
                }
                Err(e) => Err(format!("Failed to parse response: {}", e)),
            }
        }
        Err(e) => Err(e),
    }
}

/// Helper function to validate content
fn validate_content(server: &AnyrepairMcpServer, format: &str, content: &str) -> Result<bool, String> {
    let input = json!({
        "content": content,
        "format": format
    });

    match server.process_tool_call("validate", &input) {
        Ok(result) => {
            match serde_json::from_str::<Value>(&result) {
                Ok(parsed) => {
                    if let Some(valid) = parsed.get("valid").and_then(|v| v.as_bool()) {
                        Ok(valid)
                    } else {
                        Err("No valid field in response".to_string())
                    }
                }
                Err(e) => Err(format!("Failed to parse response: {}", e)),
            }
        }
        Err(e) => Err(e),
    }
}

fn main() {
    println!("=== AnyRepair MCP Server - Usage Pattern Example ===\n");

    let server = AnyrepairMcpServer::new();

    // Scenario 1: Repair and validate JSON
    println!("Scenario 1: Repair and validate JSON");
    println!("------------------------------------");

    let malformed_json = r#"{"name": "Alice", "age": 30,}"#;
    println!("Original: {}", malformed_json);

    match repair_content(&server, "json", malformed_json) {
        Ok(repaired) => {
            println!("Repaired: {}", repaired);

            // Validate the repaired content
            match validate_content(&server, "json", &repaired) {
                Ok(valid) => {
                    println!("Valid: {}\n", valid);
                }
                Err(e) => eprintln!("Validation error: {}\n", e),
            }
        }
        Err(e) => eprintln!("Repair error: {}\n", e),
    }

    // Scenario 2: Batch repair multiple items
    println!("Scenario 2: Batch repair multiple JSON items");
    println!("--------------------------------------------");

    let items = vec![
        r#"{"id": 1, "name": "Item1",}"#,
        r#"{'id': 2, 'name': 'Item2'}"#,
        r#"{id: 3, name: Item3}"#,
    ];

    for (i, item) in items.iter().enumerate() {
        println!("Item {}: {}", i + 1, item);
        match repair_content(&server, "json", item) {
            Ok(repaired) => println!("  → {}", repaired),
            Err(e) => eprintln!("  → Error: {}", e),
        }
    }
    println!();

    // Scenario 3: Format detection and repair
    println!("Scenario 3: Format detection and repair");
    println!("---------------------------------------");

    let mixed_content = vec![
        ("JSON", r#"{"key": "value",}"#),
        ("YAML", "name: John\n  age: 30"),
        ("Markdown", "#Header\nContent"),
    ];

    for (format, content) in mixed_content {
        println!("{}: {}", format, content);
        let input = json!({
            "content": content
        });

        match server.process_tool_call("repair", &input) {
            Ok(result) => {
                if let Ok(parsed) = serde_json::from_str::<Value>(&result) {
                    if let Some(repaired) = parsed.get("repaired").and_then(|v| v.as_str()) {
                        println!("  → {}", repaired);
                    }
                }
            }
            Err(e) => eprintln!("  → Error: {}", e),
        }
    }
    println!();

    // Scenario 4: Error handling
    println!("Scenario 4: Error handling");
    println!("---------------------------");

    // Missing content parameter
    let input = json!({});
    match server.process_tool_call("repair_json", &input) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error (missing content): {}", e),
    }

    // Unknown tool
    let input = json!({
        "content": "test"
    });
    match server.process_tool_call("unknown_tool", &input) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error (unknown tool): {}", e),
    }

    // Invalid format for validation
    let input = json!({
        "content": "test",
        "format": "invalid_format"
    });
    match server.process_tool_call("validate", &input) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error (invalid format): {}", e),
    }
    println!();

    // Scenario 5: Get available tools
    println!("Scenario 5: Available tools");
    println!("----------------------------");

    let tools = server.get_tools();
    println!("Total tools: {}", tools.len());
    for tool in tools {
        println!("  - {}: {}", tool.name, tool.description);
    }

    println!("\n=== Example completed ===");
}
