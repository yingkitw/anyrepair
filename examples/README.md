# AnyRepair Examples

This directory contains examples demonstrating how to use the AnyRepair library and MCP server.

## MCP Server Examples

### 1. JSON Repair Example

**File**: `mcp_repair_json.rs`

Demonstrates how to repair malformed JSON using the MCP server:
- Trailing commas
- Single quotes
- Missing quotes
- Validation

**Run:**
```bash
cargo run --example mcp_repair_json
```

**Output:**
```
=== AnyRepair MCP Server - JSON Repair Example ===

Example 1: Repair JSON with trailing comma
-------------------------------------------
Input:  {"name": "John", "age": 30,}
Output: {"repaired": "{\"name\": \"John\", \"age\": 30}", "confidence": 0.95, "success": true}
...
```

### 2. Multi-Format Example

**File**: `mcp_multi_format.rs`

Demonstrates repairing content in multiple formats:
- YAML
- Markdown
- XML
- TOML
- CSV
- INI
- Validation for all formats

**Run:**
```bash
cargo run --example mcp_multi_format
```

**Output:**
```
=== AnyRepair MCP Server - Multi-Format Example ===

Example 1: Repair YAML
----------------------
Input:
name: Alice
  age: 30
  city: New York

Output:
name: Alice
age: 30
city: New York
...
```

### 3. Server Usage Pattern Example

**File**: `mcp_server_usage.rs`

Demonstrates real-world usage patterns:
- Repair and validate workflow
- Batch repair operations
- Format detection
- Error handling
- Tool discovery

**Run:**
```bash
cargo run --example mcp_server_usage
```

**Output:**
```
=== AnyRepair MCP Server - Usage Pattern Example ===

Scenario 1: Repair and validate JSON
------------------------------------
Original: {"name": "Alice", "age": 30,}
Repaired: {"name": "Alice", "age": 30}
Valid: true
...
```

### 4. MCP Protocol Documentation

**File**: `mcp_protocol.md`

Complete reference for the MCP server protocol:
- Request/response format
- All available tools
- Parameter specifications
- Real-world examples
- Integration guides (Python, Node.js, cURL)
- Claude integration
- Troubleshooting

## Quick Start

### Using the Library Directly

```rust
use anyrepair::AnyrepairMcpServer;
use serde_json::json;

fn main() {
    let server = AnyrepairMcpServer::new();
    
    let input = json!({
        "content": r#"{"key": "value",}"#
    });
    
    match server.process_tool_call("repair_json", &input) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Running the MCP Server

```bash
# Build the MCP server binary
cargo build --bin anyrepair-mcp --release

# Run the server
./target/release/anyrepair-mcp

# In another terminal, send a request
echo '{"tool": "repair_json", "input": {"content": "{\"key\": \"value\",}"}}' | ./target/release/anyrepair-mcp
```

### Integration with Claude

```json
{
  "mcpServers": {
    "anyrepair": {
      "command": "/path/to/anyrepair-mcp"
    }
  }
}
```

## Available Tools

### Repair Tools

| Tool | Purpose |
|------|---------|
| `repair` | Auto-detect format and repair |
| `repair_json` | Repair JSON |
| `repair_yaml` | Repair YAML |
| `repair_markdown` | Repair Markdown |
| `repair_xml` | Repair XML |
| `repair_toml` | Repair TOML |
| `repair_csv` | Repair CSV |
| `repair_ini` | Repair INI |

### Validation Tool

| Tool | Purpose |
|------|---------|
| `validate` | Validate content in specified format |

## Example Scenarios

### Scenario 1: Repair LLM JSON Output

```rust
let malformed = r#"{"response": "Hello", "status": "ok",}"#;
let input = json!({"content": malformed});
server.process_tool_call("repair_json", &input)?;
```

### Scenario 2: Validate Configuration Files

```rust
let config = "name = \"app\"\nversion = \"1.0\"";
let input = json!({"content": config, "format": "toml"});
server.process_tool_call("validate", &input)?;
```

### Scenario 3: Batch Process Multiple Items

```rust
let items = vec![
    r#"{"id": 1,}"#,
    r#"{'id': 2}"#,
    r#"{id: 3}"#,
];

for item in items {
    let input = json!({"content": item});
    server.process_tool_call("repair_json", &input)?;
}
```

### Scenario 4: Auto-Detect and Repair

```rust
let content = "[1, 2, 3,]";  // Could be JSON or YAML
let input = json!({"content": content});
server.process_tool_call("repair", &input)?;  // Auto-detects format
```

## Error Handling

All examples include proper error handling:

```rust
match server.process_tool_call("repair_json", &input) {
    Ok(result) => {
        // Parse and use result
        let parsed: Value = serde_json::from_str(&result)?;
        println!("Repaired: {}", parsed["repaired"]);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Performance Tips

1. **Reuse Server Instance**: Create once, use multiple times
2. **Batch Operations**: Process multiple items sequentially
3. **Format Specificity**: Use specific repair tools when format is known
4. **Large Files**: Use streaming repair for files > 1MB

## Testing Examples

Run all examples:

```bash
cargo run --example mcp_repair_json
cargo run --example mcp_multi_format
cargo run --example mcp_server_usage
```

Run with output:

```bash
cargo run --example mcp_repair_json -- --nocapture
```

## Integration Examples

### Python Integration

```python
import subprocess
import json

def repair_json(content):
    result = subprocess.run(
        ["./target/release/anyrepair-mcp"],
        input=json.dumps({
            "tool": "repair_json",
            "input": {"content": content}
        }),
        capture_output=True,
        text=True
    )
    return json.loads(result.stdout)
```

### Node.js Integration

```javascript
const { spawn } = require('child_process');

function repairJson(content) {
  const process = spawn('./target/release/anyrepair-mcp');
  
  process.stdin.write(JSON.stringify({
    tool: 'repair_json',
    input: { content }
  }));
  
  return new Promise((resolve) => {
    process.stdout.on('data', (data) => {
      resolve(JSON.parse(data.toString()));
    });
  });
}
```

## Documentation

- [MCP Server Documentation](../MCP_SERVER.md)
- [MCP Protocol Reference](mcp_protocol.md)
- [Main README](../README.md)

## Support

For issues or questions:
1. Check the [MCP Server Documentation](../MCP_SERVER.md)
2. Review the [Protocol Reference](mcp_protocol.md)
3. Check the [Main README](../README.md)
4. Open an issue on GitHub

## License

Apache-2.0
