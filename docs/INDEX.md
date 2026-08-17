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
- **[TEST_SUMMARY.md](TEST_SUMMARY.md)** - Test coverage and organization (578 tests)

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

- **Formats Supported**: 16 (10 text + 6 binary media: JSON, YAML, Markdown, XML, TOML, CSV, INI, Diff, Properties, Env, PNG, JPEG, WAV, MP3, MP4, MOV)
- **Test Coverage**: 578 tests (100% pass rate)
- **Binary Size**: 1.5 MB (optimized)
- **MCP Tools**: 12 available
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
│   ├── image.rs, audio.rs, video.rs  # Binary media repairers
│   └── ...              # Other modules
├── examples/            # Usage examples
│   ├── README.md
│   └── data/           # Test data files
├── tests/               # Test suites (578 tests)
├── docs/                # Documentation (this folder)
│   ├── INDEX.md        # This file
│   ├── ARCHITECTURE.md # System design
│   └── ...             # Other docs
└── target/             # Build output
```

## Last Updated

August 18, 2026

## Status

✅ **PRODUCTION READY**

All features implemented, tested, and documented. Version 0.2.11 with 10 text formats, 6 binary media formats, MCP server, streaming, and 578 tests.
