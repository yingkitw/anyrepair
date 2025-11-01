# AnyRepair MCP Server

## Overview

AnyRepair now exposes its repair functionality as an **MCP (Model Context Protocol)** server, enabling integration with Claude and other MCP-compatible clients.

## Features

- **Auto-detect repair**: Automatically detects format and repairs content
- **Format-specific repair**: Direct repair for JSON, YAML, Markdown, XML, TOML, CSV, INI
- **Validation**: Validate content in any supported format
- **Confidence scoring**: Get confidence scores for repairs
- **Streaming support**: Handle large files efficiently

## Installation

### From Source

```bash
cargo build --bin anyrepair-mcp --release
```

The binary will be available at `target/release/anyrepair-mcp`

### From Crates.io

```bash
cargo install anyrepair --bin anyrepair-mcp
```

## Usage

### Running the MCP Server

```bash
anyrepair-mcp
```

The server reads JSON-formatted requests from stdin and writes responses to stdout.

### Integration with Claude

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "anyrepair": {
      "command": "anyrepair-mcp"
    }
  }
}
```

## Available Tools

### 1. `repair` - Auto-detect and repair

Automatically detects the format and repairs content.

**Input:**
```json
{
  "tool": "repair",
  "input": {
    "content": "{'key': 'value',}"
  }
}
```

**Output:**
```json
{
  "type": "result",
  "tool": "repair",
  "result": "{\"repaired\": \"{\\\"key\\\": \\\"value\\\"}\", \"success\": true}"
}
```

### 2. Format-Specific Repair Tools

- `repair_json` - Repair JSON content
- `repair_yaml` - Repair YAML content
- `repair_markdown` - Repair Markdown content
- `repair_xml` - Repair XML content
- `repair_toml` - Repair TOML content
- `repair_csv` - Repair CSV content
- `repair_ini` - Repair INI content

**Input:**
```json
{
  "tool": "repair_json",
  "input": {
    "content": "{\"key\": \"value\",}"
  }
}
```

**Output:**
```json
{
  "type": "result",
  "tool": "repair_json",
  "result": "{\"repaired\": \"{\\\"key\\\": \\\"value\\\"}\", \"confidence\": 0.95, \"success\": true}"
}
```

### 3. `validate` - Validate content

Validates content in a specified format.

**Input:**
```json
{
  "tool": "validate",
  "input": {
    "content": "{\"key\": \"value\"}",
    "format": "json"
  }
}
```

**Output:**
```json
{
  "type": "result",
  "tool": "validate",
  "result": "{\"valid\": true, \"format\": \"json\"}"
}
```

## Supported Formats

| Format | Repair | Validate | Auto-detect |
|--------|--------|----------|-------------|
| JSON | ✅ | ✅ | ✅ |
| YAML | ✅ | ✅ | ✅ |
| Markdown | ✅ | ✅ | ✅ |
| XML | ✅ | ✅ | ✅ |
| TOML | ✅ | ✅ | ✅ |
| CSV | ✅ | ✅ | ✅ |
| INI | ✅ | ✅ | ✅ |

## Examples

### Example 1: Repair Malformed JSON

```json
{
  "tool": "repair_json",
  "input": {
    "content": "{\n  'name': 'John',\n  'age': 30,\n}"
  }
}
```

Response:
```json
{
  "type": "result",
  "tool": "repair_json",
  "result": "{\"repaired\": \"{\\n  \\\"name\\\": \\\"John\\\",\\n  \\\"age\\\": 30\\n}\", \"confidence\": 0.95, \"success\": true}"
}
```

### Example 2: Validate YAML

```json
{
  "tool": "validate",
  "input": {
    "content": "name: John\nage: 30",
    "format": "yaml"
  }
}
```

Response:
```json
{
  "type": "result",
  "tool": "validate",
  "result": "{\"valid\": true, \"format\": \"yaml\"}"
}
```

### Example 3: Auto-detect and Repair

```json
{
  "tool": "repair",
  "input": {
    "content": "[1, 2, 3,]"
  }
}
```

Response:
```json
{
  "type": "result",
  "tool": "repair",
  "result": "{\"repaired\": \"[1, 2, 3]\", \"success\": true}"
}
```

## Error Handling

When an error occurs, the server responds with an error message:

```json
{
  "type": "error",
  "tool": "repair_json",
  "error": "JSON repair failed: invalid input"
}
```

## Performance

- **Memory efficient**: Streaming support for large files
- **Fast**: Optimized repair strategies with caching
- **Reliable**: Comprehensive error handling

## Configuration

The MCP server uses default configurations. For advanced options, use the CLI:

```bash
anyrepair repair --input file.json --output repaired.json
```

## Troubleshooting

### Server not responding

Ensure the server is running and receiving input:

```bash
echo '{"tool": "validate", "input": {"content": "{}", "format": "json"}}' | anyrepair-mcp
```

### Tool not found

Verify the tool name is correct. Use lowercase format names:
- `repair_json` (not `repair_JSON`)
- `repair_yaml` (not `repair_YAML`)

### Confidence score not returned

Only format-specific repair tools return confidence scores. Use `repair_json`, `repair_yaml`, etc. instead of `repair`.

## Architecture

The MCP server is built on top of the anyrepair library:

```
anyrepair-mcp (binary)
    ↓
AnyrepairMcpServer (mcp_server.rs)
    ↓
anyrepair library (repair functions)
    ↓
Format-specific repairers (json, yaml, markdown, etc.)
```

## Contributing

To add new tools or formats:

1. Add the repair logic to the appropriate module
2. Add a new tool definition in `AnyrepairMcpServer::new()`
3. Add a handler method in `AnyrepairMcpServer::process_tool_call()`
4. Add tests in `src/mcp_server.rs`

## License

Apache-2.0

## See Also

- [AnyRepair Documentation](README.md)
- [MCP Protocol Specification](https://modelcontextprotocol.io/)
- [Claude Integration Guide](https://claude.ai/docs/mcp)
