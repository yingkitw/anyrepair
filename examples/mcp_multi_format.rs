//! Example: Using AnyRepair MCP server with multiple formats
//!
//! Run with: cargo run --example mcp_multi_format

use anyrepair::json_util::{get_json_string_field, tool_input_json, validate_input_json};
use anyrepair::AnyrepairMcpServer;

fn main() {
    println!("=== AnyRepair MCP Server - Multi-Format Example ===\n");

    let server = AnyrepairMcpServer::new();

    println!("Example 1: Repair YAML");
    let malformed_yaml = "name: Alice\n  age: 30\n  city: New York";
    println!("Input:\n{}\n", malformed_yaml);
    if let Ok(result) = server.process_tool_call("repair_yaml", &tool_input_json(malformed_yaml)) {
        if let Some(out) = get_json_string_field(&result, "repaired") {
            println!("Output: {}\n", out);
        }
    }

    println!("Example 2: Repair Markdown");
    let malformed_markdown = "#Header\n##Subheader\nSome content";
    println!("Input:\n{}\n", malformed_markdown);
    if let Ok(result) =
        server.process_tool_call("repair_markdown", &tool_input_json(malformed_markdown))
    {
        if let Some(out) = get_json_string_field(&result, "repaired") {
            println!("Output:\n{}\n", out);
        }
    }

    println!("Example 3: Repair XML");
    let malformed_xml = "<root><item>value</root>";
    println!("Input: {}\n", malformed_xml);
    if let Ok(result) = server.process_tool_call("repair_xml", &tool_input_json(malformed_xml)) {
        if let Some(out) = get_json_string_field(&result, "repaired") {
            println!("Output: {}\n", out);
        }
    }

    println!("Example 4: Repair TOML");
    let toml_content = "name = \"myapp\"\nversion = \"1.0\"\n[database]\nhost = \"localhost\"";
    println!("Input:\n{}\n", toml_content);
    if let Ok(result) = server.process_tool_call("repair_toml", &tool_input_json(toml_content)) {
        if let Some(out) = get_json_string_field(&result, "repaired") {
            println!("Output:\n{}\n", out);
        }
    }

    println!("Example 5: Repair CSV");
    let csv_content = "name,age,city\nAlice,30,New York\nBob,25,San Francisco";
    println!("Input:\n{}\n", csv_content);
    if let Ok(result) = server.process_tool_call("repair_csv", &tool_input_json(csv_content)) {
        if let Some(out) = get_json_string_field(&result, "repaired") {
            println!("Output:\n{}\n", out);
        }
    }

    println!("Example 6: Repair INI");
    let ini_content = "[section1]\nkey1=value1\n[section2]\nkey2=value2";
    println!("Input:\n{}\n", ini_content);
    if let Ok(result) = server.process_tool_call("repair_ini", &tool_input_json(ini_content)) {
        if let Some(out) = get_json_string_field(&result, "repaired") {
            println!("Output: {}\n", out);
        }
    }

    println!("Example 7: Validate multiple formats");
    for (format, content) in [
        ("json", r#"{"key": "value"}"#),
        ("yaml", "key: value"),
        ("markdown", "# Header"),
        ("xml", "<root></root>"),
        ("toml", "key = \"value\""),
        ("csv", "name,age\nAlice,30"),
        ("ini", "[section]\nkey=value"),
    ] {
        if let Ok(result) =
            server.process_tool_call("validate", &validate_input_json(content, format))
        {
            let valid = result.contains(r#""valid":true"#);
            println!("{}: {}", format, if valid { "✓ Valid" } else { "✗ Invalid" });
        }
    }

    println!("\n=== Examples completed ===");
}
