# Documentation Index

## Quick Start

- **[README.md](../README.md)** - Project overview, features, and usage
- **[TODO.md](../TODO.md)** - Completed tasks and roadmap
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design and module organization

## Core Documentation

### Architecture & Design
- **[../ARCHITECTURE.md](../ARCHITECTURE.md)** - System architecture, module organization, and design patterns

### Process & Workflow
- **[../AGENTS.md](../AGENTS.md)** - Agent development loop and memory system
- **[../MEMORY.md](../MEMORY.md)** - Institutional knowledge and pattern library

### Features
- **[MCP_SERVER.md](MCP_SERVER.md)** - MCP server documentation and integration guide
- **[MCP_IMPLEMENTATION.md](MCP_IMPLEMENTATION.md)** - MCP server implementation details
- **[STREAMING_FEATURE.md](STREAMING_FEATURE.md)** - Streaming repair for large files

### Testing
- **[TEST_SUMMARY.md](TEST_SUMMARY.md)** - Test coverage and organization (578 tests)

### Troubleshooting
- **[../TROUBLESHOOTING.md](../TROUBLESHOOTING.md)** - Common failures and fixes

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
1. Read [../AGENTS.md](../AGENTS.md) for the development loop and workflow
2. Read [../MEMORY.md](../MEMORY.md) for proven patterns and domain knowledge
3. Read [../ARCHITECTURE.md](../ARCHITECTURE.md) for system design
4. Check [TEST_SUMMARY.md](TEST_SUMMARY.md) for test coverage
5. Review [CHANGELOG.md](CHANGELOG.md) for recent changes

### For Contributors
1. Read [../AGENTS.md](../AGENTS.md) for the development loop
2. Read [../ARCHITECTURE.md](../ARCHITECTURE.md) for system design
3. Check [MCP_IMPLEMENTATION.md](MCP_IMPLEMENTATION.md) for MCP integration
4. Review [STREAMING_FEATURE.md](STREAMING_FEATURE.md) for streaming implementation

## Project Statistics

- **Formats Supported**: 16 (10 text + 6 binary media: JSON, YAML, Markdown, XML, TOML, CSV, INI, Diff, Properties, Env, PNG, JPEG, WAV, MP3, MP4, MOV)
- **Test Coverage**: 578 tests (100% pass rate)
- **Binary Size**: ~490 KB (release, opt-level "z", LTO, strip)
- **MCP Tools**: 12 available
- **Runtime Dependencies**: 0 (pure standard library)

## Key Files Location

```
anyrepair/
├── README.md              # Main project overview
├── AGENTS.md              # Development loop and workflow
├── MEMORY.md              # Institutional knowledge
├── TODO.md                # Task tracking
├── SPEC.md                # Technical specification
├── TROUBLESHOOTING.md     # Common failures and fixes
├── ARCHITECTURE.md        # System design
├── Cargo.toml             # Project configuration
├── src/                   # Source code
│   ├── lib.rs            # Library entry point
│   ├── main.rs           # CLI entry point
│   ├── bin/mcp_server.rs # MCP server
│   ├── diff.rs          # Diff repairer
│   ├── pattern.rs       # Shared pattern-matching helpers
│   ├── image.rs, audio.rs, video.rs  # Binary media repairers
│   └── ...               # Other modules
├── examples/             # Usage examples
│   ├── README.md
│   └── data/            # Test data files
├── tests/                # Test suites (578 tests)
├── docs/                 # Documentation (this folder)
│   ├── INDEX.md         # This file
│   ├── ARCHITECTURE.md  # Redirect to root ARCHITECTURE.md
│   └── ...              # Other docs
└── target/              # Build output
```

## Last Updated

September 5, 2026

## Status

✅ **PRODUCTION READY**

All features implemented, tested, and documented. Version 0.2.11 with 10 text formats, 6 binary media formats, MCP server, streaming, and 578 tests.
