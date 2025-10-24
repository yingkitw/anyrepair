# AnyRepair

A comprehensive Rust crate for repairing LLM responses across multiple formats including JSON, YAML, Markdown, XML, TOML, CSV, and INI files.

## Why AnyRepair?

### **Critical for Agentic AI and Tool Use**

In the era of agentic AI systems, reliable data parsing is essential for:

- **Tool Function Calls**: AI agents must parse structured data from external APIs and tools
- **MCP (Model Context Protocol)**: Ensures reliable communication between AI models and external systems
- **Agent Workflows**: Multi-step AI processes depend on consistent data format handling
- **Error Recovery**: When AI outputs malformed data, repair enables graceful recovery
- **Production Reliability**: Real-world AI applications need robust data handling

### **The Problem with LLM Output**

LLMs often generate:
- **Malformed JSON** with missing quotes, trailing commas, or syntax errors
- **Inconsistent YAML** with indentation issues or missing colons
- **Broken Markdown** with malformed headers, links, or code blocks
- **Invalid XML/TOML/CSV/INI** with structural problems

Without repair, these errors cause:
- **Tool failures** in agentic workflows
- **MCP communication breakdowns**
- **Cascading errors** in multi-step AI processes
- **Poor user experience** with unreliable AI applications

### **AnyRepair's Solution**

While there are several JSON repair tools available in Rust, AnyRepair addresses the unique challenges of LLM-generated content across multiple formats:

### **Multi-Format Support**
Unlike single-format tools like `json-repair-rs` or `json5`, AnyRepair handles **7 different formats** (JSON, YAML, Markdown, XML, TOML, CSV, INI) with auto-detection, making it perfect for LLM responses that can be in any format.

### **LLM-Specific Optimizations**
- **Intelligent Format Detection**: Automatically detects format from damaged content
- **Context-Aware Repair**: Understands LLM output patterns and common errors
- **Rule-Based Confidence Scoring**: Provides repair quality metrics using pattern-based rules (no LLM required)
- **Parallel Processing**: Optimized for batch processing of LLM responses

### **Advanced Repair Strategies**
- **Adaptive Repair**: Strategies that adapt based on content complexity
- **Multi-Pass Processing**: Applies multiple repair strategies in optimal order
- **Custom Rules**: User-defined repair patterns for specific use cases
- **Plugin System**: Extensible architecture for custom repair logic

### **Production-Ready Features**
- **Comprehensive Testing**: 116+ test cases with fuzz testing for robustness
- **High Performance**: Regex caching with 99.6% performance improvement
- **CLI & Library**: Both command-line tool and Rust library for integration
- **Configuration Management**: TOML-based configuration with custom rules

### **Comparison with Other Tools**

| Feature | AnyRepair | json-repair-rs | json5 | Other Tools |
|---------|-----------|----------------|-------|-------------|
| Multi-format | ✅ 7 formats | ❌ JSON only | ❌ JSON only | ❌ Single format |
| Auto-detection | ✅ | ❌ | ❌ | ❌ |
| LLM-optimized | ✅ | ❌ | ❌ | ❌ |
| Agentic AI support | ✅ | ❌ | ❌ | ❌ |
| MCP integration | ✅ | ❌ | ❌ | ❌ |
| Custom rules | ✅ | ❌ | ❌ | ❌ |
| Plugin system | ✅ | ❌ | ❌ | ❌ |
| Rule-based confidence scoring | ✅ | ❌ | ❌ | ❌ |
| Parallel processing | ✅ | ❌ | ❌ | ❌ |
| Fuzz testing | ✅ | ❌ | ❌ | ❌ |

## Features

- **Multi-format repair**: JSON, YAML, Markdown, XML, TOML, CSV, INI
- **Auto-detection**: Automatically detects format and applies appropriate repairs
- **High performance**: Regex caching with up to 99.6% performance improvement
- **CLI tool**: Command-line interface for easy usage
- **Comprehensive testing**: 116+ test cases with snapshot and fuzz testing
- **Parallel processing**: Multi-threaded strategy application for better performance
- **Advanced strategies**: Intelligent format detection, adaptive repair, and context-aware processing
- **Plugin system**: Extensible architecture for custom repair strategies
- **Custom rules**: User-defined repair rules with full CLI management
- **Fuzz testing**: Comprehensive property-based testing for robustness
- **Configuration**: TOML-based configuration with custom rules and plugin settings

## Rule-Based Confidence Scoring

AnyRepair uses sophisticated pattern-based rules to calculate confidence scores without requiring any LLM calls:

### **JSON Confidence Rules**
- **Structure Detection**: Checks for balanced braces `{}` and brackets `[]`
- **Key-Value Patterns**: Detects colon `:` separators and quote patterns
- **Syntax Validation**: Validates JSON structure without parsing
- **Balance Scoring**: Rewards properly balanced opening/closing delimiters

### **YAML Confidence Rules**
- **Indentation Analysis**: Evaluates consistent indentation patterns
- **Key-Value Detection**: Identifies colon `:` separators and list indicators `-`
- **Document Structure**: Recognizes YAML document separators `---`
- **Content Patterns**: Detects YAML-specific syntax elements

### **Format-Specific Rules**
Each format has tailored confidence rules:
- **Markdown**: Header patterns `#`, code blocks ````, link syntax `[]()`
- **XML**: Tag structure `<>`, attribute patterns, entity encoding
- **TOML**: Table headers `[]`, key-value pairs `=`, array syntax
- **CSV**: Delimiter consistency, quote patterns, row structure
- **INI**: Section headers `[]`, key-value pairs `=`, comment patterns

### **Benefits of Rule-Based Approach**
- **Fast**: No external API calls or network requests
- **Reliable**: Consistent scoring based on deterministic rules
- **Transparent**: Clear understanding of how scores are calculated
- **Customizable**: Rules can be extended through the plugin system

## Agentic AI & MCP Integration

### **Model Context Protocol (MCP) Support**

AnyRepair is designed to work seamlessly with MCP implementations:

```rust
use anyrepair::repair;

// MCP tool call response repair
let mcp_response = r#"{"tool": "search", "params": {"query": "AI news"}, "result": "..."}"#;
let repaired = repair(mcp_response)?;

// MCP context data repair
let context_data = r#"name: AI Assistant
version: 1.0
capabilities: [search, analyze, generate]"#;
let repaired_context = repair(context_data)?;
```

### **Agentic AI Workflow Integration**

Perfect for AI agent systems that need reliable data handling:

```rust
// Agent tool execution with repair
async fn execute_agent_tool(tool_call: &str) -> Result<String> {
    let response = call_external_api(tool_call).await?;
    
    // Repair the response before processing
    let repaired = anyrepair::repair(&response)?;
    
    // Parse and use the repaired data
    let parsed: serde_json::Value = serde_json::from_str(&repaired)?;
    process_agent_response(parsed)
}
```

### **Use Cases**

- **AI Agent Tool Calls**: Repair malformed responses from external APIs
- **MCP Communication**: Ensure reliable data exchange between AI models and tools
- **Multi-Agent Systems**: Handle data format inconsistencies across different agents
- **Production AI Apps**: Robust error handling for real-world AI applications
- **LLM Output Processing**: Clean and validate AI-generated structured data

## Installation

```toml
[dependencies]
anyrepair = "0.1.2"
```

## Usage

### Library

```rust
use anyrepair::repair;

// Auto-detect format and repair
let content = r#"{"name": "John", "age": 30,}"#;
let repaired = repair(content)?;
println!("{}", repaired); // {"name": "John", "age": 30}
```

### CLI

```bash
# Install
cargo install anyrepair

# Repair a file
anyrepair input.json

# Batch process
anyrepair batch --input-dir ./files --output-dir ./repaired

# Get statistics
anyrepair stats --input-dir ./files
```

## Supported Formats

- **JSON**: Missing quotes, trailing commas, malformed numbers, boolean/null values, nested structures
- **YAML**: Indentation, missing colons, list formatting, complex structures, document separators
- **Markdown**: Headers, code blocks, lists, tables, links, images, bold/italic formatting
- **XML**: Unclosed tags, malformed attributes, missing quotes, invalid characters, entity encoding
- **TOML**: Missing quotes, malformed arrays, table headers, numbers, dates, inline tables
- **CSV**: Unquoted strings, malformed quotes, extra/missing commas, headers, field escaping
- **INI**: Malformed sections, missing equals signs, unquoted values, comments, key-value pairs

## Plugin System

AnyRepair features a powerful plugin system that allows you to extend functionality with custom repair strategies, validators, and repairers.

### Plugin Management

```bash
# List available plugins
anyrepair plugins list

# Show plugin information
anyrepair plugins info my_plugin

# Enable/disable plugins
anyrepair plugins toggle --id my_plugin --enable

# Show plugin statistics
anyrepair plugins stats

# Discover plugins in directories
anyrepair plugins discover --paths ./plugins,./custom-plugins
```

### Custom Rules

Create and manage custom repair rules:

```bash
# Initialize configuration with templates
anyrepair rules init

# Add a custom rule
anyrepair rules add --id "fix_undefined" --name "Fix Undefined" --format "json" --pattern "undefined" --replacement "null"

# Test a rule
anyrepair rules test --id "fix_undefined" --input '{"value": undefined}'

# List all rules
anyrepair rules list
```

### Plugin Development

See [Plugin Development Guide](docs/PLUGIN_DEVELOPMENT.md) for detailed information on creating custom plugins.

## License

Apache-2.0

## Repository

https://github.com/yingkitw/anyrepair