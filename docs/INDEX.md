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
- **[MCP_IMPLEMENTATION.md](MCP_IMPLEMENTATION.md)** - MCP server implementation details
- **[STREAMING_FEATURE.md](STREAMING_FEATURE.md)** - Streaming repair for large files

### Testing
- **[TEST_SUMMARY.md](TEST_SUMMARY.md)** - Test coverage and organization (280+ tests)

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
2. Check [TEST_SUMMARY.md](TEST_SUMMARY.md) for test coverage
3. Review [CHANGELOG.md](CHANGELOG.md) for recent changes

### For Contributors
1. Read [ARCHITECTURE.md](ARCHITECTURE.md) for system design
2. Check [MCP_IMPLEMENTATION.md](MCP_IMPLEMENTATION.md) for MCP integration
3. Review [STREAMING_FEATURE.md](STREAMING_FEATURE.md) for streaming implementation

## Project Statistics

- **Formats Supported**: 8 (JSON, YAML, Markdown, XML, TOML, CSV, INI, Diff/Unified diff)
- **Test Coverage**: 318 tests (100% pass rate)
- **Binary Size**: 1.5 MB (optimized)
- **MCP Tools**: 10 available
- **Documentation Files**: 8 core files

## Key Files Location

```
anyrepair/
├── README.md              # Main project overview
├── TODO.md               # Task tracking
├── SPEC.md               # Technical specification
├── Cargo.toml            # Project configuration
├── src/                  # Source code
│   ├── lib.rs           # Library entry point
│   ├── main.rs          # CLI entry point
│   ├── bin/mcp_server.rs # MCP server
│   ├── diff.rs         # Diff repairer
│   └── ...              # Other modules
├── examples/            # Usage examples
│   ├── README.md
│   └── data/           # Test data files
├── tests/               # Test suites (280+ tests)
├── docs/                # Documentation (this folder)
│   ├── INDEX.md        # This file
│   ├── ARCHITECTURE.md # System design
│   └── ...             # Other docs
└── target/             # Build output
```

## Last Updated

March 29, 2026

## Status

✅ **PRODUCTION READY**

All features implemented, tested, and documented. Version 0.2.2 with KISS/DRY/SoC refactoring and centralized format registry.
