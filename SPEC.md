# AnyRepair Specification

## Overview

AnyRepair is a deterministic, heuristic-based Rust library and CLI tool for repairing malformed structured data across 8 formats. It uses pattern matching and rule-based strategies — no machine learning or external API calls.

## Supported Formats

| Format   | Module          | Aliases     |
|----------|-----------------|-------------|
| JSON     | `json.rs`       | —           |
| YAML     | `yaml.rs`       | `yml`       |
| Markdown | `markdown.rs`   | `md`        |
| XML      | `xml.rs`        | —           |
| TOML     | `toml.rs`       | —           |
| CSV      | `csv.rs`        | —           |
| INI      | `ini.rs`        | —           |
| Diff     | `diff.rs`       | —           |

## Core Architecture

### Format Registry (Single Source of Truth)

All format→repairer/validator mapping is centralized in `lib.rs`:

```rust
pub const SUPPORTED_FORMATS: &[&str];
pub fn normalize_format(format: &str) -> &str;
pub fn create_repairer(format: &str) -> Result<Box<dyn Repair>>;
pub fn create_validator(format: &str) -> Result<Box<dyn Validator>>;
pub fn detect_format(content: &str) -> Option<&'static str>;
pub fn repair(content: &str) -> Result<String>;
pub fn repair_with_format(content: &str, format: &str) -> Result<String>;
```

Adding a new format requires changes in **3 places only**:
1. `SUPPORTED_FORMATS` constant
2. `create_repairer()` match arm
3. `create_validator()` match arm
4. Detection heuristic in `format_detection.rs`

No CLI changes needed — the unified `repair --format` command picks up new formats automatically.

### Traits

```rust
pub trait Repair {
    fn repair(&mut self, content: &str) -> Result<String>;
    fn needs_repair(&self, content: &str) -> bool;
    fn confidence(&self, content: &str) -> f64;  // 0.0–1.0
}

pub trait RepairStrategy {
    fn apply(&self, content: &str) -> Result<String>;
    fn priority(&self) -> u8;   // higher = applied first
    fn name(&self) -> &str;
}

pub trait Validator {
    fn is_valid(&self, content: &str) -> bool;
    fn validate(&self, content: &str) -> Vec<String>;
}
```

### Repairer Composition

Each format repairer wraps `GenericRepairer` (composition, not inheritance):

```
FormatRepairer {
    inner: GenericRepairer {
        strategies: Vec<Box<dyn RepairStrategy>>,
        validator: Box<dyn Validator>,
        repair_log: Vec<String>,
    }
}
```

- `repair()` and `needs_repair()` delegate to `GenericRepairer`
- `confidence()` has format-specific scoring logic per repairer

### Format Detection

`format_detection.rs` contains all heuristic detection logic. Detection order:

1. **JSON** — starts with `{`/`[`
2. **Diff** — `@@` hunk headers, paired `---`/`+++` file headers
3. **YAML** — `:`, `---` document separator
4. **XML** — `<?xml`, `<tag>`
5. **TOML** — `[section]`, `key = value`
6. **CSV** — commas + multiple lines
7. **INI** — `[section]`, `key = value` (no commas/colons)
8. **Markdown** — `#`, `` ` ``, `**`, `*`

Diff is checked before YAML/CSV to avoid false positives (diff content contains colons and commas).

## CLI Interface

Single unified command for all formats:

```
anyrepair repair [FILE] [--format <fmt>] [--confidence] [--input <file>] [--output <file>]
anyrepair validate [--input <file>] [--format <fmt>]
anyrepair stream [--input <file>] [--output <file>] [--format <fmt>] [--buffer-size <bytes>]
anyrepair batch --input <dir> --output <dir> [--pattern <glob>] [--recursive]
anyrepair stats
anyrepair rules <action>
```

- `--format` accepts any value from `SUPPORTED_FORMATS` plus aliases (`yml`, `md`)
- Without `--format`, auto-detection is used
- `--confidence` prints repair confidence score (0–100%)
- `--verbose` / `--quiet` flags control output verbosity

## Streaming

`StreamingRepair` processes large files line-by-line with configurable buffer size:

```rust
let processor = StreamingRepair::with_buffer_size(65536);
processor.process(reader, &mut writer, "json")?;
```

## Python-Compatible API

Drop-in replacement for Python's `jsonrepair` library:

```rust
pub fn jsonrepair(json_str: &str) -> Result<String>;

pub struct JsonRepair { .. }
impl JsonRepair {
    pub fn new() -> Self;
    pub fn jsonrepair(&mut self, json_str: &str) -> Result<String>;
}
```

## Error Handling

```rust
pub enum RepairError {
    JsonRepair(String),
    YamlRepair(String),
    MarkdownRepair(String),
    FormatDetection(String),
    Io(std::io::Error),
    Serde(serde_json::Error),
    Yaml(serde_yaml::Error),
    Regex(regex::Error),
    Generic(String),
}
```

All public functions return `Result<T>` using this error type.

## Design Principles

- **DRY**: Centralized format registry eliminates duplicated dispatch logic
- **SoC**: Format detection, repair logic, CLI, and validation are in separate modules
- **KISS**: One `repair --format` command instead of per-format subcommands
- **Deterministic**: All repairs are heuristic/pattern-based, no ML or external calls
- **Composition over inheritance**: `GenericRepairer` composed into format repairers via `inner` field
- **Test-friendly**: Trait-based design enables easy mocking and isolated testing

## Testing

402+ tests across multiple categories:

| Category              | Count | Location                        |
|-----------------------|-------|---------------------------------|
| Library unit tests    | 216   | `src/*.rs` `#[cfg(test)]`       |
| Diff tests            | 35    | `tests/diff_tests.rs`           |
| Fuzz tests            | 36    | `tests/fuzz_tests.rs`           |
| Streaming tests       | 26    | `tests/streaming_tests.rs`      |
| Damage scenarios      | 18    | `tests/damage_scenarios.rs`     |
| Complex damage        | 18    | `tests/complex_damage_tests.rs` |
| Complex streaming     | 18    | `tests/complex_streaming_tests.rs` |
| Integration tests     | 17    | `tests/integration_tests.rs`    |
| CLI tests             | 15    | `tests/cli_tests.rs`            |
| Doc tests             | 2     | `src/lib.rs`                    |

All tests pass. Zero failures, zero binary warnings.

## Dependencies

| Crate          | Purpose                    |
|----------------|----------------------------|
| `serde`        | Serialization framework    |
| `serde_json`   | JSON parsing/validation    |
| `serde_yaml`   | YAML parsing/validation    |
| `pulldown-cmark` | Markdown parsing         |
| `regex`        | Pattern matching           |
| `thiserror`    | Error type derivation      |
| `anyhow`       | Error context              |
| `clap`         | CLI argument parsing       |
| `rmcp`         | MCP server protocol        |

## File Structure

```
src/
├── lib.rs                 # Public API, format registry
├── main.rs                # CLI entry point
├── format_detection.rs    # Format detection heuristics
├── traits.rs              # Repair, RepairStrategy, Validator traits
├── repairer_base.rs       # GenericRepairer (shared repair logic)
├── error.rs               # Error types
├── json.rs                # JSON repairer + strategies + validator
├── yaml.rs                # YAML repairer
├── markdown.rs            # Markdown repairer
├── xml.rs                 # XML repairer
├── toml.rs                # TOML repairer
├── csv.rs                 # CSV repairer
├── ini.rs                 # INI repairer
├── diff.rs                # Diff repairer
├── streaming.rs           # Streaming repair for large files
├── mcp_server.rs          # MCP server implementation
├── cli/                   # CLI command handlers
│   ├── mod.rs             # I/O utilities
│   ├── repair_cmd.rs      # Unified repair handler
│   ├── validate_cmd.rs    # Validation handler
│   ├── batch_cmd.rs       # Batch processing
│   ├── stream_cmd.rs      # Streaming handler
│   └── rules_cmd.rs       # Rules management
├── advanced.rs            # Advanced repair features
├── enhanced_json.rs       # Enhanced JSON (json.loads compat)
├── context_parser.rs      # Context-aware string parsing
├── config.rs              # Configuration management
└── custom_rules.rs        # Custom rule engine
```
