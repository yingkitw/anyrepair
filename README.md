# AnyRepair

A Rust crate for repairing LLM responses including JSON, YAML, Markdown, XML, TOML, CSV, and INI files.

## Features

- **Multi-format repair**: JSON, YAML, Markdown, XML, TOML, CSV, INI
- **Auto-detection**: Automatically detects format and applies appropriate repairs
- **High performance**: Regex caching with up to 99.6% performance improvement
- **CLI tool**: Command-line interface for easy usage
- **Comprehensive testing**: 100+ test cases with snapshot testing
- **Parallel processing**: Multi-threaded strategy application for better performance
- **Advanced strategies**: Intelligent format detection, adaptive repair, and context-aware processing

## Installation

```toml
[dependencies]
anyrepair = "0.1.1"
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

- **JSON**: Missing quotes, trailing commas, malformed numbers, boolean/null values
- **YAML**: Indentation, missing colons, list formatting, complex structures
- **Markdown**: Headers, code blocks, lists, tables, links, images
- **XML**: Unclosed tags, malformed attributes, missing quotes, invalid characters
- **TOML**: Missing quotes, malformed arrays, table headers, numbers, dates
- **CSV**: Unquoted strings, malformed quotes, extra/missing commas, headers
- **INI**: Malformed sections, missing equals signs, unquoted values, comments

## License

Apache-2.0

## Repository

https://github.com/yingkitw/anyrepair