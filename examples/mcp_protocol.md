# MCP Server Protocol Examples

This document shows how to interact with the AnyRepair MCP server using the stdin/stdout protocol.

## Running the MCP Server

```bash
cargo build --bin anyrepair-mcp --release
./target/release/anyrepair-mcp
```

The server reads JSON requests from stdin and writes JSON responses to stdout.

## Protocol Format

### Request Format

```json
{
  "tool": "tool_name",
  "input": {
    "content": "content to repair",
    "format": "format_name"  // optional for some tools
  }
}
```

### Response Format

**Success:**
```json
{
  "type": "result",
  "tool": "tool_name",
  "result": "{...}"
}
```

**Error:**
```json
{
  "type": "error",
  "tool": "tool_name",
  "error": "error message"
}
```

## Examples

### Example 1: Repair JSON with Trailing Comma

**Request:**
```bash
echo '{"tool": "repair_json", "input": {"content": "{\"key\": \"value\",}"}}' | ./target/release/anyrepair-mcp
```

**Response:**
```json
{
  "type": "result",
  "tool": "repair_json",
  "result": "{\"repaired\": \"{\\\"key\\\": \\\"value\\\"}\", \"confidence\": 0.95, \"success\": true}"
}
```

### Example 2: Validate YAML

**Request:**
```bash
echo '{"tool": "validate", "input": {"content": "name: John\\nage: 30", "format": "yaml"}}' | ./target/release/anyrepair-mcp
```

**Response:**
```json
{
  "type": "result",
  "tool": "validate",
  "result": "{\"valid\": true, \"format\": \"yaml\"}"
}
```

### Example 3: Repair Markdown Headers

**Request:**
```bash
echo '{"tool": "repair_markdown", "input": {"content": "#Header\\n##Subheader"}}' | ./target/release/anyrepair-mcp
```

**Response:**
```json
{
  "type": "result",
  "tool": "repair_markdown",
  "result": "{\"repaired\": \"# Header\\n## Subheader\", \"confidence\": 0.9, \"success\": true}"
}
```

### Example 4: Auto-Detect and Repair

**Request:**
```bash
echo '{"tool": "repair", "input": {"content": "[1, 2, 3,]"}}' | ./target/release/anyrepair-mcp
```

**Response:**
```json
{
  "type": "result",
  "tool": "repair",
  "result": "{\"repaired\": \"[1, 2, 3]\", \"success\": true}"
}
```

### Example 5: Error - Unknown Tool

**Request:**
```bash
echo '{"tool": "unknown_tool", "input": {"content": "test"}}' | ./target/release/anyrepair-mcp
```

**Response:**
```json
{
  "type": "error",
  "tool": "unknown_tool",
  "error": "Unknown tool: unknown_tool"
}
```

### Example 6: Error - Missing Parameter

**Request:**
```bash
echo '{"tool": "repair_json", "input": {}}' | ./target/release/anyrepair-mcp
```

**Response:**
```json
{
  "type": "error",
  "tool": "repair_json",
  "error": "Missing 'content' parameter"
}
```

## Available Tools

### Repair Tools

- `repair` - Auto-detect format and repair
- `repair_json` - Repair JSON
- `repair_yaml` - Repair YAML
- `repair_markdown` - Repair Markdown
- `repair_xml` - Repair XML
- `repair_toml` - Repair TOML
- `repair_csv` - Repair CSV
- `repair_ini` - Repair INI

### Validation Tool

- `validate` - Validate content in specified format

## Tool Parameters

### Repair Tools

**Input:**
```json
{
  "content": "string"  // Required: content to repair
}
```

**Output:**
```json
{
  "repaired": "string",      // Repaired content
  "confidence": 0.0-1.0,     // Confidence score (format-specific tools only)
  "success": true
}
```

### Validate Tool

**Input:**
```json
{
  "content": "string",       // Required: content to validate
  "format": "string"         // Required: format (json, yaml, markdown, xml, toml, csv, ini)
}
```

**Output:**
```json
{
  "valid": true/false,       // Whether content is valid
  "format": "string"         // Format that was validated
}
```

## Using with Python

```python
import subprocess
import json

def call_mcp_server(tool, input_data):
    request = {
        "tool": tool,
        "input": input_data
    }
    
    process = subprocess.Popen(
        ["./target/release/anyrepair-mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    
    stdout, stderr = process.communicate(
        json.dumps(request).encode()
    )
    
    return json.loads(stdout.decode())

# Example usage
result = call_mcp_server("repair_json", {
    "content": '{"key": "value",}'
})

print(result)
```

## Using with Node.js

```javascript
const { spawn } = require('child_process');

function callMcpServer(tool, inputData) {
  return new Promise((resolve, reject) => {
    const process = spawn('./target/release/anyrepair-mcp');
    
    let output = '';
    
    process.stdout.on('data', (data) => {
      output += data.toString();
    });
    
    process.on('close', (code) => {
      try {
        resolve(JSON.parse(output));
      } catch (e) {
        reject(e);
      }
    });
    
    process.stdin.write(JSON.stringify({
      tool: tool,
      input: inputData
    }));
    process.stdin.end();
  });
}

// Example usage
callMcpServer('repair_json', {
  content: '{"key": "value",}'
}).then(result => console.log(result));
```

## Using with cURL

```bash
# Create a request file
cat > request.json << 'EOF'
{"tool": "repair_json", "input": {"content": "{\"key\": \"value\",}"}}
EOF

# Send to server
cat request.json | ./target/release/anyrepair-mcp
```

## Integration with Claude

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "anyrepair": {
      "command": "/path/to/anyrepair-mcp"
    }
  }
}
```

Then Claude can call tools like:
- `anyrepair/repair_json`
- `anyrepair/repair_yaml`
- `anyrepair/validate`
- etc.

## Performance Tips

1. **Batch Operations**: Send multiple requests sequentially
2. **Large Files**: Use streaming repair for files > 1MB
3. **Format Detection**: Use auto-detect for unknown formats
4. **Error Handling**: Always check for error responses

## Troubleshooting

### Server not responding
- Check if the binary is built: `cargo build --bin anyrepair-mcp`
- Verify stdin/stdout are properly connected
- Check for encoding issues (UTF-8 required)

### Invalid JSON response
- Ensure request is valid JSON
- Check for special characters in content
- Verify parameter names are correct

### Tool not found
- Use lowercase tool names
- Check available tools list
- Verify format names (json, yaml, markdown, etc.)
