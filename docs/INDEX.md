# Documentation Index

## Quick Start

- **[README.md](../README.md)** - Project overview, features, and usage
- **[TODO.md](../TODO.md)** - Completed tasks and roadmap
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design and module organization

## Core Documentation

### Architecture & Design
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture, module organization, and design patterns

### Features
- **[MCP_SERVER.md](MCP_SERVER.md)** - MCP server documentation and integration guide
- **[STREAMING_FEATURE.md](STREAMING_FEATURE.md)** - Streaming repair for large files
- **[PLUGIN_DEVELOPMENT.md](PLUGIN_DEVELOPMENT.md)** - Plugin system development guide
- **[enterprise_features.md](enterprise_features.md)** - Enterprise features overview

### Development Guides
- **[MODULIZATION_GUIDE.md](MODULIZATION_GUIDE.md)** - Code organization principles and patterns
- **[BUILD_OPTIMIZATION.md](BUILD_OPTIMIZATION.md)** - Build optimization and binary size reduction

### Testing
- **[TEST_SUMMARY.md](TEST_SUMMARY.md)** - Test coverage and organization (364+ tests)

### Version History
- **[CHANGELOG.md](CHANGELOG.md)** - Version history and release notes

## Examples

- **[examples/README.md](../examples/README.md)** - Usage examples and code samples
- **[examples/data/README.md](../examples/data/README.md)** - Test data files documentation

## Quick Navigation

### For Users
1. Start with [README.md](../README.md)
2. Check [examples/README.md](../examples/README.md) for usage examples
3. Review [MCP_SERVER.md](MCP_SERVER.md) for MCP integration

### For Developers
1. Read [ARCHITECTURE.md](ARCHITECTURE.md) for system design
2. Review [MODULIZATION_GUIDE.md](MODULIZATION_GUIDE.md) for code organization
3. Check [TEST_SUMMARY.md](TEST_SUMMARY.md) for test coverage

### For Contributors
1. Review [PLUGIN_DEVELOPMENT.md](PLUGIN_DEVELOPMENT.md) for extending functionality
2. Check [BUILD_OPTIMIZATION.md](BUILD_OPTIMIZATION.md) for build guidelines
3. See [CHANGELOG.md](CHANGELOG.md) for recent changes

## Project Statistics

- **Formats Supported**: 8 (JSON, YAML, Markdown, XML, TOML, CSV, INI, Diff/Unified diff)
- **Test Coverage**: 364+ tests (100% pass rate)
- **Binary Size**: 1.5 MB (optimized)
- **MCP Tools**: 10 available
- **Documentation Files**: 10 core files (consolidated from 22+)

## Key Files Location

```
anyrepair/
├── README.md              # Main project overview
├── TODO.md               # Task tracking
├── Cargo.toml            # Project configuration
├── src/                  # Source code
│   ├── lib.rs           # Library entry point
│   ├── main.rs          # CLI entry point
│   ├── mcp_server.rs   # MCP server
│   ├── diff.rs         # Diff repairer
│   └── ...              # Other modules
├── examples/            # Usage examples
│   ├── README.md
│   └── data/           # Test data files
├── tests/               # Test suites (7 files, 364+ tests)
├── docs/                # Documentation (this folder)
│   ├── INDEX.md        # This file
│   ├── ARCHITECTURE.md # System design
│   └── ...             # Other docs
└── target/             # Build output
```

## Archived Documentation

Historical and redundant documentation has been moved to `docs/archive/`:
- Modulization progress files
- Implementation summaries
- Test detail files
- Optimization summaries

## Last Updated

November 24, 2024

## Status

✅ **PRODUCTION READY**

All features implemented, tested, and documented. Documentation consolidated for better maintainability.
