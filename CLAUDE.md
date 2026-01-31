# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Table of Contents:**
- [Project Overview](#project-overview)
- [Development Commands](#development-commands) - Build, test, CLI reference
- [Architecture Overview](#architecture-overview) - Core patterns, format detection
- [Module Organization](#module-organization) - File structure and descriptions
- [Adding New Format Support](#adding-new-format-support)
- [Common Task Workflows](#common-task-workflows) - Debugging, custom strategies, streaming
- [Testing Conventions](#testing-conventions)
- [Enterprise Features](#enterprise-features-deep-dive) - Analytics, batch processing, validation, audit
- [Custom Rules System](#custom-rules-system) - User-defined repair patterns
- [Plugin System](#plugin-system) - Extensible plugin architecture
- [MCP Server Integration](#mcp-server-integration)
- [Quick Reference](#quick-reference) - Task/command lookup table

## Project Overview

AnyRepair is a Rust crate for repairing malformed LLM-generated content across 8 formats: JSON, YAML, XML, TOML, CSV, INI, Markdown, and Diff/unified diff. It provides auto-detection, Python jsonrepair-compatible API, MCP server integration, and enterprise features (analytics, batch processing, custom validation, audit logging).

## Development Commands

### Build and Test
```bash
# Standard build (debug)
cargo build

# Optimized release build (size-optimized binaries, ~1.5 MB)
cargo build --release

# Run all tests
cargo test

# Run specific test module
cargo test jsonrepair
cargo test yaml_tests
cargo test format_detection

# Run benchmarks
cargo bench repair_benchmarks

# Install CLI tool locally
cargo install --path .

# Run MCP server
cargo run --bin anyrepair-mcp
```

### Binaries
- `anyrepair` - Main CLI tool ([src/main.rs](src/main.rs))
- `anyrepair-mcp` - MCP server for Claude integration ([src/bin/mcp_server.rs](src/bin/mcp_server.rs))

### Build Profiles
```bash
# Standard release (size-optimized)
cargo build --release

# Distribution profile (maximum size optimization, fat LTO)
cargo build --profile dist
```

The `dist` profile ([Cargo.toml:67-73](Cargo.toml:67-73)) provides:
- Inherits `release` settings
- `opt-level = "z"` (optimize for size)
- Fat LTO for cross-crate optimization
- Single codegen unit for better optimization
- Stripped symbols and panic=abort for minimal binary size

### Testing Tools
- `insta` - Snapshot testing (run `cargo test` then review with `cargo insta review`)
- `criterion` - Benchmarking (`cargo bench`)
- `proptest` - Property-based fuzz testing

### CLI Commands Reference

The `anyrepair` CLI provides these subcommands:

```bash
# Auto-detect and repair content (supports all 8 formats)
anyrepair repair <input_file>

# Format-specific repair commands
anyrepair json <input_file>
anyrepair yaml <input_file>
anyrepair markdown <input_file>
anyrepair xml <input_file>
anyrepair toml <input_file>
anyrepair csv <input_file>
anyrepair ini <input_file>
anyrepair diff <input_file>

# Validation without repair
anyrepair validate <input_file> [--format <format>]

# Batch process multiple files
anyrepair batch <input_path> [--recursive] [--filter <pattern>]

# Custom rules management
anyrepair rules add --id <rule_id> --format <format> --pattern <pattern> --replacement <replacement>
anyrepair rules list
anyrepair rules remove <rule_id>
anyrepair rules enable <rule_id>
anyrepair rules disable <rule_id>

# Stream large files
anyrepair stream <input_file> <output_file> [--format <format>] [--buffer-size <size>]

# Statistics and analytics
anyrepair stats

# Plugin management
anyrepair plugins list
anyrepair plugins load <plugin_path>
anyrepair plugins unload <plugin_id>
```

All commands support:
- `--format, -f <format>` - Override auto-detection with specific format
- `--output, -o <file>` - Write output to file instead of stdout
- `--verbose, -v` - Enable detailed logging

## Architecture Overview

### Core Design Pattern: Strategy-Based Repair System

The codebase uses a **trait-based strategy pattern** that is central to understanding how repairs work:

1. **Entry Point** ([src/lib.rs](src/lib.rs:74-108)): `repair()` function auto-detects format via `is_*_like()` functions and routes to appropriate repairer
2. **Repair Trait** ([src/traits.rs](src/traits.rs)): All format repairers implement `Repair`, `RepairStrategy`, and `Validator` traits
3. **Format Repairers**: Single-file modules ([json.rs](src/json.rs), [yaml.rs](src/yaml.rs), etc.) contain:
   - Repairer struct (e.g., `JsonRepairer`)
   - Multiple strategy structs (e.g., `StripTrailingCommasStrategy`)
   - Validator for format-specific validation
4. **Base Repairer** ([src/repairer_base.rs](src/repairer_base.rs)): Provides DRY-compliant base implementation

### Format Detection Priority Order

The `repair()` function in [lib.rs:74-108](src/lib.rs:74-108) checks formats in this priority order:
1. JSON (`{}` or `[]` patterns)
2. YAML (`:`, `---`, or key-value patterns)
3. XML (`<` tags, `<?xml`)
4. TOML (`[table]`, `key = value`)
5. CSV (commas, multiple lines)
6. INI (`[section]`, `key = value`)
7. Diff (`@@` hunks, `+++`/`---` prefixes, `+`/`-`/` ` line prefixes)
8. Markdown (defaults for unknown)

**Key**: Detection is heuristic-based and may have edge cases. Unknown formats default to Markdown repair.

### Diff/Unified Diff Format Specifics

Diff format ([diff.rs](src/diff.rs)) is the newest format addition and has unique repair patterns:

**Detection Patterns** ([lib.rs:324-350](src/lib.rs:324-350)):
- Hunk headers: `@@ -from,to +from,to @@`
- File headers: Lines starting with `---` or `+++`
- Line prefixes: `+` (added), `-` (removed), ` ` (context)
- Threshold: If >30% of lines have diff prefixes, detected as diff

**Common Diff Issues Repaired**:
1. Missing hunk headers (`@@` lines) - added based on line counts
2. Incorrect line prefixes - normalized to `+`, `-`, or ` `
3. Malformed ranges - fixed hunk header line numbers
4. Missing file headers - `---`/`+++` added when absent
5. Whitespace issues - normalized spacing in hunks

**Diff Strategy Priorities**:
1. `AddMissingHunkHeadersStrategy` (priority 100) - Ensure all hunks have headers
2. `FixLinePrefixesStrategy` (priority 90) - Correct line prefix characters
3. `FixHunkRangesStrategy` (priority 80) - Repair malformed line number ranges
4. `AddFileHeadersStrategy` (priority 70) - Add missing file headers
5. `NormalizeWhitespaceStrategy` (priority 60) - Fix spacing issues

**Example Diff Repair**:
```rust
use anyrepair::DiffRepairer;

let malformed = r"@@ -1,3 +1,4 @@
-line 1
+line 1 modified
 line 2
+line 3";

let mut repairer = DiffRepairer::new();
let repaired = repairer.repair(malformed)?;
// Adds missing file headers, fixes hunk ranges if needed
```

### Strategy Pattern Implementation

Each format repairer uses **priority-ordered strategies**:
- Strategies implement `RepairStrategy` trait with `apply()` and `priority()` methods
- Higher priority strategies run first
- Strategies are applied sequentially until validation passes
- Example from JSON: `StripTrailingContentStrategy` (priority 100) runs before `AddMissingQuotesStrategy` (priority 90)

### Python-Compatible API

Two APIs match Python's `jsonrepair` library for drop-in migration:

**1. Function-based API** ([lib.rs:134](src/lib.rs:134)):
```rust
use anyrepair::jsonrepair;

// Simple function call
let repaired = jsonrepair(r#"{"key": "value",}"#)?;
```

**2. Class-based API** ([lib.rs:158](src/lib.rs:158)):
```rust
use anyrepair::JsonRepair;

// Reusable instance
let mut jr = JsonRepair::new();
let repaired1 = jr.jsonrepair(r#"{"key": "value",}"#)?;
let repaired2 = jr.jsonrepair(r#"{name: "John"}"#)?;
```

**Enhanced JSON API** (drop-in for `json.loads()`):
```rust
use anyrepair::{loads, load, from_file, repair_json_advanced};

// Drop-in replacements for Python's json module
let value = loads(r#"{name: "John", age: 30}"#)?;
let value = load(reader)?;
let value = from_file("config.json")?;

// Advanced repair with options
let value = repair_json_advanced(
    json_str,
    skip_json_loads: true,  // Skip validation for performance
    logging: true,          // Enable detailed logging
    stream_stable: true     // Enable streaming support
)?;

// Repair with logging
let (value, log) = repair_json_with_logging(json_str)?;
// log contains vector of repair operations performed
```

## Module Organization

### Format-Specific Repairers (Single-File Pattern)
Each format is self-contained in a single module file:
- [json.rs](src/json.rs) - JSON repair (571 lines, 8 strategies)
- [yaml.rs](src/yaml.rs) - YAML repair
- [markdown.rs](src/markdown.rs) - Markdown repair (~550 lines)
- [xml.rs](src/xml.rs) - XML repair
- [toml.rs](src/toml.rs) - TOML repair
- [csv.rs](src/csv.rs) - CSV repair
- [ini.rs](src/ini.rs) - INI repair
- [diff.rs](src/diff.rs) - Diff/unified diff repair

### Core Infrastructure
- [traits.rs](src/traits.rs) - `Repair`, `RepairStrategy`, `Validator` trait definitions
- [repairer_base.rs](src/repairer_base.rs) - Base repairer implementation (DRY compliance)
- [error.rs](src/error.rs) - `RepairError` enum with comprehensive error types

### Enterprise Features
- [analytics.rs](src/analytics.rs) - Repair metrics tracking (success rates, performance)
- [batch_processor.rs](src/batch_processor.rs) - Multi-file batch processing
- [validation_rules.rs](src/validation_rules.rs) - Custom validation rule engine
- [audit_log.rs](src/audit_log.rs) - Compliance logging

### Enterprise Features Deep Dive

**AnalyticsTracker** ([analytics.rs](src/analytics.rs)):
- Tracks total repairs, successful repairs, failed repairs
- Monitors performance: average repair time, total repair time
- Per-format statistics breakdown
- Success rate calculation and reporting
- Usage: `AnalyticsTracker::new()` then `track_repair_result()`, `get_metrics()`

**BatchProcessor** ([batch_processor.rs](src/batch_processor.rs)):
- Process entire directories or multiple files
- Recursive or single-level directory traversal
- File filtering by extension pattern
- Automatic format detection per file
- Returns detailed per-file results (success/failure, format used, repair time)
- Integrated analytics tracking for batch operations

**ValidationRulesEngine** ([validation_rules.rs](src/validation_rules.rs)):
- Multiple rule types: Regex, Length, Format, Custom
- Rule management: add, remove, enable/disable rules
- Validates content against all active rules
- Returns detailed violation reports
- Supports conditional rules with format targeting

**AuditLogger** ([audit_log.rs](src/audit_log.rs)):
- Event types: repairs, validations, batch operations, config changes
- Detailed entries: timestamp, actor, resource, action, result
- File persistence for compliance requirements
- Query capabilities: filter by event type or actor
- JSON format for easy parsing and integration

**ConfidenceScorer** ([confidence_scorer.rs](src/confidence_scorer.rs)):
- Assesses repair confidence on scale 0.0-1.0
- Factors: validation result, number of strategies applied, format detection score
- Used to determine if manual review is needed
- Higher confidence = more likely repair is correct

### Advanced Features
- [streaming.rs](src/streaming.rs) - Large file streaming support
- [plugin.rs](src/plugin.rs) + [plugin_config.rs](src/plugin_config.rs) + [plugin_integration.rs](src/plugin_integration.rs) - Plugin system
- [parallel.rs](src/parallel.rs) + [parallel_strategy.rs](src/parallel_strategy.rs) - Multi-threaded strategy application
- [enhanced_json.rs](src/enhanced_json.rs) - Advanced JSON parsing with repair
- [context_parser.rs](src/context_parser.rs) - Context-aware parsing
- [confidence_scorer.rs](src/confidence_scorer.rs) - Repair confidence assessment

### Plugin System

The plugin architecture ([plugin.rs](src/plugin.rs), [plugin_config.rs](src/plugin_config.rs), [plugin_integration.rs](src/plugin_integration.rs)) enables custom repair strategies:

**Plugin Trait**:
```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn execute(&self, content: &str, context: &PluginContext) -> Result<String>;
    fn supports_format(&self, format: &str) -> bool;
}
```

**Plugin Configuration** (TOML):
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

**Plugin Lifecycle**:
1. Load: `PluginManager::load_plugin(path)`
2. Register: `PluginRegistry::register(plugin)`
3. Execute: Automatically applied during repair if format matches
4. Unload: `PluginManager::unload_plugin(plugin_id)`

**CLI Usage**:
```bash
# List loaded plugins
anyrepair plugins list

# Load a plugin
anyrepair plugins load ./plugins/custom_rules.so

# Unload a plugin
anyrepair plugins unload custom_rules
```

See [docs/PLUGIN_DEVELOPMENT.md](docs/PLUGIN_DEVELOPMENT.md) for complete plugin development guide.

### Custom Rules System

The custom rules system ([config.rs](src/config.rs), [custom_rules.rs](src/custom_rules.rs)) allows user-defined repair patterns:

**Rule Configuration** (TOML):
```toml
# Global settings
verbose = true
max_strategies = 10

# Format-specific settings
[formats.json]
enable_trailing_comma_removal = true
enable_single_quote_conversion = true

# Custom rules
[[rules]]
id = "fix_undefined"
format = "json"
pattern = "undefined"
replacement = "null"
priority = 90

[[rules]]
id = "fix_js_comments"
format = "json"
pattern = "//.*\\n"
replacement = ""
priority = 80
```

**Rule Types**:
- **Regex rules**: Pattern matching and replacement
- **Length rules**: Validate string/array lengths
- **Format rules**: Enforce specific format constraints
- **Custom rules**: User-defined validation logic

**CLI Management**:
```bash
# Add a new rule
anyrepair rules add \
  --id "fix_trailing_comma" \
  --format "json" \
  --pattern ",\\s*}" \
  --replacement "}" \
  --priority 95

# List all rules
anyrepair rules list

# Remove a rule
anyrepair rules remove "fix_trailing_comma"

# Enable/disable rules
anyrepair rules enable "fix_trailing_comma"
anyrepair rules disable "fix_trailing_comma"
```

**Programmatic Usage**:
```rust
use anyrepair::{RepairConfig, CustomRule, CustomRuleEngine};

// Create a custom rule
let rule = CustomRule::new("fix_undefined", "json")
    .with_pattern("undefined")
    .with_replacement("null")
    .with_priority(90);

// Load config from file
let config = RepairConfig::from_file("anyrepair.toml")?;

// Create rule engine from config
let engine = CustomRuleEngine::from_config(&config);

// Apply rules to content
let repaired = engine.apply_rules(content, &format)?;
```

### CLI
- [main.rs](src/main.rs) - CLI entry point (180 lines, heavily refactored)
- [cli/](src/cli/) directory - Modular command handlers (repair, validate, batch, rules, stream)

## Adding New Format Support

When adding a new format, follow this pattern:

1. Create new module file (e.g., `src/newformat.rs`)
2. Implement these three traits:
   - `Repair` trait with `repair()`, `needs_repair()`, `confidence()` methods
   - `RepairStrategy` trait for each repair strategy
   - `Validator` trait for format validation
3. Add detection function to `lib.rs` (e.g., `is_newformat_like()`)
4. Add branch to `repair()` function in `lib.rs`
5. Export in `lib.rs` with `pub mod newformat;`
6. Add CLI subcommand in `main.rs`
7. Add comprehensive tests (unit + snapshot)

## Adding New Repair Strategies

For existing formats:
1. Create strategy struct implementing `RepairStrategy` trait
2. Set appropriate `priority()` (higher = runs first)
3. Implement `apply()` returning `Result<String>`
4. Add to repairer's strategy list in `new()` constructor
5. Add snapshot test for expected behavior

## Common Task Workflows

### Debugging a Failed Repair

When a repair fails or produces unexpected output:

1. **Enable logging**:
   ```rust
   let mut repairer = JsonRepairer::with_logging(true);
   let result = repairer.repair(malformed_json)?;
   let log = repairer.get_repair_log();
   ```

2. **Check confidence score**:
   ```rust
   let scorer = ConfidenceScorer::new();
   let confidence = scorer.score(&result, &format);
   if confidence < 0.8 {
       // Manual review may be needed
   }
   ```

3. **Validate output**:
   ```rust
   let validator = JsonValidator::new();
   if !validator.is_valid(&result) {
       let errors = validator.validate(&result);
       // Each error describes what's wrong
   }
   ```

4. **Test individual strategies**:
   ```rust
   let strategy = FixTrailingCommasStrategy::new();
   let partial_repair = strategy.apply(malformed_json)?;
   ```

### Adding a Custom Repair Strategy to JSON

Example: Adding a strategy to fix JavaScript-style comments in JSON

1. Create the strategy in [json.rs](src/json.rs):
   ```rust
   pub struct StripJsCommentsStrategy {
       regex: Regex,
   }

   impl StripJsCommentsStrategy {
       pub fn new() -> Self {
           Self {
               regex: Regex::new(r"//[^\n]*\n").unwrap(),
           }
       }
   }

   impl RepairStrategy for StripJsCommentsStrategy {
       fn apply(&self, content: &str) -> Result<String> {
           Ok(self.regex.replace_all(content, "").to_string())
       }

       fn priority(&self) -> u8 {
           95  // Run early, before quote fixing
       }
   }
   ```

2. Add to `JsonRepairer::new()` in strategy list

3. Add snapshot test:
   ```rust
   #[test]
   fn test_strip_js_comments() {
       let strategy = StripJsCommentsStrategy::new();
       let input = r#"{"key": "value", // comment\n}"#;
       let result = strategy.apply(input).unwrap();
       insta::assert_snapshot!(result);
   }
   ```

### Processing Large Files with Streaming

For files that don't fit in memory:

```rust
use anyrepair::StreamingRepair;
use std::fs::File;
use std::io::BufReader;

let input = BufReader::new(File::open("large_file.json")?);
let mut output = File::create("repaired.json")?;

// Configure buffer size (default 8192)
let processor = StreamingRepair::with_buffer_size(65536);

// Process with automatic format detection
processor.process(input, &mut output, None)?;

// Or specify format explicitly
processor.process(input, &mut output, Some("json"))?;
```

CLI usage:
```bash
anyrepair stream large_input.json repaired_output.json --format json --buffer-size 65536
```

## Testing Conventions

- **Unit tests**: Inline in each module file
- **Snapshot tests**: Use `insta` crate, files in `tests/snapshots/`
- **Integration tests**: In `tests/` directory
- **Fuzz tests**: Use `proptest` for property-based testing
- Run `cargo test` then review snapshots with `cargo insta review`

## MCP Server Integration

The MCP server ([mcp_server.rs](src/mcp_server.rs), [src/bin/mcp_server.rs](src/bin/mcp_server.rs)) provides 10 tools:
- `repair` - Auto-detect and repair
- `repair_json`, `repair_yaml`, `repair_markdown`, `repair_xml`, `repair_toml`, `repair_csv`, `repair_ini`, `repair_diff` - Format-specific repair
- `validate` - Validate content without repair

Configure in Claude Desktop's `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "anyrepair": {
      "command": "anyrepair-mcp"
    }
  }
}
```

## Performance Optimizations

- **Regex caching**: Cached regex patterns provide ~99.6% performance improvement
- **Binary size optimization**: Release profile uses `opt-level = "z"`, LTO, single codegen unit
- **Parallel processing**: `rayon` for multi-threaded strategy application
- **Streaming**: Process files larger than available RAM using `StreamingRepair`

## Documentation Structure

- [README.md](README.md) - Quick start and overview
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - Detailed system design
- [docs/MCP_SERVER.md](docs/MCP_SERVER.md) - MCP integration guide
- [docs/PLUGIN_DEVELOPMENT.md](docs/PLUGIN_DEVELOPMENT.md) - Plugin development
- [docs/TEST_SUMMARY.md](docs/TEST_SUMMARY.md) - Test coverage (364+ tests)
- [docs/INDEX.md](docs/INDEX.md) - Complete documentation index

## Key Dependencies

- **Core**: `serde`, `serde_json`, `serde_yaml`, `regex`, `thiserror`, `anyhow`
- **CLI**: `clap`, `tokio`, `futures`
- **Formats**: `pulldown-cmark` (Markdown), `quick-xml` (XML), `toml` (TOML), `csv` (CSV), `ini` (INI)
- **MCP**: `rmcp` (Model Context Protocol)
- **Testing**: `insta`, `criterion`, `tempfile`, `proptest`, `arbitrary`

## Important Constraints

- **Rust Edition 2024**: Uses latest Rust features and syntax
- **Single-file pattern**: Format repairers should be single files, avoid subdirectories unless necessary (see [docs/MODULIZATION_GUIDE.md](docs/MODULIZATION_GUIDE.md))
- **Trait implementation**: All repairers must implement the `Repair` trait
- **Validation**: Must use format-specific parsers (e.g., `serde_json::from_str` for JSON, `serde_yaml::from_str` for YAML)
- **Error handling**: Use `thiserror` for automatic trait implementations and proper error chaining
- **Strategy priorities**: Higher priority values run first (100 = highest, typical range 60-100)
- **DRY compliance**: Use `GenericRepairer` base when appropriate to avoid code duplication
- **Snapshot tests**: All new strategies must have snapshot tests for regression prevention
- **Format detection order**: When adding new formats, insert detection in priority order in `repair()` function

## Quick Reference

| Task | Command/API | Location |
|------|-------------|----------|
| Auto-repair content | `anyrepair repair <file>` or `repair(content)` | [lib.rs:75](src/lib.rs:75) |
| JSON repair only | `anyrepair json <file>` or `JsonRepairer::new()` | [json.rs](src/json.rs) |
| Python-compatible API | `jsonrepair(str)` or `JsonRepair::new().jsonrepair(str)` | [lib.rs:134](src/lib.rs:134) |
| Batch process files | `anyrepair batch <path>` | [batch_processor.rs](src/batch_processor.rs) |
| Stream large files | `anyrepair stream <in> <out>` or `StreamingRepair` | [streaming.rs](src/streaming.rs) |
| Custom rules | `anyrepair rules add/list/remove` | [custom_rules.rs](src/custom_rules.rs) |
| Run tests | `cargo test` | Throughout codebase |
| Run MCP server | `cargo run --bin anyrepair-mcp` | [src/bin/mcp_server.rs](src/bin/mcp_server.rs) |
| Build release | `cargo build --release` | [Cargo.toml:59](Cargo.toml:59) |
| Build dist | `cargo build --profile dist` | [Cargo.toml:67](Cargo.toml:67) |

## File Size Metrics

| Format Repairer | Lines (approx) | Strategies |
|-----------------|----------------|------------|
| JSON ([json.rs](src/json.rs)) | 600+ | 9 |
| Markdown ([markdown.rs](src/markdown.rs)) | ~550 | 6 |
| Diff ([diff.rs](src/diff.rs)) | ~400 | 5 |
| YAML ([yaml.rs](src/yaml.rs)) | ~300 | 5 |
| XML ([xml.rs](src/xml.rs)) | ~250 | 4 |
| TOML ([toml.rs](src/toml.rs)) | ~200 | 4 |
| CSV ([csv.rs](src/csv.rs)) | ~180 | 3 |
| INI ([ini.rs](src/ini.rs)) | ~150 | 3 |
