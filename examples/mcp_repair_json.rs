//! Example: Using AnyRepair MCP server to repair JSON
//!
//! This example demonstrates how to use the AnyRepair MCP server
//! to repair malformed JSON content.
//!
//! Run with: cargo run --example mcp_repair_json

use anyrepair::AnyrepairMcpServer;
use serde_json::json;

fn main() {
    println!("=== AnyRepair MCP Server - JSON Repair Example ===\n");

    // Create MCP server instance
    let server = AnyrepairMcpServer::new();

    // Example 1: Repair JSON with trailing comma
    println!("Example 1: Repair JSON with trailing comma");
    println!("-------------------------------------------");
    let malformed_json = r#"{"name": "John", "age": 30,}"#;
    println!("Input:  {}", malformed_json);

    let input = json!({
        "content": malformed_json
    });

    match server.process_tool_call("repair_json", &input) {
        Ok(result) => {
            println!("Output: {}\n", result);
        }
        Err(e) => {
            eprintln!("Error: {}\n", e);
        }
    }

    // Example 2: Repair JSON with single quotes
    println!("Example 2: Repair JSON with single quotes");
    println!("------------------------------------------");
    let single_quote_json = "{'name': 'Alice', 'age': 25}";
    println!("Input:  {}", single_quote_json);

    let input = json!({
        "content": single_quote_json
    });

    match server.process_tool_call("repair_json", &input) {
        Ok(result) => {
            println!("Output: {}\n", result);
        }
        Err(e) => {
            eprintln!("Error: {}\n", e);
        }
    }

    // Example 3: Repair JSON with missing quotes
    println!("Example 3: Repair JSON with missing quotes");
    println!("-------------------------------------------");
    let missing_quotes_json = "{name: Bob, age: 35}";
    println!("Input:  {}", missing_quotes_json);

    let input = json!({
        "content": missing_quotes_json
    });

    match server.process_tool_call("repair_json", &input) {
        Ok(result) => {
            println!("Output: {}\n", result);
        }
        Err(e) => {
            eprintln!("Error: {}\n", e);
        }
    }

    // Example 4: Validate JSON
    println!("Example 4: Validate JSON");
    println!("------------------------");
    let valid_json = r#"{"name": "Charlie", "age": 40}"#;
    println!("Input:  {}", valid_json);

    let input = json!({
        "content": valid_json,
        "format": "json"
    });

    match server.process_tool_call("validate", &input) {
        Ok(result) => {
            println!("Output: {}\n", result);
        }
        Err(e) => {
            eprintln!("Error: {}\n", e);
        }
    }

    // Example 5: Auto-detect and repair
    println!("Example 5: Auto-detect and repair");
    println!("----------------------------------");
    let array_json = "[1, 2, 3,]";
    println!("Input:  {}", array_json);

    let input = json!({
        "content": array_json
    });

    match server.process_tool_call("repair", &input) {
        Ok(result) => {
            println!("Output: {}\n", result);
        }
        Err(e) => {
            eprintln!("Error: {}\n", e);
        }
    }

    println!("=== Examples completed ===");
}
