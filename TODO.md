# TODO

## Recently Completed ✅

### v0.2.9
- [x] **`detect_format_with_confidence`** — expose detection confidence via `DetectionResult`
- [x] **Docker container** — multi-stage `Dockerfile` for `anyrepair` + `anyrepair-mcp`
- [x] **MCP binary version** — sync with `CARGO_PKG_VERSION`
- [x] Integration tests for smart quotes, boolean variants, prose extraction
- [x] Docs synced (README, SPEC, CHANGELOG, TODO); **450 tests**

### v0.2.8
- [x] CLI: `--diff`, `--dry-run`, `--json`, `--min-confidence`, `--explain`, `--color`
- [x] Shell completions (`clap_complete`)
- [x] LLM JSON strategies: smart quotes, boolean variants, prose extraction
- [x] `strict` feature flag; golden master + properties/env tests

### v0.2.6–0.2.7
- [x] Lean deps, `json_util`, heuristic validators
- [x] Auto-detect properties/env; criterion benches; validator bugfixes

## Current Priorities 🚀

### High
- [ ] **Mutation testing** — `cargo-mutants` on critical repair paths
- [x] **Real-world corpus** — checked-in samples from LLM / API failures

### Medium
- [ ] **Publish Docker image** — GHCR / Docker Hub CI workflow
- [x] **Expose detection confidence in CLI** — `repair --json` includes `detection_confidence`

## Planned Features 📋

### Formats
- [ ] **Protobuf** — Binary/text protobuf repair (research scope)

### Repair quality
- [ ] **Format-preserving repairs** — Whitespace, comments, key order
- [ ] **Schema-guided repair** — JSON Schema to coerce types / fill defaults

### Platform
- [ ] **Web UI** — Browser-based repair
- [ ] **REST API** — HTTP `POST /api/repair`
- [x] **Docker image** — Containerized CLI/MCP (`Dockerfile` in repo)

### Testing
- [ ] **Mutation testing** — `cargo-mutants` on critical paths
- [x] Golden master + properties/env suites

## Ideas 💡

- Language bindings (Python/PyO3, Node.js, Go)
- WASM bindings for browser use
- Single-pass byte-oriented repair for large inputs
- gRPC/WebSocket streaming repair API

### Competitive intelligence (researched Jul 2026)
Competitors: `json_repair` (Python), `jsonrepair` (JS), `repairjson` (Rust+PyO3), `safe-json-repair`, schema-guided JS tools.

Shipped vs competitors:
- [x] **Smart quote normalization**
- [x] **Prose/preamble extraction**
- [x] **Boolean variant recognition** (`yes`/`no`, `on`/`off`)
- [x] **Format-detection confidence** in public API
- [ ] **Schema-guided repair**
- [ ] **WASM bindings**
- [ ] **Python bindings (PyO3)**
- [ ] **Hosted REST API**

## Known Issues (Fixed) ✅

- [x] YAML validator too permissive
- [x] Per-format CLI subcommand duplication
- [x] Heavy parser dependency tree
- [x] Stale MCP server version string

---

See also: [ARCHITECTURE.md](ARCHITECTURE.md) · [SPEC.md](SPEC.md) · [docs/CHANGELOG.md](docs/CHANGELOG.md)
