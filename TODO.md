# TODO

## Recently Completed ✅ (v0.2.x)

### Core Improvements
- [x] KISS/DRY/SoC refactoring - Centralized format registry, unified CLI
- [x] Removed test dependency on `insta` - Cleaned up test code
- [x] Removed all compilation warnings - Zero warning build
- [x] Updated dev-dependencies (tempfile 3.25, arbitrary 1.4)
- [x] All 8 format support completed (JSON, YAML, Markdown, XML, TOML, CSV, INI, Diff)
- [x] MCP server implementation with 10 tools
- [x] Python-compatible API (`jsonrepair()` function and `JsonRepair` struct)
- [x] Streaming support for large files
- [x] Custom repair rule configuration
- [x] Fuzz testing with proptest (36 tests)
- [x] Build size optimization (94% reduction to 1.5 MB)
- [x] 280+ test cases with 100% pass rate

## Current Implementation Priorities 🚀

### High Priority
- [ ] **Code coverage improvement** - Add more edge case tests
- [ ] **Performance regression tests** - Catch performance degradation in CI
- [ ] **Real-world corpus testing** - Test with actual failures from users

### Medium Priority
- [ ] **Web interface** - Simple web UI for online repair
- [ ] **REST API** - HTTP API for programmatic access
- [ ] **Docker container** - Easy deployment

## Planned Features 📋

### Additional Format Support
- [ ] **Properties files** (.properties) - Java properties format
- [ ] **Env files** (.env) - Environment variables format
- [ ] **Protobuf** - Protocol Buffers format

### CLI Enhancements
- [ ] **Diff preview** - `--diff` flag to show changes before applying
- [ ] **Dry-run mode** - `--dry-run` to preview changes without modifying
- [ ] **Colored output** - Syntax-highlighted output with ANSI colors
- [ ] **Progress bars** - Visual progress for batch operations
- [ ] **JSON output** - Machine-readable output for CI/CD
- [ ] **Exit codes** - Standard exit codes (0=success, 1=repaired, 2=error)
- [ ] **Config file search** - Support `.anyrepair.toml` in project dirs
- [ ] **Shell completions** - Bash/zsh/fish completions
- [ ] **Man pages** - Unix manual pages

### Repair Quality Improvements
- [ ] **Format-preserving repairs** - Maintain whitespace, comments, ordering
- [ ] **Semantic repairs** - Understand meaning, not just syntax
- [ ] **Repair explanation** - Explain what was repaired and why
- [ ] **Confidence thresholds** - Configurable confidence cutoffs
- [ ] **Multi-pass repairs** - Iterative repair for complex issues
- [ ] **Style preservation** - Maintain coding style (quotes, indentation)
- [ ] **Comment preservation** - Keep original comments intact
- [ ] **Minimally invasive repairs** - Change only what's necessary

### Performance Optimizations
- [ ] **Lazy strategy evaluation** - Skip strategies if not needed
- [ ] **Compile-time regex optimization** - Use compile-time regex optimization
- [ ] **SIMD operations** - Use SIMD for string processing
- [ ] **Memory-mapped files** - Use memmap for large files
- [ ] **Incremental repair** - Repair only modified portions
- [ ] **Strategy caching** - Cache successful strategy patterns

### Testing Improvements
- [ ] **Mutation testing** - Use cargo-mutants to test error handling
- [ ] **Property-based testing** - Expand proptest coverage
- [ ] **Cross-format tests** - Test content mixing multiple formats
- [ ] **Fuzzing integration** - Continuous fuzzing in CI
- [ ] **Golden master tests** - Compare against known-good repairs
- [ ] **Locale-specific tests** - Test Unicode and locale-specific content

### Documentation Enhancements
- [ ] **Rustdoc completeness** - Document all public APIs
- [ ] **Code examples** - More runnable examples in docs
- [ ] **Video tutorials** - Screen recordings for workflows
- [ ] **Troubleshooting guide** - Common issues and solutions
- [ ] **Migration guides** - Version upgrade guides
- [ ] **Contribution guidelines** - Detailed contribution instructions
- [ ] **API reference** - Complete API reference with all types
- [ ] **Performance guide** - Tuning and optimization guide
- [ ] **Security considerations** - Security best practices

### Error Handling & Diagnostics
- [ ] **Structured error codes** - Machine-readable error codes
- [ ] **Error suggestions** - Suggest fixes for common errors
- [ ] **Diagnostic context** - Show surrounding content on errors
- [ ] **Recovery hints** - Suggest recovery strategies
- [ ] **Error formatting** - Pretty-printed error messages
- [ ] **Error location tracking** - Track exact line/column of issues

### Security & Safety
- [ ] **Input sanitization** - Prevent code injection vulnerabilities
- [ ] **Resource limits** - Configurable memory/CPU limits
- [ ] **Rate limiting** - Prevent abuse in API/server mode
- [ ] **PII detection** - Detect and redact sensitive data

### Internationalization
- [ ] **Full Unicode support** - Complete Unicode content handling
- [ ] **Localized error messages** - Error messages in multiple languages
- [ ] **Locale-specific formats** - Handle locale-specific number/date formats
- [ ] **RTL text support** - Right-to-left text handling

## Technical Debt & Maintenance 🔧

### Code Quality
- [ ] **Clippy lints** - Enforce all clippy suggestions
- [ ] **Rustfmt strict** - Strict formatting compliance
- [ ] **Documentation coverage** - Enforce doc coverage percentage
- [ ] **Complexity metrics** - Monitor cyclomatic complexity
- [ ] **Code coverage** - Enforce test coverage thresholds
- [ ] **Dead code elimination** - Regular unused code cleanup
- [ ] **Dependency review** - Regular dependency audit
- [ ] **Security audit** - Regular security audits

## Ideas for Future 💡

### Advanced Features
- [ ] **Web interface for online repair** - Browser-based repair tool
- [ ] **REST API for programmatic access** - HTTP API
- [ ] **Docker container for easy deployment** - Containerized deployment
- [ ] **Repair history and undo functionality** - Track and revert repairs
- [ ] **Custom repair templates** - Pre-built repair templates
- [ ] **Repair quality feedback system** - User feedback on repairs
- [ ] **WebSocket streaming** - Real-time repair over WebSocket
- [ ] **Server-Sent Events** - SSE for repair progress updates
- [ ] **gRPC streaming** - Bidirectional streaming API
- [ ] **Live preview** - Show repair results as you type
- [ ] **Language bindings** - Python, Node.js, Go native libraries

### Format Detection Enhancements
- [ ] **Format detection confidence scoring** - Expose detection confidence
- [ ] **Manual format hints** - Allow format hints for better detection
- [ ] **Multi-format content handling** - Handle files with multiple formats
- [ ] **Content preview analysis** - Scan larger context for accuracy

## Known Issues (Fixed) ✅

- [x] YAML validator too permissive - Fixed with custom validation
- [x] Complex JSON structures not fully repaired - Improved with advanced strategies
- [x] Markdown repair not aggressive enough - Enhanced with better strategies
- [x] Confidence scoring not sophisticated - Improved with format-specific scoring
- [x] Unused imports and compilation warnings - All removed
- [x] insta dependency cluttering tests - Removed snapshot tests

---

For more details, see:
- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design
- **[SPEC.md](SPEC.md)** - Technical specification
- **[CHANGELOG.md](docs/CHANGELOG.md)** - Version history
