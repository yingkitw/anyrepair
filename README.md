# AnyRepair

A Rust crate for repairing malformed LLM responses across multiple formats (JSON, YAML, Markdown, XML, TOML, CSV, INI, Diff).

## Quick Start

### Installation

```toml
[dependencies]
anyrepair = "0.1.5"
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

# Repair a file
anyrepair input.json

# Auto-detect format
anyrepair input.txt
```

## What's New

### Latest Release (v0.1.5+)

**🆕 Diff/Unified Diff Format Support**
- Repair malformed git diffs and unified diff patches
- Fix missing hunk headers, incorrect line prefixes, malformed ranges
- Support for multi-file diffs and complex diff scenarios
- 35+ comprehensive test cases

**📊 Enterprise Features**
- Advanced analytics and metrics tracking
- Batch processing for multiple files
- Custom validation rules engine
- Comprehensive audit logging for compliance

**📚 Documentation Improvements**
- Consolidated documentation structure
- Simplified README for easier adoption
- Enhanced architecture documentation
- Complete test coverage documentation

**⚡ Performance & Quality**
- 364+ test cases with 100% pass rate
- Optimized regex caching (99.6% improvement)
- Streaming support for large files
- Enhanced confidence scoring algorithms

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
- ✅ 364+ tests, 100% pass rate

## Usage Examples

### Multi-Format Auto-Detection

```rust
use anyrepair::repair;

// JSON
let json = repair(r#"{"key": value,}"#)?;

// YAML
let yaml = repair("name: John\nage: 30")?;

// Markdown
let markdown = repair("# Header\n[link](url")?;
```

### Format-Specific Repair

```rust
use anyrepair::{JsonRepair, YamlRepairer, MarkdownRepairer};

// JSON (Python jsonrepair-compatible API)
let mut jr = JsonRepair::new();
let repaired = jr.jsonrepair(malformed_json)?;

// YAML
let mut yaml_repairer = YamlRepairer::new();
let repaired = yaml_repairer.repair(malformed_yaml)?;

// Markdown
let mut md_repairer = MarkdownRepairer::new();
let repaired = md_repairer.repair(malformed_markdown)?;
```

### Streaming Large Files

```rust
use anyrepair::StreamingRepair;
use std::fs::File;
use std::io::BufReader;

let input = BufReader::new(File::open("large_file.json")?);
let mut output = File::create("repaired.json")?;

let processor = StreamingRepair::with_buffer_size(8192);
processor.process(input, &mut output, "json")?;
```

### MCP Server Integration

```bash
# Run MCP server for Claude
anyrepair-mcp

# Configure in claude_desktop_config.json:
# {
#   "mcpServers": {
#     "anyrepair": {
#       "command": "anyrepair-mcp"
#     }
#   }
# }
```

See [MCP_SERVER.md](docs/MCP_SERVER.md) for details.

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
# Add custom repair rule
anyrepair rules add --id "fix_undefined" --format "json" \
  --pattern "undefined" --replacement "null"
```

### Plugin System

Extend functionality with custom repair strategies. See [Plugin Development Guide](docs/PLUGIN_DEVELOPMENT.md).

### Enterprise Features

- **Analytics**: Track repair operations and success rates
- **Batch Processing**: Process multiple files across formats
- **Validation Rules**: Define and enforce custom validation
- **Audit Logging**: Comprehensive logging for compliance

See [Enterprise Features](docs/enterprise_features.md) for details.

## Performance

- **Regex Caching**: Up to 99.6% performance improvement
- **Optimized Binaries**: 1.5 MB release builds
- **Parallel Processing**: Multi-threaded strategy application
- **Streaming**: Process files larger than available RAM

## Testing

- **364+ test cases** with 100% pass rate
- **Fuzz testing** for robustness
- **Snapshot testing** for complex scenarios
- **Integration tests** for end-to-end workflows

See [TEST_SUMMARY.md](docs/TEST_SUMMARY.md) for details.

## Comparison

| Feature | AnyRepair | json-repair-rs | json5 |
|---------|-----------|----------------|-------|
| Multi-format | ✅ 8 formats | ❌ JSON only | ❌ JSON only |
| Auto-detection | ✅ | ❌ | ❌ |
| MCP integration | ✅ | ❌ | ❌ |
| Streaming | ✅ | ❌ | ❌ |
| Custom rules | ✅ | ❌ | ❌ |

## Documentation

- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design and architecture
- **[MCP_SERVER.md](docs/MCP_SERVER.md)** - MCP server integration
- **[PLUGIN_DEVELOPMENT.md](docs/PLUGIN_DEVELOPMENT.md)** - Plugin development guide
- **[TEST_SUMMARY.md](docs/TEST_SUMMARY.md)** - Test coverage details
- **[INDEX.md](docs/INDEX.md)** - Complete documentation index

## Examples

See the [examples/](examples/) directory for:
- JSON repair examples
- Multi-format usage
- MCP server integration
- Plugin development

## License

Apache-2.0

## Repository

https://github.com/yingkitw/anyrepair
