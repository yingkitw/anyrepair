# TODO

## Completed ✅

- [x] Initialize Rust project with Cargo.toml and proper structure
- [x] Create core repair traits and modules for JSON, YAML, Markdown
- [x] Implement JSON repair functionality
- [x] Implement YAML repair functionality
- [x] Implement Markdown repair functionality
- [x] Add comprehensive test cases with insta snapshots (60+ test cases)
- [x] Create CLI interface using clap
- [x] Update documentation (README.md, ARCHITECTURE.md, TODO.md)
- [x] Add comprehensive JSON damage scenarios and edge cases
- [x] Enhance CLI interface with progress indicators and better error messages
- [x] Add batch processing command for multiple files
- [x] Add statistics command for repair quality analysis
- [x] Add advanced JSON repair strategies (malformed numbers, boolean/null values)
- [x] Add comprehensive .gitignore and .gitattributes for proper version control
- [x] Performance optimization and benchmarking (regex caching, 99.6% improvement)
- [x] XML repair support implementation with comprehensive strategies
- [x] Technical debt cleanup and code optimization
- [x] Publication to crates.io (v0.1.0) - Successfully published!
- [x] TOML repair support implementation
- [x] CSV repair support implementation
- [x] INI file repair support implementation
- [x] Diff/Unified diff repair support implementation
- [x] Advanced repair strategies with enhanced capabilities
- [x] Codebase simplification - Removed redundant directories (repairers/, utils/)
- [x] Codebase simplification - Consolidated JSON and Markdown subdirectories into single files
- [x] Python jsonrepair compatible API - Added `jsonrepair()` function and `JsonRepair` struct
- [x] Comprehensive test coverage - Added 14 test cases for Python-compatible API (326 total tests)

## In Progress 🔄

- [x] Custom repair rule configuration
- [x] Fuzz testing for robustness
- [x] Additional documentation improvements
- [x] Streaming repair for large files
- [x] Complex damage test cases (18 tests)
- [x] Complex streaming test cases (18 tests)
- [x] Code modulization (JSON, Markdown, CLI modules)
- [x] MCP server implementation
- [x] MCP test coverage (43 tests)
- [x] MCP examples and documentation
- [x] Build size optimization (94% reduction)

## Next Implementation Priorities 🚀

### High Priority
- [x] **Custom repair rule configuration** - Allow users to define custom repair rules ✅
- [x] **Fuzz testing** - Add comprehensive fuzz testing for robustness ✅
### Medium Priority  
- [ ] **Web interface** - Create a simple web interface for online repair
- [ ] **REST API** - Add REST API for programmatic access
- [ ] **Docker container** - Create Docker image for easy deployment

### Low Priority
- [ ] **Video tutorials** - Create video content for better user onboarding

## Planned 📋

### Short Term (Next Release)
- [x] Add more comprehensive JSON repair strategies
  - [x] Fix nested object/array issues
  - [x] Handle malformed numbers
  - [x] Fix boolean/null value issues
- [x] Improve YAML repair accuracy
  - [x] Better indentation detection
  - [x] Handle complex nested structures
  - [x] Fix multi-line string formatting
- [x] Enhanced Markdown repair
  - [x] Fix table formatting
  - [x] Handle nested lists
  - [x] Fix image syntax
- [x] CLI improvements
  - [x] Add progress indicators
  - [x] Better error messages
  - [x] Configuration file support
  - [x] Batch processing
  - [x] Statistics command
- [x] Documentation
  - [x] Add more examples
  - [x] API documentation improvements
  - [x] Performance guide

### Medium Term
- [x] Additional format support
  - [x] XML repair
  - [x] TOML repair
  - [x] CSV repair
  - [x] INI file repair
  - [x] Diff/Unified diff repair
- [x] Advanced features
  - [x] Custom repair rule configuration
  - [x] Repair quality scoring
  - [x] Batch processing
- [x] Performance improvements
  - [x] Memory optimization
  - [x] Caching mechanisms

## Technical Debt

- [x] Remove unused imports and dependencies
- [x] Improve error messages with more context
- [x] Add more comprehensive input validation
- [x] Optimize regex patterns for better performance
- [x] Add more edge case handling
- [x] Improve confidence scoring algorithms
- [x] Consolidate duplicate code in repairers
- [x] Fix invalid Cargo.toml edition (2024 → 2021)
- [x] Optimize format detection functions
- [x] KISS/DRY/SoC refactoring (v0.2.0)
  - [x] Centralized format registry: `create_repairer()`, `create_validator()`, `normalize_format()`, `SUPPORTED_FORMATS`
  - [x] Eliminated 8 duplicated CLI `handle_*` functions → single `handle_repair` with registry
  - [x] Removed 8 per-format CLI subcommands → unified `repair --format <fmt>`
  - [x] Extracted `format_detection.rs` module (SoC: detection logic out of lib.rs)
  - [x] Removed dead code: `BaseRepairer` trait, standalone `apply_strategies()`
  - [x] DRY'd `JsonRepairer::with_logging` (reuses `new()` instead of duplicating strategies)
  - [x] Refactored `streaming.rs` and `validate_cmd.rs` to use centralized registry
  - [x] Improved diff detection precision (no false positives on YAML `---`)

## Testing

- [x] Add more edge case tests
- [x] Performance benchmarks
- [x] Memory usage tests
- [x] Fuzz testing for robustness
- [x] Integration tests with real-world data
- [x] Python jsonrepair API test coverage (14 comprehensive tests)
- [x] Total test coverage: 326 tests (204 library + 4 integration + 26 streaming + 18 complex damage + 18 complex streaming + 36 fuzz + 18 damage scenarios + 2 doctests)

## Documentation

- [ ] Add more code examples
- [ ] Create video tutorials
- [ ] Add troubleshooting guide
- [ ] Create migration guide for future versions
- [ ] Add contribution guidelines

## Known Issues

- [x] YAML validator is too permissive (serde_yaml limitation) - Fixed with custom validation
- [x] Some complex JSON structures may not be fully repaired - Improved with advanced strategies
- [x] Markdown repair could be more aggressive for better formatting - Enhanced with better strategies
- [x] Confidence scoring could be more sophisticated - Improved with format-specific scoring

## Ideas for Future

- [ ] Web interface for online repair
- [ ] REST API for programmatic access
- [ ] Docker container for easy deployment
- [ ] Repair history and undo functionality
- [ ] Custom repair templates
- [ ] Repair quality feedback system

---

## New Areas for Improvement 🎯

### Format Detection Enhancements
- [ ] **Format detection confidence scoring** - Expose detection confidence to users
- [ ] **Manual format hints** - Allow users to provide format hints for better detection
- [ ] **Multi-format content handling** - Handle files containing multiple formats (e.g., JSON in Markdown code blocks)
- [ ] **Content preview analysis** - Scan larger context for more accurate detection

### Additional Format Support
- [ ] **Properties files** (.properties) - Java properties format repair
- [ ] **Env files** (.env) - Environment variable format repair
- [ ] **Protobuf** - Protocol Buffers format repair

### CLI Improvements
- [ ] **Diff preview** - Show diff before applying repairs (`--diff` flag)
- [ ] **Dry-run mode** - Preview changes without modifying files (`--dry-run`)
- [ ] **Colored output** - Syntax-highlighted output with ANSI colors
- [ ] **Progress bars** - Visual progress for batch operations
- [ ] **JSON output** - Machine-readable JSON output for CI/CD integration
- [ ] **Exit codes** - Standard exit codes for scripting (0=success, 1=repair needed, 2=error)
- [ ] **Config file search** - Support `.anyrepair.toml` in project directories
- [ ] **Shell completions** - Generate completion scripts for bash/zsh/fish
- [ ] **Man pages** - Generate Unix manual pages

### Performance Optimizations
- [ ] **Lazy strategy evaluation** - Skip strategies if not needed for content
- [ ] **Compile-time regex optimization** - Use `regex_lite!` or compile-time optimization
- [ ] **SIMD operations** - Use SIMD for string processing operations
- [ ] **Memory-mapped files** - Use memmap for large file processing
- [ ] **Incremental repair** - Repair only modified portions in watch mode
- [ ] **Strategy caching** - Cache successful strategy patterns per format
- [ ] **Arena allocation** - Use bumpalo for reduced allocations

### Testing Improvements
- [ ] **Mutation testing** - Use cargo-mutants to test error handling
- [ ] **Property-based testing** - Expand proptest coverage for all formats
- [ ] **Performance regression tests** - Catch performance degradation in CI
- [ ] **Cross-format tests** - Test content mixing multiple formats
- [ ] **Real-world corpus** - Test with actual real-world failures from users
- [ ] **Fuzzing integration** - Continuous fuzzing in CI pipeline
- [ ] **Golden master tests** - Compare against known-good repairs
- [ ] **Locale-specific tests** - Test Unicode and locale-specific content

### Documentation Enhancements
- [ ] **Rustdoc completeness** - Add JSDoc-style docs to all public APIs
- [ ] **Code examples** - More runnable examples in documentation
- [ ] **Video tutorials** - Screen recordings for common workflows
- [ ] **Troubleshooting guide** - Common issues and solutions
- [ ] **Migration guides** - Version upgrade guides
- [ ] **Contribution guidelines** - Detailed contribution instructions
- [ ] **Architecture decision records** - ADRs for major decisions
- [ ] **API reference** - Complete API reference with all types
- [ ] **Performance guide** - Tuning and optimization guide
- [ ] **Security considerations** - Security best practices document

### Repair Quality Improvements
- [ ] **Format-preserving repairs** - Maintain whitespace, comments, ordering
- [ ] **Semantic repairs** - Understand meaning, not just syntax
- [ ] **Context-aware repairs** - Use surrounding content for better decisions
- [ ] **Repair explanation** - Explain what was repaired and why
- [ ] **Confidence thresholds** - Configurable confidence cutoffs
- [ ] **Multi-pass repairs** - Iterative repair for complex issues
- [ ] **Style preservation** - Maintain coding style (quotes, indentation)
- [ ] **Comment preservation** - Keep original comments intact
- [ ] **Minimally invasive repairs** - Change only what's necessary

### Security & Safety
- [ ] **Input sanitization** - Prevent code injection vulnerabilities
- [ ] **Resource limits** - Configurable memory/CPU limits
- [ ] **Rate limiting** - Prevent abuse in API/server mode
- [ ] **PII detection** - Detect and redact sensitive data

### Streaming & Real-time
- [ ] **WebSocket streaming** - Real-time repair over WebSocket
- [ ] **Server-Sent Events** - SSE for repair progress updates
- [ ] **gRPC streaming** - Bidirectional streaming API
- [ ] **Live preview** - Show repair results as you type
- [ ] **Incremental validation** - Validate before full repair

### Error Handling & Diagnostics
- [ ] **Structured error codes** - Machine-readable error codes
- [ ] **Error suggestions** - Suggest fixes for common errors
- [ ] **Diagnostic context** - Show surrounding content on errors
- [ ] **Recovery hints** - Suggest recovery strategies
- [ ] **Error formatting** - Pretty-printed error messages
- [ ] **Error location tracking** - Track exact line/column of issues

### Internationalization
- [ ] **Unicode support** - Full Unicode content handling
- [ ] **Localized error messages** - Error messages in multiple languages
- [ ] **Locale-specific formats** - Handle locale-specific number/date formats
- [ ] **RTL text support** - Right-to-left text handling

### Code Quality
- [ ] **Clippy lints** - Enforce all clippy suggestions
- [ ] **Rustfmt strict** - Strict formatting compliance
- [ ] **Documentation coverage** - Enforce doc coverage percentage
- [ ] **Complexity metrics** - Monitor cyclomatic complexity
- [ ] **Code coverage** - Enforce test coverage thresholds
- [ ] **Dead code elimination** - Regular unused code cleanup
- [ ] **Dependency review** - Regular dependency audit
- [ ] **Security audit** - Regular security audits
