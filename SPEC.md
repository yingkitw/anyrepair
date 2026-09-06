# AnyRepair Specification

## Overview

AnyRepair is a deterministic, heuristic-based Rust library and CLI for repairing malformed structured data. It uses pattern matching and ordered repair strategies—no machine learning and no external API calls.

**Current version:** 0.2.11 (Rust edition 2024).

## Supported formats

| Format | Module | Aliases | Auto-detect |
|--------|--------|---------|-------------|
| JSON | `json.rs` | — | Yes |
| YAML | `yaml.rs` | `yml` | Yes |
| Markdown | `markdown.rs` | `md` | Yes |
| XML | `xml.rs` | — | Yes |
| TOML | `toml.rs` | — | Yes |
| CSV | `csv.rs` | — | Yes |
| INI | `key_value.rs` | — | Yes |
| Diff | `diff.rs` | — | Yes |
| Properties | `key_value.rs` | — | Yes |
| Env | `key_value.rs` | `.env` | Yes |
| PNG | `image.rs` | — | Yes (magic bytes) |
| JPEG | `image.rs` | `jpg` | Yes (magic bytes) |
| WAV | `audio.rs` | — | Yes (magic bytes) |
| MP3 | `audio.rs` | — | Yes (magic bytes) |
| MP4 | `video.rs` | — | Yes (magic bytes) |
| MOV | `video.rs` | — | Yes (magic bytes) |

Binary media formats are repaired via `BinaryRepair`/`BinaryValidator` (`binary_traits.rs`) with byte-level detection in `binary_detection.rs` (`detect_binary_format`).

## Public API (`src/lib.rs`)

```rust
pub const SUPPORTED_FORMATS: &[&str];
pub const BINARY_FORMATS: &[&str];

pub fn normalize_format(format: &str) -> &str;
pub fn create_repairer(format: &str) -> Result<Box<dyn Repair>>;
pub fn create_validator(format: &str) -> Result<Box<dyn Validator>>;
pub fn create_binary_repairer(format: &str) -> Result<Box<dyn BinaryRepair>>;
pub fn detect_format(content: &str) -> Option<&'static str>;
pub fn detect_format_with_confidence(content: &str) -> Option<DetectionResult>;
pub fn detect_binary_format(data: &[u8]) -> Option<BinaryDetectionResult>;
pub fn repair(content: &str) -> Result<String>;
pub fn repair_with_format(content: &str, format: &str) -> Result<String>;
pub fn repair_binary(data: &[u8]) -> Result<Vec<u8>>;
pub fn repair_binary_with_format(data: &[u8], format: &str) -> Result<Vec<u8>>;
pub fn jsonrepair(json_str: &str) -> Result<String>;
pub fn repair_with_explanations(content: &str, format: &str) -> Result<(String, Vec<String>)>;
```

Re-exports include format repairers (`JsonRepairer`, `IniRepairer`, `PropertiesRepairer`, `EnvRepairer`, …), `StreamingRepair`, `AnyrepairMcpServer`, `DetectionResult`, `RepairError`, and `Repair`.

### Registry extension

To add a format:

1. Add name to `SUPPORTED_FORMATS`.
2. Add arms in `create_repairer()` and `create_validator()`.
3. Implement repairer + validator (typically via `GenericRepairer`).
4. Optionally add `format_detection` heuristic.
5. Add tests; MCP tools are generated from `SUPPORTED_FORMATS` automatically.

## Traits

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

## Repairer composition

Format repairers compose `GenericRepairer` (not inheritance):

```
FormatRepairer {
    inner: GenericRepairer {
        strategies: Vec<Box<dyn RepairStrategy>>,
        validator: Box<dyn Validator>,
    }
}
```

- `repair()` / `needs_repair()` delegate to the inner pipeline.
- `confidence()` may use format-specific logic on the outer type.

## Format detection

Implemented in `format_detection.rs`. Detection order:

1. JSON — `{` / `[`
2. Diff — `@@` hunks, `---` / `+++` headers (before YAML/CSV)
3. YAML — `:`, `---`
4. XML — `<?xml`, tags
5. TOML — `[table]`, `key = value`
6. CSV — commas across lines
7. Env — `KEY=value` (uppercase keys)
8. Properties — `key=value` (dot keys, `!` comments)
9. INI — `[section]` headers
10. Markdown — headers, fences, emphasis

If `repair()` cannot detect a format, it uses the Markdown repairer as fallback.

## CLI

```
anyrepair repair [FILE] [--format <fmt>] [--confidence] [--diff] [--dry-run] [--json] [--min-confidence <float>] [--explain] [--color auto|always|never] [--input <file>] [--output <file>]
anyrepair validate [--input <file>] [--format <fmt>]
anyrepair stream [--input <file>] [--output <file>] [--format <fmt>] [--buffer-size <bytes>]
anyrepair batch --input <dir> --output <dir> [--pattern <glob>] [--recursive]
anyrepair repair-binary [FILE] [--format <fmt>] [--dry-run] [--input <file>] [--output <file>]
```

- `--format` accepts `SUPPORTED_FORMATS` plus `yml`, `md`; `repair-binary --format` accepts `BINARY_FORMATS` plus `jpg`.
- Without `--format`, repair uses auto-detection where supported.
- `--confidence` prints a repair confidence score (0–100%).
- `--diff` prints a unified diff of changes to stdout.
- `--dry-run` performs repair but does not write output (useful with `--diff`).
- `--json` outputs a machine-readable JSON result to stdout (for CI pipelines).
- `--min-confidence <float>` exits with error if repair confidence is below threshold (0.0–1.0).
- `--explain` prints the names of repair strategies that changed the content to stderr.
- `--color auto|always|never` controls ANSI color output for diff and explain (default: auto, detects TTY).

## Streaming

```rust
let processor = StreamingRepair::with_buffer_size(65536);
processor.process(reader, &mut writer, Some("json"))?;
```

## Python-compatible JSON API

```rust
pub fn jsonrepair(json_str: &str) -> Result<String>;
```

The repairer-based equivalent is `create_repairer("json")` (a `JsonRepairer`) used via the `Repair` trait.

## MCP

- Binary: `anyrepair-mcp`
- **12 tools:** `repair`, `repair_<format>` × 10, `validate`
- JSON-line protocol on stdin/stdout

## Error handling

```rust
pub enum RepairError {
    JsonRepair(String),
    YamlRepair(String),
    MarkdownRepair(String),
    FormatDetection(String),
    Io(std::io::Error),
    Utf8(std::string::FromUtf8Error),
    BinaryRepair(String),
    Generic(String),
}
```

Public functions return `crate::Result<T>` (`Result<T, RepairError>`).

## Design principles

- **DRY:** Central registry for dispatch.
- **SoC:** Detection, repair, CLI, and validation in separate modules.
- **KISS:** One `repair --format` command for all formats.
- **Deterministic:** Heuristic repairs only.
- **Composition:** `GenericRepairer` shared across formats.
- **Test-friendly:** Trait-based boundaries.

## Testing

| Category | Count | Location |
|----------|------:|----------|
| Library / module tests | 217 | `src/lib.rs` + modules |
| Binary (CLI handlers) | 20 | `src/main.rs` |
| CLI | 15 | `tests/cli_tests.rs` |
| Integration | 21 | `tests/integration_tests.rs` |
| Properties/env | 25 | `tests/properties_env_tests.rs` |
| Diff | 35 | `tests/diff_tests.rs` |
| Fuzz (proptest) | 34 | `tests/fuzz_tests.rs` |
| Streaming | 26 | `tests/streaming_tests.rs` |
| Damage scenarios | 18 | `tests/damage_scenarios.rs` |
| Complex damage | 18 | `tests/complex_damage_tests.rs` |
| Complex streaming | 18 | `tests/complex_streaming_tests.rs` |
| Corpus | 17 | `tests/corpus_tests.rs` |
| Binary integration | 23 | `tests/binary_tests.rs` |
| Binary comprehensive | 65 | `tests/binary_comprehensive_tests.rs` |
| Golden master | 26 | `tests/golden_master_tests.rs` |
| **Total** | **578** | `cargo test` |

## Dependencies (runtime)

No runtime crate dependencies — `anyrepair` uses only the Rust standard library.

Dev: `criterion`, `proptest`.

## Source layout

```
src/
├── lib.rs
├── main.rs
├── bin/mcp_server.rs
├── format_detection.rs
├── traits.rs
├── repairer_base.rs
├── error.rs
├── json.rs, yaml.rs, markdown.rs, xml.rs, toml.rs, csv.rs, diff.rs
├── key_value.rs          # ini, properties, env
├── image.rs, audio.rs, video.rs   # binary media repairers (PNG/JPEG, WAV/MP3, MP4/MOV)
├── binary_detection.rs   # magic-bytes detection
├── binary_traits.rs      # BinaryRepair, BinaryValidator
├── streaming.rs
├── mcp_server.rs
├── pattern.rs          # shared pattern-matching helpers (internal)
└── cli/
    ├── mod.rs
    ├── repair_cmd.rs
    ├── validate_cmd.rs
    ├── batch_cmd.rs
    ├── stream_cmd.rs
    ├── binary_cmd.rs
```
