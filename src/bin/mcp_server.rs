//! MCP server binary for anyrepair
//!
//! Runs anyrepair as an MCP server that can be integrated with Claude and other MCP clients

use anyrepair::AnyrepairMcpServer;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let server = AnyrepairMcpServer::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock();

    // Send server info
    let server_info = json!({
        "name": "anyrepair",
        "version": "0.1.5",
        "description": "MCP server for repairing LLM responses in various formats"
    });
    writeln!(stdout, "{}", server_info)?;
    stdout.flush()?;

    // Send available tools
    let tools = server.get_tools();
    for tool in tools {
        let tool_json = json!({
            "type": "tool",
            "name": tool.name,
            "description": tool.description,
            "inputSchema": tool.input_schema
        });
        writeln!(stdout, "{}", tool_json)?;
        stdout.flush()?;
    }

    // Process requests
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                if let Ok(request) = serde_json::from_str::<Value>(&line) {
                    if let Some(tool_name) = request.get("tool").and_then(|v| v.as_str()) {
                        if let Some(input) = request.get("input") {
                            match server.process_tool_call(tool_name, input) {
                                Ok(result) => {
                                    let response = json!({
                                        "type": "result",
                                        "tool": tool_name,
                                        "result": result
                                    });
                                    writeln!(stdout, "{}", response)?;
                                }
                                Err(error) => {
                                    let response = json!({
                                        "type": "error",
                                        "tool": tool_name,
                                        "error": error
                                    });
                                    writeln!(stdout, "{}", response)?;
                                }
                            }
                        }
                    }
                    stdout.flush()?;
                }
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}
