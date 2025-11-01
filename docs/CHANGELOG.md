# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.5] - 2025-10-26

### Added

#### Enterprise Features
- **Advanced Analytics Module** (`analytics.rs`)
  - Repair success metrics tracking
  - Performance monitoring with detailed timing
  - Format-specific metrics
  - Success rate calculation
  - Thread-safe operation with Arc<Mutex<>>

- **Batch Processing Module** (`batch_processor.rs`)
  - Multi-format batch file processing
  - Directory processing (recursive and single-level)
  - File filtering by extension
  - Automatic format detection per file
  - Detailed per-file results with timing
  - Integrated analytics tracking

- **Validation Rules Module** (`validation_rules.rs`)
  - Custom validation rules engine
  - Multiple rule types (Regex, Length, Format, Custom)
  - Rule management (add, remove, enable/disable)
  - Flexible validation against multiple rules
  - Detailed violation reporting

- **Audit Logging Module** (`audit_log.rs`)
  - Comprehensive audit logging for compliance
  - Event tracking (repairs, validations, batch operations, config changes)
  - Detailed entries with RFC3339 timestamps
  - Optional file persistence for compliance
  - Query capabilities (filter by type or actor)
  - JSON format for easy parsing

#### Confidence Scoring
- **Advanced Confidence Scorer** (`confidence_scorer.rs`)
  - Improved confidence scoring algorithms for all formats
  - Component-based scoring (structure, completeness, syntax, format)
  - Weighted average calculation
  - Format-specific scoring optimizations
  - JSON: Structure validation, completeness check, syntax validation
  - YAML: Indentation analysis, key-value detection, document structure
  - XML: Tag structure validation, proper nesting checks
  - Markdown: Header patterns, code blocks, link syntax
  - CSV: Delimiter consistency, column count validation
  - TOML: Section headers, key-value pairs, proper structure

#### Documentation
- Comprehensive enterprise features documentation (`docs/enterprise_features.md`)
- Implementation summary with detailed feature descriptions (`IMPLEMENTATION_SUMMARY.md`)
- Enhanced README with enterprise features and usage examples
- Updated ARCHITECTURE.md with enterprise modules
- CHANGELOG.md for version history

#### Metadata
- Updated Cargo.toml with crates.io publication metadata
- Added homepage and documentation links
- Enhanced description and keywords
- Version bump to 0.1.5

### Changed
- Improved confidence scoring across all repairers
- Enhanced README with enterprise features section
- Updated Cargo.toml metadata for better discoverability
- Improved JSON repair with better quote escaping
- Enhanced JSONL format support

### Fixed
- Fixed operator precedence in trailing content detection
- Improved handling of stray backticks in JSON
- Better support for JSONL (JSON Lines) format
- Fixed idempotency issues in repair operations

### Technical Improvements
- Added chrono dependency for RFC3339 timestamps
- Improved thread safety with Arc<Mutex<>> patterns
- Better error handling across all modules
- Optimized performance with caching strategies

## [0.1.4] - 2025-10-25

### Added
- Improved JSON repair with unescaped quote handling
- Better trailing content detection
- Support for JSONL format

### Fixed
- Fixed stray backticks in JSON files
- Improved quote escaping in string values
- Better handling of special characters

## [0.1.3] - 2025-10-24

### Added
- Plugin system with external strategy loading
- Custom repair rules configuration
- Plugin discovery and management
- Extended repair configuration

### Fixed
- Improved plugin integration
- Better error handling in plugin loading

## [0.1.2] - 2025-10-23

### Added
- Parallel strategy application
- Performance monitoring
- Memory optimization with caching
- Advanced repair strategies

### Fixed
- Regex caching for better performance
- Improved strategy ordering

## [0.1.1] - 2025-10-22

### Added
- Multi-format support (JSON, YAML, XML, TOML, CSV, INI, Markdown)
- Format auto-detection
- CLI tool with multiple subcommands
- Configuration management
- Custom repair rules

### Fixed
- Initial bug fixes and stability improvements

## [0.1.0] - 2025-10-21

### Added
- Initial release
- JSON repair functionality
- YAML repair functionality
- Markdown repair functionality
- Basic CLI interface
- Comprehensive test suite

[0.1.5]: https://github.com/yingkitw/anyrepair/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/yingkitw/anyrepair/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/yingkitw/anyrepair/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/yingkitw/anyrepair/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/yingkitw/anyrepair/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/yingkitw/anyrepair/releases/tag/v0.1.0
