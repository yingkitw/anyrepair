//! Example: MCP Server usage pattern
//!
//! Run with: cargo run --example mcp_server_usage

use anyrepair::json_util::{
    get_json_string_field, tool_input_json, validate_input_json,
};
use anyrepair::AnyrepairMcpServer;

fn repair_content(server: &AnyrepairMcpServer, format: &str, content: &str) -> Result<String, String> {
    let tool_name = format!("repair_{}", format);
    let result = server.process_tool_call(&tool_name, &tool_input_json(content))?;
    get_json_string_field(&result, "repaired")
        .ok_or_else(|| "No repaired content in response".to_string())
}

fn validate_content(server: &AnyrepairMcpServer, format: &str, content: &str) -> Result<bool, String> {
    let result = server.process_tool_call("validate", &validate_input_json(content, format))?;
    Ok(result.contains(r#""valid":true"#))
}

fn main() {
    println!("=== AnyRepair MCP Server - Usage Pattern Example ===\n");

    let server = AnyrepairMcpServer::new();

    println!("Scenario 1: Repair and validate JSON");
    let malformed_json = r#"{"name": "Alice", "age": 30,}"#;
    println!("Original: {}", malformed_json);

    match repair_content(&server, "json", malformed_json) {
        Ok(repaired) => {
            println!("Repaired: {}", repaired);
            match validate_content(&server, "json", &repaired) {
                Ok(valid) => println!("Valid: {}\n", valid),
                Err(e) => eprintln!("Validation error: {}\n", e),
            }
        }
        Err(e) => eprintln!("Repair error: {}\n", e),
    }

    println!("Scenario 2: Batch repair multiple JSON items");
    for (i, item) in [
        r#"{"id": 1, "name": "Item1",}"#,
        r#"{'id': 2, 'name': 'Item2'}"#,
        r#"{id: 3, name: Item3}"#,
    ]
    .iter()
    .enumerate()
    {
        println!("Item {}: {}", i + 1, item);
        match repair_content(&server, "json", item) {
            Ok(repaired) => println!("  → {}", repaired),
            Err(e) => eprintln!("  → Error: {}", e),
        }
    }
    println!();

    println!("Scenario 3: Format detection and repair");
    for (format, content) in [
        ("JSON", r#"{"key": "value",}"#),
        ("YAML", "name: John\n  age: 30"),
        ("Markdown", "#Header\nContent"),
    ] {
        println!("{}: {}", format, content);
        match server.process_tool_call("repair", &tool_input_json(content)) {
            Ok(result) => {
                if let Some(repaired) = get_json_string_field(&result, "repaired") {
                    println!("  → {}", repaired);
                }
            }
            Err(e) => eprintln!("  → Error: {}", e),
        }
    }

    println!("\n=== Examples completed ===");
}
