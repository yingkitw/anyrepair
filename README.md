# AnyRepair

A Rust crate for repairing malformed LLM responses across multiple formats (JSON, YAML, Markdown, XML, TOML, CSV, INI, Diff).

## Quick Start

### Installation

```toml
[dependencies]
anyrepair = "0.1.10"
```

### Basic Usage

```rust
use anyrepair::repair;

// Auto-detect format and repair
let malformed = r#"{"name": "John", age: 30,}"#;
let repaired = repair(malformed)?;
println!("{}", repaired); // {"name": "John", "age": 30}
```

### CLI

```bash
# Install
cargo install anyrepair

# Auto-detect and repair
anyrepair repair input.json

# Format-specific repair
anyrepair json input.json
anyrepair yaml input.yaml
anyrepair markdown input.md

# Batch process multiple files
anyrepair batch ./data --recursive

# Custom rules management
anyrepair rules add --id "fix_undefined" --format "json" \
  --pattern "undefined" --replacement "null"
anyrepair rules list

# Stream large files
anyrepair stream large_input.json repaired_output.json

# Plugin management
anyrepair plugins list
anyrepair plugins load ./plugins/custom_rules.so

# Validation without repair
anyrepair validate input.json --format json
```

## What's New

### v0.1.10 - Latest Release

**🔧 8 Format Support**
- JSON, YAML, Markdown, XML, TOML, CSV, INI, Diff/Unified Diff
- Auto-detection from malformed content
- Format-specific validation and repair strategies

**🐍 Python-Compatible API**
- Drop-in compatible with Python's `jsonrepair` library
- `jsonrepair()` function and `JsonRepair` class API
- Easy migration from Python to Rust projects

**🔌 MCP Server Integration**
- Native Claude Desktop integration via MCP
- 10 MCP tools for format-specific and auto-detect repair
- Simple configuration, zero-dependency setup

**📦 Enterprise Features**
- **Analytics**: Track repair metrics, success rates, performance
- **Batch Processing**: Multi-file processing with format auto-detection
- **Validation Rules**: Custom rule engine for format validation
- **Audit Logging**: Compliance logging with event tracking
- **Confidence Scoring**: Assess repair quality (0.0-1.0 scale)

**⚡ Performance & Quality**
- **326+ test cases** with 100% pass rate
- **99.6% improvement** from regex caching
- **Streaming support** for files larger than RAM
- **Parallel strategy application** using rayon
- **94% binary size reduction** (1.5 MB release builds)

**🎯 Plugin System**
- Extensible plugin architecture for custom repair strategies
- WASM-based sandboxing support
- Plugin configuration via TOML
- Hot-reload capabilities

See [CHANGELOG.md](docs/CHANGELOG.md) for complete version history.

## Why AnyRepair?

LLMs often generate malformed structured data. AnyRepair fixes common issues:

- **JSON**: Missing quotes, trailing commas, syntax errors
- **YAML**: Indentation issues, missing colons
- **Markdown**: Malformed headers, broken links
- **XML/TOML/CSV/INI/Diff**: Format-specific repairs

**Key Features:**
- ✅ Auto-detects format from damaged content
- ✅ Multi-format support (8 formats)
- ✅ High performance (regex caching, optimized binaries)
- ✅ MCP server for Claude integration
- ✅ Streaming support for large files
- ✅ 326+ tests, 100% pass rate

## Usage Examples

### Multi-Format Auto-Detection

```rust
use anyrepair::repair;

// JSON - auto-detected and repaired
let json = repair(r#"{"key": value,}"#)?;
// Output: {"key": "value"}

// YAML - auto-detected and repaired
let yaml = repair("name: John\nage: 30")?;

// Markdown - auto-detected and repaired
let markdown = repair("# Header\n[link](url")?;

// Diff - auto-detected and repaired
let diff = repair("@@ -1,3 +1,4 @@\n-line 1\n+line 1 modified")?;
```

### Python-Compatible JSON API

```rust
use anyrepair::{jsonrepair, JsonRepair};

// Function-based API (like Python's jsonrepair)
let repaired = jsonrepair(r#"{"name": "John", age: 30,}"#)?;

// Class-based API (like Python's JsonRepair class)
let mut jr = JsonRepair::new();
let repaired1 = jr.jsonrepair(r#"{"key": "value",}"#)?;
let repaired2 = jr.jsonrepair(r#"{name: "John"}"#)?;
```

### Format-Specific Repairers

```rust
use anyrepair::{
    JsonRepairer, YamlRepairer, MarkdownRepairer,
    DiffRepairer, XmlRepairer, TomlRepairer
};

// JSON
let mut json_repairer = JsonRepairer::new();
let repaired = json_repairer.repair(malformed_json)?;

// YAML
let mut yaml_repairer = YamlRepairer::new();
let repaired = yaml_repairer.repair(malformed_yaml)?;

// Diff
let mut diff_repairer = DiffRepairer::new();
let repaired = diff_repairer.repair(malformed_diff)?;

// Check if repair is needed
if json_repairer.needs_repair(content) {
    let repaired = json_repairer.repair(content)?;
    let confidence = json_repairer.confidence(&repaired);
}
```

### Streaming Large Files

```rust
use anyrepair::StreamingRepair;
use std::fs::File;
use std::io::BufReader;

let input = BufReader::new(File::open("large_file.json")?);
let mut output = File::create("repaired.json")?;

// Configure buffer size (default 8192 bytes)
let processor = StreamingRepair::with_buffer_size(65536);

// Process with automatic format detection
processor.process(input, &mut output, None)?;

// Or specify format explicitly
processor.process(input, &mut output, Some("json"))?;
```

### Batch Processing

```rust
use anyrepair::BatchProcessor;

let processor = BatchProcessor::new();

// Process directory with options
let results = processor.process_directory(
    "./data",
    true,  // recursive
    "*.json",  // file filter
)?;

// Get per-file results
for result in results {
    println!("{}: {:?} ({}ms)",
        result.file_path,
        result.status,
        result.repair_time_ms
    );
}

// Get analytics
let analytics = processor.get_analytics();
println!("Success rate: {}%", analytics.success_rate());
```

### MCP Server Integration

The MCP server provides seamless integration with Claude Desktop:

```bash
# Install and run MCP server
cargo install anyrepair
anyrepair-mcp
```

**Configure in `claude_desktop_config.json`:**
```json
{
  "mcpServers": {
    "anyrepair": {
      "command": "anyrepair-mcp"
    }
  }
}
```

**Available MCP Tools:**
- `repair` - Auto-detect and repair any format
- `repair_json`, `repair_yaml`, `repair_markdown`, `repair_xml`
- `repair_toml`, `repair_csv`, `repair_ini`, `repair_diff`
- `validate` - Validate content without repair

**Usage in Claude:**
```
Please repair this JSON: {"name": "John", age: 30,}
(Claude will use the anyrepair MCP tool to fix it)
```

See [MCP_SERVER.md](docs/MCP_SERVER.md) for complete documentation.

## Supported Formats

| Format | Common Issues Fixed |
|--------|---------------------|
| **JSON** | Missing quotes, trailing commas, malformed numbers, boolean/null values |
| **YAML** | Indentation, missing colons, list formatting, document separators |
| **Markdown** | Headers, code blocks, lists, tables, links, images |
| **XML** | Unclosed tags, malformed attributes, missing quotes, entity encoding |
| **TOML** | Missing quotes, malformed arrays, table headers, dates |
| **CSV** | Unquoted strings, malformed quotes, extra/missing commas |
| **INI** | Malformed sections, missing equals signs, unquoted values |
| **Diff** | Missing hunk headers, incorrect line prefixes, malformed ranges |

## Advanced Features

### Custom Rules

```bash
# Add custom repair rule via CLI
anyrepair rules add --id "fix_undefined" --format "json" \
  --pattern "undefined" --replacement "null" --priority 90

# List all rules
anyrepair rules list

# Enable/disable rules
anyrepair rules enable "fix_undefined"
anyrepair rules disable "fix_undefined"

# Remove a rule
anyrepair rules remove "fix_undefined"
```

**Configuration file (anyrepair.toml):**
```toml
# Custom rules configuration
[[rules]]
id = "fix_trailing_comma"
format = "json"
pattern = ",\\s*}"
replacement = "}"
priority = 95

[[rules]]
id = "fix_js_comments"
format = "json"
pattern = "//.*\\n"
replacement = ""
priority = 80
```

### Plugin System

Extend functionality with custom repair strategies:

```rust
use anyrepair::plugin::{Plugin, PluginContext};

struct CustomJsonPlugin;

impl Plugin for CustomJsonPlugin {
    fn name(&self) -> &str { "custom_json" }
    fn version(&self) -> &str { "1.0.0" }
    fn supports_format(&self, format: &str) -> bool {
        format == "json"
    }
    fn execute(&self, content: &str, _ctx: &PluginContext) -> Result<String> {
        // Custom repair logic here
        Ok(content.to_string())
    }
}
```

**Plugin configuration (plugins.toml):**
```toml
[[plugins]]
name = "custom_json_rules"
path = "./plugins/custom_json_rules.so"
enabled = true
priority = 85
formats = ["json"]

[plugins.config]
strict_mode = true
preserve_comments = false
```

See [Plugin Development Guide](docs/PLUGIN_DEVELOPMENT.md) for complete documentation.

### Enterprise Features

**Analytics:**
```rust
use anyrepair::AnalyticsTracker;

let tracker = AnalyticsTracker::new();
tracker.track_repair_result("json", true, 150.0);
let metrics = tracker.get_metrics();
println!("Success rate: {}", metrics.success_rate());
```

**Validation Rules:**
```rust
use anyrepair::ValidationRulesEngine;

let engine = ValidationRulesEngine::new();
let violations = engine.validate(content, "json")?;
for violation in violations {
    println!("Rule '{}' violated: {}", violation.rule_id, violation.message);
}
```

**Audit Logging:**
```rust
use anyrepair::AuditLogger;

let logger = AuditLogger::new();
logger.log_repair("json", input, output, true)?;
logger.log_validation("json", content, &violations)?;
```

**Confidence Scoring:**
```rust
use anyrepair::ConfidenceScorer;

let scorer = ConfidenceScorer::new();
let confidence = scorer.score(&repaired_content, "json");
if confidence < 0.8 {
    // Manual review may be needed
}
```

See [Enterprise Features](docs/enterprise_features.md) for details.

## Performance

- **Regex Caching**: 99.6% performance improvement over uncached operations
- **Optimized Binaries**: 1.5 MB release builds (94% size reduction)
- **Parallel Processing**: Multi-threaded strategy application with rayon
- **Streaming**: Process files larger than available RAM using configurable buffers
- **Lazy Evaluation**: Skip unnecessary strategies for faster repairs

**Build Profiles:**
```bash
# Standard release (size-optimized)
cargo build --release

# Distribution profile (maximum optimization)
cargo build --profile dist
```

## Testing

- **326+ test cases** with 100% pass rate
  - 204 library tests
  - 26 streaming tests
  - 36 fuzz tests
  - 18 complex damage tests
  - 18 complex streaming tests
  - 4 integration tests
  - 2 doctests
- **Fuzz testing** using proptest for robustness
- **Snapshot testing** with insta for regression prevention
- **Integration tests** for end-to-end workflows
- **MCP test coverage** with 43 comprehensive tests

See [TEST_SUMMARY.md](docs/TEST_SUMMARY.md) for details.

## Comparison

| Feature | AnyRepair | json-repair-rs | json5 | Python jsonrepair |
|---------|-----------|----------------|-------|-------------------|
| **Multi-format** | ✅ 8 formats | ❌ JSON only | ❌ JSON only | ❌ JSON only |
| **Auto-detection** | ✅ Smart detection | ❌ | ❌ | ❌ |
| **MCP integration** | ✅ Native | ❌ | ❌ | ❌ |
| **Streaming** | ✅ Large file support | ❌ | ❌ | ❌ |
| **Custom rules** | ✅ CLI + API | ❌ | ❌ | ❌ |
| **Plugin system** | ✅ WASM plugins | ❌ | ❌ | ❌ |
| **Python API** | ✅ Compatible | ❌ | ❌ | ✅ Native |
| **Enterprise features** | ✅ Analytics, audit | ❌ | ❌ | ❌ |
| **Language** | Rust | Rust | Rust | Python |
| **Binary size** | 1.5 MB | ~500 KB | ~200 KB | N/A |

**Why AnyRepair?**
- Most comprehensive format support (8 formats vs JSON-only alternatives)
- Only Rust crate with Python-compatible API and MCP integration
- Enterprise-grade features: analytics, audit logging, validation rules
- Battle-tested with 326+ tests covering real-world LLM failures

## Documentation

- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design and architecture
- **[MCP_SERVER.md](docs/MCP_SERVER.md)** - MCP server integration guide
- **[PLUGIN_DEVELOPMENT.md](docs/PLUGIN_DEVELOPMENT.md)** - Plugin development guide
- **[TEST_SUMMARY.md](docs/TEST_SUMMARY.md)** - Test coverage details
- **[CHANGELOG.md](docs/CHANGELOG.md)** - Version history and changes
- **[INDEX.md](docs/INDEX.md)** - Complete documentation index
- **[STREAMING_FEATURE.md](docs/STREAMING_FEATURE.md)** - Streaming support details
- **[BUILD_OPTIMIZATION.md](docs/BUILD_OPTIMIZATION.md)** - Build optimization guide

### Quick Links

- **Report Issues**: [GitHub Issues](https://github.com/yingkitw/anyrepair/issues)
- **Contributing**: See [CONTRIBUTING.md](CONTRIBUTING.md) (if available)
- **Changelog**: [CHANGELOG.md](docs/CHANGELOG.md)
- **API Docs**: [docs.rs](https://docs.rs/anyrepair)

## Examples

See the [examples/](examples/) directory for:

- **[example_plugin.rs](examples/example_plugin.rs)** - Custom plugin implementation
- **[mcp_repair_json.rs](examples/mcp_repair_json.rs)** - MCP JSON repair usage
- **[mcp_multi_format.rs](examples/mcp_multi_format.rs)** - Multi-format MCP repair
- **[mcp_server_usage.rs](examples/mcp_server_usage.rs)** - MCP server setup and usage

Run examples:
```bash
cargo run --example mcp_repair_json
cargo run --example example_plugin
```

## Roadmap

See [TODO.md](TODO.md) for planned features and improvement areas. Highlights include:

- Additional format support (Properties, .env, GraphQL, SQL, Protobuf)
- CLI enhancements (watch mode, interactive mode, diff preview)
- Web interface and REST API
- Language bindings (Python, Node.js, Go)
- LSP integration and VS Code extension
- Advanced features (format-preserving repairs, LLM-assisted repairs)

## License

Apache-2.0

## Repository

https://github.com/yingkitw/anyrepair
