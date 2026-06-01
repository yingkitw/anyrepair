//! Example: Using AnyRepair MCP server to repair JSON
//!
//! Run with: cargo run --example mcp_repair_json

use anyrepair::json_util::tool_input_json;
use anyrepair::json_util::validate_input_json;
use anyrepair::AnyrepairMcpServer;

fn main() {
    println!("=== AnyRepair MCP Server - JSON Repair Example ===\n");

    let server = AnyrepairMcpServer::new();

    println!("Example 1: Repair JSON with trailing comma");
    let malformed_json = r#"{"name": "John", "age": 30,}"#;
    println!("Input:  {}", malformed_json);
    match server.process_tool_call("repair_json", &tool_input_json(malformed_json)) {
        Ok(result) => println!("Output: {}\n", result),
        Err(e) => eprintln!("Error: {}\n", e),
    }

    println!("Example 2: Repair JSON with single quotes");
    let single_quote_json = "{'name': 'Alice', 'age': 25}";
    println!("Input:  {}", single_quote_json);
    match server.process_tool_call("repair_json", &tool_input_json(single_quote_json)) {
        Ok(result) => println!("Output: {}\n", result),
        Err(e) => eprintln!("Error: {}\n", e),
    }

    println!("Example 3: Repair JSON with missing quotes");
    let missing_quotes_json = "{name: Bob, age: 35}";
    println!("Input:  {}", missing_quotes_json);
    match server.process_tool_call("repair_json", &tool_input_json(missing_quotes_json)) {
        Ok(result) => println!("Output: {}\n", result),
        Err(e) => eprintln!("Error: {}\n", e),
    }

    println!("Example 4: Validate JSON");
    let valid_json = r#"{"name": "Charlie", "age": 40}"#;
    println!("Input:  {}", valid_json);
    match server.process_tool_call("validate", &validate_input_json(valid_json, "json")) {
        Ok(result) => println!("Output: {}\n", result),
        Err(e) => eprintln!("Error: {}\n", e),
    }

    println!("Example 5: Auto-detect and repair");
    let array_json = "[1, 2, 3,]";
    println!("Input:  {}", array_json);
    match server.process_tool_call("repair", &tool_input_json(array_json)) {
        Ok(result) => println!("Output: {}\n", result),
        Err(e) => eprintln!("Error: {}\n", e),
    }

    println!("=== Examples completed ===");
}
