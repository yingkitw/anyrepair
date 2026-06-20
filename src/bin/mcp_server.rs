//! MCP server binary for anyrepair
//!
//! Runs anyrepair as an MCP server that can be integrated with Claude and other MCP clients

use anyrepair::json_util::{json_string, parse_mcp_request_line};
use anyrepair::AnyrepairMcpServer;
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let server = AnyrepairMcpServer::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock();

    let server_info = format!(
        r#"{{"name":"anyrepair","version":"0.2.6","description":"MCP server for repairing malformed structured data"}}"#
    );
    writeln!(stdout, "{}", server_info)?;
    stdout.flush()?;

    for tool in server.get_tools() {
        let tool_json = format!(
            r#"{{"type":"tool","name":{},"description":{},"inputSchema":{}}}"#,
            json_string(&tool.name),
            json_string(&tool.description),
            tool.input_schema
        );
        writeln!(stdout, "{}", tool_json)?;
        stdout.flush()?;
    }

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if let Ok((tool_name, input_json)) = parse_mcp_request_line(&line) {
                    match server.process_tool_call(&tool_name, &input_json) {
                        Ok(result) => {
                            let response = format!(
                                r#"{{"type":"result","tool":{},"result":{}}}"#,
                                json_string(&tool_name),
                                result
                            );
                            writeln!(stdout, "{}", response)?;
                        }
                        Err(error) => {
                            let response = format!(
                                r#"{{"type":"error","tool":{},"error":{}}}"#,
                                json_string(&tool_name),
                                json_string(&error)
                            );
                            writeln!(stdout, "{}", response)?;
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
