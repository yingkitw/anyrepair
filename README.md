# AnyRepair

[![GitHub stars](https://img.shields.io/github/stars/yingkitw/anyrepair?style=social)](https://github.com/yingkitw/anyrepair)

A Rust crate for repairing malformed structured data across **10 formats** (JSON, YAML, Markdown, XML, TOML, CSV, INI, Diff, Java properties, and `.env`).

## Quick Start

### Installation

```toml
[dependencies]
anyrepair = "0.2.9"
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
anyrepair repair input.json --format json
anyrepair repair input.yaml --format yaml
anyrepair repair config.ini --format ini
anyrepair repair app.properties --format properties
anyrepair repair .env --format env

# Show confidence score
anyrepair repair input.json --format json --confidence

# Preview changes as a diff (no output written)
anyrepair repair input.json --format json --diff --dry-run

# Dry-run without writing output
anyrepair repair input.json --format json --dry-run

# Machine-readable JSON output for CI
anyrepair repair input.json --format json --json --dry-run

# Require minimum confidence (exit with error if below threshold)
anyrepair repair input.json --format json --min-confidence 0.8

# Show which repair strategies were applied
anyrepair repair input.json --format json --explain --dry-run

# Generate shell completions
anyrepair completions bash > /etc/bash_completion.d/anyrepair
anyrepair completions zsh > _anyrepair
anyrepair completions fish > ~/.config/fish/completions/anyrepair.fish

# Batch process multiple files
anyrepair batch --input ./data --output ./repaired --recursive

# Stream large files
anyrepair stream --input large_file.json --output repaired.json --format json

# Validation without repair
anyrepair validate --input input.json --format json
```

### Docker

```bash
docker build -t anyrepair .
docker run --rm -i anyrepair repair --format json <<'EOF'
{"name": "John", age: 30,}
EOF

# MCP server
docker run --rm -i anyrepair anyrepair-mcp
```

## What's New

### v0.2.9 — Current

- **`detect_format_with_confidence`** — format detection returns a confidence score (`DetectionResult`)
- **Docker image** — multi-stage `Dockerfile` for CLI and MCP binaries
- **MCP version** — `anyrepair-mcp` reports `CARGO_PKG_VERSION` (no more stale hardcoded version)
- Integration tests for smart quotes, boolean variants, and prose extraction
- **432 tests**, all passing

### v0.2.8

- CLI: `--diff`, `--dry-run`, `--json`, `--min-confidence`, `--explain`, `--color`
- Shell completions: `anyrepair completions <shell>`
- LLM JSON strategies: smart quotes, boolean variants (`yes`/`no`/`on`/`off`), prose/preamble extraction
- Optional `strict` feature for `serde_json`-backed JSON validation
- Golden master + properties/env integration tests

### v0.2.7

- **Auto-detect properties & env** — `detect_format()` now recognizes `.properties` and `.env` files
- **Heuristic validator fixes** — XML / CSV edge cases
- **Criterion benchmarks** — All 10 formats + format detection + large-document throughput

### v0.2.6

- **Minimal dependencies** and **`json_util`** without serde
- Heuristic validators for XML, TOML, CSV, and YAML

### v0.2.0–0.2.5 (highlights)

- Properties/`.env` formats, centralized registry, Python-compatible `jsonrepair()`, MCP, streaming

See [CHANGELOG.md](docs/CHANGELOG.md) for full version history.

## Why AnyRepair?

Structured data from LLMs, APIs, or manual editing is often malformed. AnyRepair fixes common issues:

- **JSON**: Missing quotes, trailing commas, syntax errors
- **YAML**: Indentation, missing colons
- **Markdown**: Headers, links, fences
- **XML / TOML / CSV / INI / Diff**: Format-specific repairs
- **Properties / `.env`**: Key=value lines, sections, escaping

**Key features:**

- Auto-detects format for all 10 formats
- Deterministic heuristic repairs (no network, no ML)
- Small dependency footprint (four runtime crates)
- MCP server for Claude and other MCP clients
- Streaming for large files
- Python-compatible JSON API
- Optional `strict` feature for full `serde_json` parser validation
- 415 tests (`cargo test`)

## Dependencies

| Kind | Crates |
|------|--------|
| **Runtime** | `regex`, `thiserror`, `clap`, `clap_complete` |
| **Optional** | `serde_json` (via `strict` feature) |
| **Dev** | `criterion`, `arbitrary`, `proptest` |

Parsing and validation for JSON, XML, TOML, CSV, and YAML use in-crate heuristics and `json_util` rather than heavyweight parser dependencies. Enable the `strict` feature for `serde_json`-backed JSON validation:

```bash
cargo build --features strict
cargo test --features strict
```

## Usage Examples

### Multi-Format Auto-Detection

```rust
use anyrepair::repair;

let json = repair(r#"{"key": value,}"#)?;
let yaml = repair("name: John\nage: 30")?;
let markdown = repair("# Header\n[link](url")?;
```

### Explicit key-value formats

```rust
use anyrepair::repair_with_format;

let props = repair_with_format("db.url jdbc:postgresql://localhost", "properties")?;
let env = repair_with_format("API_KEY = secret", "env")?;
```

### Python-Compatible JSON API

```rust
use anyrepair::{jsonrepair, JsonRepair};

let repaired = jsonrepair(r#"{"name": "John", age: 30,}"#)?;

let mut jr = JsonRepair::new();
let repaired = jr.jsonrepair(r#"{name: "John"}"#)?;
```

### Format-Specific Repairers

```rust
use anyrepair::{create_repairer, repair_with_format, traits::Repair};

let mut repairer = create_repairer("json")?;
let repaired = repairer.repair(malformed_json)?;
let confidence = repairer.confidence(&repaired);

let repaired = repair_with_format(malformed_yaml, "yaml")?;
```

### Streaming Large Files

```rust
use anyrepair::StreamingRepair;
use std::fs::File;
use std::io::BufReader;

let input = BufReader::new(File::open("large_file.json")?);
let mut output = File::create("repaired.json")?;

let processor = StreamingRepair::with_buffer_size(65536);
processor.process(input, &mut output, Some("json"))?;
```

### MCP Server Integration

```bash
cargo install anyrepair
anyrepair-mcp
```

**`claude_desktop_config.json`:**

```json
{
  "mcpServers": {
    "anyrepair": {
      "command": "anyrepair-mcp"
    }
  }
}
```

**Tools:** `repair`, `repair_json`, `repair_yaml`, `repair_markdown`, `repair_xml`, `repair_toml`, `repair_csv`, `repair_ini`, `repair_diff`, `repair_properties`, `repair_env`, `validate`.

See [MCP_SERVER.md](docs/MCP_SERVER.md) for setup details.

## Supported Formats

| Format | Common issues fixed | Auto-detect |
|--------|---------------------|-------------|
| **JSON** | Quotes, commas, booleans/null | Yes |
| **YAML** | Indentation, colons, lists | Yes |
| **Markdown** | Headers, links, fences | Yes |
| **XML** | Tags, attributes, entities | Yes |
| **TOML** | Quotes, arrays, tables | Yes |
| **CSV** | Quoting, commas | Yes |
| **INI** | Sections, `=` signs | Yes |
| **Diff** | Hunk headers, line prefixes | Yes |
| **Properties** | `key=value`, escaping, continuations | Yes |
| **Env** | `KEY=value`, comments, quoting | Yes |

## Performance

- Strategy pipeline with priority ordering
- Release profile optimized for size (`opt-level = "z"`, LTO, strip)
- Streaming for files larger than RAM
- Fewer transitive dependencies for faster builds and smaller binaries

```bash
cargo build --release
cargo build --profile dist   # distribution profile in Cargo.toml
```

## Testing

```bash
cargo test
```

| Suite | Tests |
|-------|------:|
| Library / modules | 177 |
| Completions / CLI extras | 19 |
| CLI | 15 |
| Integration | 21 |
| Diff | 35 |
| Fuzz (proptest) | 34 |
| Streaming | 26 |
| Properties/env | 25 |
| Golden master | 26 |
| Damage + complex damage + complex streaming | 54 |
| **Total** | **432** |

See [TEST_SUMMARY.md](docs/TEST_SUMMARY.md) for more detail.

## Comparison

| Feature | AnyRepair | json-repair-rs | json5 | Python jsonrepair |
|---------|-----------|----------------|-------|-------------------|
| **Multi-format** | 10 formats | JSON only | JSON only | JSON only |
| **Auto-detection** | 10 formats | No | No | No |
| **Lean deps** | 3 runtime crates | Varies | Varies | N/A |
| **MCP** | Yes (12 tools) | No | No | No |
| **Streaming** | Yes | No | No | No |
| **Python JSON API** | Compatible | No | No | Native |
| **Language** | Rust | Rust | Rust | Python |

## Documentation

| Doc | Description |
|-----|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | System design |
| [SPEC.md](SPEC.md) | Technical specification |
| [TODO.md](TODO.md) | Roadmap |
| [TROUBLESHOOTING.md](TROUBLESHOOTING.md) | Common failures and fixes |
| [docs/CHANGELOG.md](docs/CHANGELOG.md) | Version history |
| [docs/MCP_SERVER.md](docs/MCP_SERVER.md) | MCP integration |
| [docs/TEST_SUMMARY.md](docs/TEST_SUMMARY.md) | Test breakdown |
| [docs/INDEX.md](docs/INDEX.md) | Documentation index |

**Links:** [GitHub Issues](https://github.com/yingkitw/anyrepair/issues) · [crates.io](https://crates.io/crates/anyrepair) · [docs.rs](https://docs.rs/anyrepair)

## Examples

```bash
cargo run --example mcp_repair_json
cargo run --example mcp_multi_format
```

See [examples/](examples/) and [examples/README.md](examples/README.md).

## Roadmap

See [TODO.md](TODO.md) for planned work (Protobuf, CLI polish, web/API, format-preserving repairs, and more).

## License

Apache-2.0

## Repository

**If AnyRepair helps you, consider starring the repo on GitHub** — it helps others find the project.

https://github.com/yingkitw/anyrepair
