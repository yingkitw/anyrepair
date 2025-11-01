# Project Folder Structure

## Overview

The AnyRepair project is organized for clarity, maintainability, and ease of navigation.

## Root Directory

```
anyrepair/
├── .gitattributes          # Git attributes configuration
├── .gitignore             # Git ignore rules
├── Cargo.toml             # Rust project manifest
├── Cargo.lock             # Dependency lock file
├── README.md              # Main project documentation
├── TODO.md                # Task tracking and roadmap
├── FOLDER_STRUCTURE.md    # This file
├── anyrepair.png          # Project logo
├── anyrepair.toml         # Configuration file
├── plugin_config.toml     # Plugin configuration
├── benches/               # Benchmarks
├── docs/                  # Documentation (22 files)
├── examples/              # Usage examples
├── src/                   # Source code
├── tests/                 # Test suites
└── target/                # Build output (gitignored)
```

## Source Code Structure (`src/`)

```
src/
├── lib.rs                 # Library entry point
├── main.rs                # CLI entry point (180 lines)
├── bin/
│   └── mcp_server.rs     # MCP server binary
├── cli/                   # CLI module (modulized)
│   ├── mod.rs            # CLI utilities
│   ├── repair_cmd.rs     # Repair commands
│   ├── validate_cmd.rs   # Validation commands
│   ├── batch_cmd.rs      # Batch processing
│   ├── rules_cmd.rs      # Rules management
│   └── stream_cmd.rs     # Streaming commands
├── json/                  # JSON module (modulized, 573 lines)
│   ├── mod.rs            # Main repairer (216 lines)
│   ├── strategies.rs     # Repair strategies (312 lines)
│   └── validator.rs      # JSON validator (45 lines)
├── markdown/              # Markdown module (modulized, 554 lines)
│   ├── mod.rs            # Main repairer (186 lines)
│   ├── strategies.rs     # Repair strategies (301 lines)
│   └── validator.rs      # Markdown validator (67 lines)
├── mcp_server.rs         # MCP server implementation (312 lines)
├── streaming.rs          # Streaming repair support
├── error.rs              # Error types
├── traits.rs             # Core trait definitions
├── repairer_base.rs      # Base repairer
├── yaml.rs               # YAML repairer
├── xml.rs                # XML repairer
├── csv.rs                # CSV repairer
├── toml.rs               # TOML repairer
├── ini.rs                # INI repairer
├── plugin.rs             # Plugin system
├── plugin_config.rs      # Plugin configuration
├── plugin_integration.rs # Plugin integration
├── config.rs             # Configuration management
├── custom_rules.rs       # Custom repair rules
├── parallel.rs           # Parallel processing
├── parallel_strategy.rs  # Strategy application
├── advanced.rs           # Advanced features
├── context_parser.rs     # Context parsing
├── enhanced_json.rs      # Enhanced JSON repair
├── analytics.rs          # Analytics tracking
├── batch_processor.rs    # Batch processing
├── validation_rules.rs   # Validation rules
└── audit_log.rs          # Audit logging
```

## Documentation Structure (`docs/`)

### Core Documentation (22 files)
- `INDEX.md` - Documentation index (this folder)
- `ARCHITECTURE.md` - System design
- `FINAL_SUMMARY.md` - Project summary

### Feature Documentation
- `MCP_SERVER.md` - MCP server guide
- `MCP_IMPLEMENTATION.md` - Implementation details
- `MCP_TEST_COVERAGE.md` - Test coverage
- `STREAMING_FEATURE.md` - Streaming support
- `PLUGIN_DEVELOPMENT.md` - Plugin guide
- `enterprise_features.md` - Enterprise features

### Modulization Documentation
- `MODULIZATION_GUIDE.md` - Principles and patterns
- `MODULIZATION_SUMMARY.md` - Work summary
- `MODULIZATION_COMPLETE.md` - Completion status

### Testing Documentation
- `COMPLEX_DAMAGE_TESTS.md` - Damage scenarios
- `COMPLEX_STREAMING_TESTS.md` - Streaming scenarios
- `TEST_SUMMARY.md` - Test overview

### Build & Deployment
- `BUILD_OPTIMIZATION.md` - Optimization guide
- `PUBLICATION_CHECKLIST.md` - Publication checklist

### Additional
- `CHANGELOG.md` - Version history
- `EXAMPLES_SUMMARY.md` - Examples overview
- `IMPLEMENTATION_SUMMARY.md` - Implementation summary
- `DRY_KISS_IMPROVEMENTS.md` - Code quality

## Examples Structure (`examples/`)

```
examples/
├── README.md                # Examples guide
├── mcp_repair_json.rs      # JSON repair example
├── mcp_multi_format.rs     # Multi-format example
├── mcp_server_usage.rs     # Usage patterns
└── mcp_protocol.md         # Protocol reference
```

## Tests Structure (`tests/`)

```
tests/
├── complex_damage_tests.rs      # Complex damage scenarios (18 tests)
├── complex_streaming_tests.rs   # Complex streaming (18 tests)
├── damage_scenarios.rs          # Damage scenarios (18 tests)
├── fuzz_tests.rs               # Fuzz testing (36 tests)
├── integration_tests.rs        # Integration tests (4 tests)
└── streaming_tests.rs          # Streaming tests (26 tests)
```

## Configuration Files

- `Cargo.toml` - Project manifest with dependencies
- `Cargo.lock` - Locked dependency versions
- `anyrepair.toml` - Application configuration
- `plugin_config.toml` - Plugin configuration
- `.gitignore` - Git ignore rules
- `.gitattributes` - Git attributes

## Build Output (`target/`)

```
target/
├── debug/           # Debug builds
├── release/         # Release builds (1.5 MB binaries)
├── dist/            # Distribution builds
└── doc/             # Generated documentation
```

## Key Statistics

### Code Organization
- **Total Source Files**: 30+
- **Total Lines of Code**: ~1662 (modulized)
- **Code Reduction**: 57% through modulization
- **Modules**: 3 major (JSON, Markdown, CLI)

### Documentation
- **Total Documentation Files**: 22
- **Total Examples**: 3 working examples
- **Documentation Lines**: ~5000+

### Tests
- **Total Test Files**: 6
- **Total Tests**: 311
- **Pass Rate**: 100%

### Binary Sizes
- **Debug**: 25 MB
- **Release**: 1.5 MB (94% reduction)
- **Distribution**: 1.5 MB

## File Organization Principles

### 1. Root Level
- Only essential files (README, TODO, Cargo.toml)
- Configuration files
- Hidden files (.git*, .ignore)

### 2. Source Code (`src/`)
- Organized by module
- Modulized components in subdirectories
- Clear separation of concerns

### 3. Documentation (`docs/`)
- All documentation centralized
- Organized by category
- INDEX.md for navigation

### 4. Examples (`examples/`)
- Standalone, runnable examples
- Comprehensive documentation
- Multiple scenarios covered

### 5. Tests (`tests/`)
- Integration tests
- Complex scenarios
- Fuzz testing

## Navigation Guide

### For Users
1. Start: `README.md`
2. Examples: `examples/README.md`
3. Features: `docs/MCP_SERVER.md`

### For Developers
1. Architecture: `docs/ARCHITECTURE.md`
2. Modulization: `docs/MODULIZATION_GUIDE.md`
3. Source: `src/` directory

### For Deployment
1. Build: `docs/BUILD_OPTIMIZATION.md`
2. Publish: `docs/PUBLICATION_CHECKLIST.md`
3. Status: `docs/FINAL_SUMMARY.md`

## Cleanup Summary

### Moved to `docs/` (22 files)
- Architecture documentation
- Feature documentation
- Modulization guides
- Test documentation
- Build optimization
- Implementation summaries

### Kept in Root (9 files)
- `.gitattributes` - Version control
- `.gitignore` - Git configuration
- `Cargo.toml` - Project manifest
- `Cargo.lock` - Dependencies
- `README.md` - Main documentation
- `TODO.md` - Task tracking
- Configuration files (3)
- Project logo

### Result
- ✅ Clean root directory
- ✅ Organized documentation
- ✅ Easy navigation
- ✅ Professional structure

## Last Updated

November 1, 2025

## Status

✅ **ORGANIZED & CLEAN**

All files properly organized for production use.
