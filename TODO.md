# TODO

## Recently Completed ✅

### v0.2.6
- [x] Bump crate version to 0.2.6
- [x] **Minimal runtime deps** — `regex`, `thiserror`, `clap` only
- [x] Removed `serde`, `serde_json`, `serde_yaml`, `quick-xml`, `toml`, `csv`, `ini` from dependencies
- [x] Removed unused dev-deps `serde_json`, `tempfile`
- [x] **`json_util` module** — JSON validation, escaping, MCP request/response helpers without serde
- [x] Heuristic validators for XML, TOML, CSV, YAML (no external parser crates)
- [x] MCP and integration tests updated to use `json_util`
- [x] **316 tests** passing (`cargo test`)

### v0.2.5
- [x] **Properties** (`.properties`) and **Env** (`.env`) repair via `key_value.rs`
- [x] Consolidate INI with properties/env; remove `ini` crate dependency
- [x] MCP tools auto-registered for all 10 formats (12 tools total)
- [x] Sync root docs (README, SPEC, ARCHITECTURE, TODO)

### v0.2.x
- [x] KISS/DRY/SoC refactoring — centralized format registry, unified CLI
- [x] Python-compatible API (`jsonrepair()`, `JsonRepair`)
- [x] Streaming support for large files
- [x] MCP server implementation
- [x] Fuzz testing with proptest
- [x] Dependency cleanup (`pulldown-cmark`, `anyhow`, and others)

## Current Priorities 🚀

### High
- [x] **CHANGELOG for 0.2.6** — Document lean-deps release and validator changes
- [x] **Code coverage** — More edge cases for properties/env and heuristic validators (+15 unit tests, 342 total)
- [x] **Performance benchmarks** — `criterion` suite covering all 10 formats + format detection + large docs

### Medium
- [x] **Auto-detect properties/env** — Heuristics in `format_detection.rs` without breaking INI
- [x] **MCP binary version** — Align `anyrepair-mcp` server info string with crate version
- [x] **Update Cargo.toml description** — Mention 10 formats and properties/env
- [x] **Refresh docs/TEST_SUMMARY.md** — Match current 353-test breakdown and 10 formats

## Planned Features 📋

### Formats
- [ ] **Protobuf** — Binary/text protobuf repair (research scope)

### CLI
- [x] **Diff preview** — `--diff` shows unified diff of changes
- [x] **Dry-run** — `--dry-run` repairs without writing output
- [x] **Colored output** — `--color auto|always|never` for diff and explain output
- [x] **JSON output mode** — `--json` outputs machine-readable JSON for CI
- [x] **Shell completions** — `anyrepair completions <shell>` (bash/zsh/fish/elvish/powershell)
- [x] **Restore custom rules CLI** — Resolved: `anyrepair.toml` removed, no custom rules engine planned

### Repair quality
- [ ] **Format-preserving repairs** — Whitespace, comments, key order
- [x] **Repair explanations** — `--explain` prints which repair strategies were applied
- [x] **Configurable confidence thresholds** — `--min-confidence <float>` exits with error if below threshold
- [x] **Stronger validators** — `strict` Cargo feature uses `serde_json` for full parser-based JSON validation

### Platform
- [ ] **Web UI** — Browser-based repair
- [ ] **REST API** — HTTP access
- [ ] **Docker image** — Containerized CLI/MCP

### Documentation
- [x] **Rustdoc pass** — Full public API documented (lib.rs, json_util, traits, streaming, repairer_base)
- [x] **Troubleshooting guide** — `TROUBLESHOOTING.md` with common failures and fixes
- [x] **Sync SPEC.md** — Fixed error handling section (removed serde refs), updated test count and dependency table

### Testing
- [x] **Dedicated properties/env integration tests** — 25 tests in `tests/properties_env_tests.rs`
- [ ] **Mutation testing** — `cargo-mutants` on critical paths
- [x] **Golden master repairs** — 26 checked-in expected outputs in `tests/golden_master_tests.rs`

## Technical Debt 🔧

- [x] **Compiler warnings** — Removed 3 dead functions from `json_util.rs`, fixed 7 unnecessary `mut` in `fuzz_tests.rs`
- [x] **Prune stale docs** — Updated `docs/ARCHITECTURE.md` pointer (v0.2.6, 353 tests)
- [x] **Remove `anyrepair.toml`** — Dead sample config file, unreferenced by code
- [x] **Review heuristic validator false positives/negatives** — Fixed XML validator content `=` false positive, XML entity corruption bug, CSV space→comma destructive replacement, removed 3 dead regex fields

## Ideas 💡

- Optional `full-validation` Cargo feature restoring `serde_json` / format parsers for strict checks
- Language bindings (Python, Node.js, Go)
- gRPC/WebSocket streaming repair API
- Format-detection confidence exposed in API

### Competitive intelligence (researched Jul 2026)
Competitors: `json_repair` (Python, 5K★), `jsonrepair` (JS, ecosystem leader), `repairjson` (Rust+PyO3, 200x faster), `safe-json-repair` (Rust+WASM), `@datatool/json-heal` (safe repair), `@isdk/json-repair.js` (schema-guided)

Capabilities competitors have that anyrepair lacks:
- [ ] **Smart quote normalization** — curly/typographic quotes → straight quotes (easy, high-value for LLM output)
- [ ] **Prose/preamble extraction** — extract JSON from surrounding LLM chatter (e.g. "Here's the result: {...}")
- [ ] **Boolean variant recognition** — `yes`/`no`, `on`/`off`, `1`/`0` → `true`/`false` in JSON
- [ ] **Schema-guided repair** — use JSON Schema to coerce types, resolve ambiguities, fill defaults
- [ ] **WASM bindings** — browser use via WebAssembly
- [ ] **Single-pass byte-oriented repair** — performance optimization for large inputs
- [ ] **Python bindings (PyO3)** — native Python extension for anyrepair
- [ ] **Hosted REST API** — `POST /api/repair` endpoint

## Known Issues (Fixed) ✅

- [x] YAML validator too permissive (partially addressed with structural checks)
- [x] Per-format CLI subcommands duplicated logic
- [x] Unused dependencies and compilation warnings
- [x] `insta` snapshot clutter in tests
- [x] Heavy transitive dependency tree from serde/XML/TOML/CSV crates

---

See also: [ARCHITECTURE.md](ARCHITECTURE.md) · [SPEC.md](SPEC.md) · [docs/CHANGELOG.md](docs/CHANGELOG.md)
