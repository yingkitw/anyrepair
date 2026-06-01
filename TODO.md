# TODO

## Recently Completed ✅

### v0.2.5
- [x] Bump crate version to 0.2.5
- [x] **Properties** (`.properties`) and **Env** (`.env`) repair via `key_value.rs`
- [x] Consolidate INI with properties/env; remove `ini` crate dependency
- [x] MCP tools auto-registered for all 10 formats (12 tools total)
- [x] Sync root docs (README, SPEC, ARCHITECTURE, TODO)

### v0.2.x
- [x] KISS/DRY/SoC refactoring — centralized format registry, unified CLI
- [x] Eight core formats + diff; later expanded to ten with key-value module
- [x] Python-compatible API (`jsonrepair()`, `JsonRepair`)
- [x] Streaming support for large files
- [x] MCP server implementation
- [x] Fuzz testing with proptest
- [x] Zero-warning release builds; dependency cleanup (e.g. `pulldown-cmark`, `anyhow`)
- [x] **316 tests** with 100% pass rate (`cargo test`)

## Current Priorities 🚀

### High
- [ ] **Code coverage** — More edge cases for properties/env and cross-format detection
- [ ] **Performance regression tests** — Benchmark gate in CI
- [ ] **Real-world corpus** — User-submitted malformed samples

### Medium
- [ ] **Auto-detect properties/env** — Heuristics in `format_detection.rs` without breaking INI
- [ ] **CHANGELOG entry for 0.2.5** — Document properties/env and dependency changes
- [ ] **MCP binary version** — Align `anyrepair-mcp` server info with crate version (currently stale in bin)

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

### Platform
- [ ] **Web UI** — Browser-based repair
- [ ] **REST API** — HTTP access
- [ ] **Docker image** — Containerized CLI/MCP

### Documentation
- [ ] **Rustdoc pass** — Full public API on docs.rs
- [ ] **Troubleshooting guide** — Common failures and fixes
- [ ] **Refresh docs/TEST_SUMMARY.md** — Match 316-test breakdown

### Testing
- [ ] **Dedicated properties/env integration tests** — Beyond `key_value` unit tests
- [ ] **Mutation testing** — `cargo-mutants` on critical paths
- [ ] **Golden master repairs** — Checked-in expected outputs

## Technical Debt 🔧

- [ ] **Clippy** — `-D warnings` in CI
- [ ] **Description in Cargo.toml** — Mention properties/env in package description
- [ ] **Prune stale docs** — `docs/ARCHITECTURE.md` vs root `ARCHITECTURE.md` (keep one canonical)
- [ ] **Remove or implement `anyrepair.toml`** — Sample custom rules file is not loaded by current code

## Ideas 💡

- Language bindings (Python, Node.js, Go)
- gRPC/WebSocket streaming repair API
- Format-detection confidence exposed in API

## Known Issues (Fixed) ✅

- [x] YAML validator too permissive
- [x] Per-format CLI subcommands duplicated logic
- [x] Unused dependencies and compilation warnings
- [x] `insta` snapshot clutter in tests

---

See also: [ARCHITECTURE.md](ARCHITECTURE.md) · [SPEC.md](SPEC.md) · [docs/CHANGELOG.md](docs/CHANGELOG.md)
