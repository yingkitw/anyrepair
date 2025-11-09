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
- **Comprehensive Testing**: 326 test cases (204 library + 4 integration + 26 streaming + 18 complex damage + 18 complex streaming + 36 fuzz + 18 damage scenarios + 2 doctests) with 100% pass rate
- **High Performance**: Regex caching with 99.6% performance improvement, optimized binaries (1.5 MB)
- **CLI & Library**: Both command-line tool and Rust library for integration
- **MCP Server**: Model Context Protocol server for Claude and other AI clients
- **Streaming Support**: Process large files with minimal memory overhead
- **Configuration Management**: TOML-based configuration with custom rules
- **Enterprise Features**: Analytics, batch processing, validation rules, and audit logging
- **Python-Compatible API**: Drop-in replacement for Python's jsonrepair library

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
- **Comprehensive testing**: 326 test cases with snapshot and fuzz testing
- **Parallel processing**: Multi-threaded strategy application for better performance
- **Advanced strategies**: Intelligent format detection, adaptive repair, and context-aware processing
- **Plugin system**: Extensible architecture for custom repair strategies
- **Custom rules**: User-defined repair rules with full CLI management
- **Fuzz testing**: Comprehensive property-based testing for robustness
- **Configuration**: TOML-based configuration with custom rules and plugin settings
- **Python-compatible API**: Drop-in replacement for Python's jsonrepair library

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

### **MCP Server**

AnyRepair now includes a dedicated MCP server for integration with Claude and other MCP-compatible clients:

```bash
# Run the MCP server
anyrepair-mcp

# Or integrate with Claude
# Add to claude_desktop_config.json:
# {
#   "mcpServers": {
#     "anyrepair": {
#       "command": "anyrepair-mcp"
#     }
#   }
# }
```

See [MCP_SERVER.md](MCP_SERVER.md) for detailed documentation.

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

## Enterprise Features

AnyRepair now includes comprehensive enterprise-grade features:

### **Advanced Analytics**
Track repair operations with detailed metrics:
```rust
use anyrepair::AnalyticsTracker;
use std::time::Duration;

let tracker = AnalyticsTracker::new();
tracker.record_repair("json", true, Duration::from_millis(10), 0.95);
let metrics = tracker.get_metrics();
println!("Success rate: {}%", tracker.get_success_rate());
```

### **Batch Processing**
Process multiple files across different formats:
```rust
use anyrepair::BatchProcessor;
use std::path::Path;

let processor = BatchProcessor::new();
let results = processor.process_directory(
    Path::new("./data"),
    true,
    Some(&["json", "yaml", "xml"])
)?;
println!("Processed: {}", results.total_files);
```

### **Custom Validation Rules**
Define and enforce validation rules:
```rust
use anyrepair::ValidationRulesEngine;
use anyrepair::validation_rules::{ValidationRule, RuleType};

let mut engine = ValidationRulesEngine::new();
let rule = ValidationRule {
    name: "max_size".to_string(),
    rule_type: RuleType::Length,
    pattern: "10000".to_string(),
    error_message: "Content exceeds maximum size".to_string(),
    enabled: true,
};
engine.add_rule(rule);
let result = engine.validate(content);
```

### **Audit Logging**
Comprehensive audit logging for compliance:
```rust
use anyrepair::AuditLogger;

let logger = AuditLogger::with_file("audit.log");
logger.log_repair("data.json", "json", true, "user@example.com", Some("Automated repair"));
let entries = logger.get_entries_by_type("REPAIR");
```

### **Advanced Confidence Scoring**
Improved confidence scoring algorithms:
```rust
use anyrepair::ConfidenceScorer;

let score = ConfidenceScorer::score_json(content);
let yaml_score = ConfidenceScorer::score_yaml(content);
let xml_score = ConfidenceScorer::score_xml(content);
```

## Installation

```toml
[dependencies]
anyrepair = "0.1.5"
```

## Usage

### Library

#### Python jsonrepair Compatible API

AnyRepair provides a Python jsonrepair-compatible interface for easy migration:

**Function-based API:**
```rust
use anyrepair::jsonrepair;

// Simple function call matching Python's jsonrepair
let malformed = r#"{"name": "John", age: 30,}"#;
let repaired = jsonrepair(malformed)?;
println!("{}", repaired); // {"name": "John", "age": 30}
```

**Class-based API:**
```rust
use anyrepair::JsonRepair;

// Class-like interface matching Python's JsonRepair class
let mut jr = JsonRepair::new();
let malformed = r#"{"key": "value",}"#;
let repaired = jr.jsonrepair(malformed)?;
println!("{}", repaired); // {"key": "value"}
```

#### Multi-Format Auto-Detection

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

# Stream repair large files with minimal memory
anyrepair stream --input large_file.json --output repaired.json --format json

# Batch process
anyrepair batch --input-dir ./files --output-dir ./repaired

# Get statistics
anyrepair stats --input-dir ./files
```

## Streaming Repair for Large Files

AnyRepair includes streaming repair capabilities for processing large files with minimal memory overhead:

```rust
use anyrepair::StreamingRepair;
use std::fs::File;
use std::io::BufReader;

let input = BufReader::new(File::open("large_file.json")?);
let mut output = File::create("repaired.json")?;

let processor = StreamingRepair::with_buffer_size(8192);
let bytes_processed = processor.process(input, &mut output, "json")?;
println!("Processed {} bytes", bytes_processed);
```

**Benefits:**
- Process files larger than available RAM
- Configurable buffer size for memory optimization
- Automatic format detection
- Streaming from stdin/stdout support
- Progress tracking via byte count

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