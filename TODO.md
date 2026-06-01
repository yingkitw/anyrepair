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
- [ ] **CHANGELOG for 0.2.6** — Document lean-deps release and validator changes
- [ ] **Code coverage** — More edge cases for properties/env and heuristic validators
- [ ] **Performance regression tests** — Benchmark gate in CI (`criterion` benches exist)

### Medium
- [ ] **Auto-detect properties/env** — Heuristics in `format_detection.rs` without breaking INI
- [ ] **MCP binary version** — Align `anyrepair-mcp` server info string with crate version
- [ ] **Update Cargo.toml description** — Mention 10 formats and properties/env
- [ ] **Refresh docs/TEST_SUMMARY.md** — Match current 316-test breakdown and deps

## Planned Features 📋

### Formats
- [ ] **Protobuf** — Binary/text protobuf repair (research scope)

### CLI
- [ ] **Diff preview** — `--diff` before applying repairs
- [ ] **Dry-run** — `--dry-run` without writing output
- [ ] **Colored output** — Syntax-highlighted stdout
- [ ] **JSON output mode** — Machine-readable results for CI
- [ ] **Shell completions** — bash/zsh/fish
- [ ] **Restore custom rules CLI** — Wire `anyrepair.toml` or remove sample config

### Repair quality
- [ ] **Format-preserving repairs** — Whitespace, comments, key order
- [ ] **Repair explanations** — What changed and why
- [ ] **Configurable confidence thresholds**
- [ ] **Stronger validators** — Optional strict mode (e.g. optional parser deps behind feature flags)

### Platform
- [ ] **Web UI** — Browser-based repair
- [ ] **REST API** — HTTP access
- [ ] **Docker image** — Containerized CLI/MCP

### Documentation
- [ ] **Rustdoc pass** — Full public API on docs.rs (including `json_util`)
- [ ] **Troubleshooting guide** — Common failures and fixes
- [ ] **Sync SPEC.md** — Dependency table and validator notes for 0.2.6

### Testing
- [ ] **Dedicated properties/env integration tests** — Beyond `key_value` unit tests
- [ ] **Mutation testing** — `cargo-mutants` on critical paths
- [ ] **Golden master repairs** — Checked-in expected outputs

## Technical Debt 🔧

- [ ] **Clippy** — `-D warnings` in CI
- [ ] **Prune stale docs** — `docs/ARCHITECTURE.md` pointer vs root `ARCHITECTURE.md`
- [ ] **Remove or implement `anyrepair.toml`** — Sample custom rules file is not loaded by current code
- [ ] **Review heuristic validator false positives/negatives** — Especially XML and CSV after parser removal

## Ideas 💡

- Optional `full-validation` Cargo feature restoring `serde_json` / format parsers for strict checks
- Language bindings (Python, Node.js, Go)
- gRPC/WebSocket streaming repair API
- Format-detection confidence exposed in API

## Known Issues (Fixed) ✅

- [x] YAML validator too permissive (partially addressed with structural checks)
- [x] Per-format CLI subcommands duplicated logic
- [x] Unused dependencies and compilation warnings
- [x] `insta` snapshot clutter in tests
- [x] Heavy transitive dependency tree from serde/XML/TOML/CSV crates

---

See also: [ARCHITECTURE.md](ARCHITECTURE.md) · [SPEC.md](SPEC.md) · [docs/CHANGELOG.md](docs/CHANGELOG.md)
