# Streaming Repair Feature

## Overview

The streaming repair feature enables processing of large files with minimal memory overhead. Instead of loading entire files into memory, content is processed in configurable chunks, making it suitable for files larger than available RAM.

## Implementation Details

### Core Module: `src/streaming.rs`

The `StreamingRepair` struct provides:

```rust
pub struct StreamingRepair {
    buffer_size: usize,
}

impl StreamingRepair {
    pub fn new() -> Self
    pub fn with_buffer_size(buffer_size: usize) -> Self
    pub fn process<R: BufRead, W: Write>(
        &self,
        reader: R,
        writer: &mut W,
        format: &str,
    ) -> Result<usize>
}
```

### Key Features

- **Configurable Buffer Size**: Default 8KB, adjustable for different memory constraints
- **Format Support**: All 7 formats (JSON, YAML, Markdown, XML, TOML, CSV, INI)
- **Auto-Detection**: Automatic format detection when format="auto"
- **Progress Tracking**: Returns total bytes processed
- **Stream I/O**: Works with any `BufRead` and `Write` implementations

## CLI Usage

### Basic Streaming Repair

```bash
# Stream repair a JSON file
anyrepair stream --input large_file.json --output repaired.json --format json

# Stream repair with custom buffer size (16KB)
anyrepair stream --input large_file.yaml --output repaired.yaml --format yaml --buffer-size 16384

# Stream from stdin to stdout
cat large_file.json | anyrepair stream --format json > repaired.json

# Auto-detect format from content
anyrepair stream --input data.txt --output repaired.txt
```

## Library Usage

### Basic Example

```rust
use anyrepair::StreamingRepair;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = BufReader::new(File::open("large_file.json")?);
    let mut output = File::create("repaired.json")?;
    
    let processor = StreamingRepair::new();
    let bytes_processed = processor.process(input, &mut output, "json")?;
    
    println!("Processed {} bytes", bytes_processed);
    Ok(())
}
```

### Custom Buffer Size

```rust
use anyrepair::StreamingRepair;

// Use 64KB buffer for better performance with large files
let processor = StreamingRepair::with_buffer_size(65536);
```

### Streaming from stdin/stdout

```rust
use anyrepair::StreamingRepair;
use std::io::{stdin, stdout};

let processor = StreamingRepair::new();
let bytes = processor.process(
    stdin().lock(),
    &mut stdout(),
    "json"
)?;
```

## Performance Characteristics

### Memory Usage

- **Default (8KB buffer)**: ~8KB + repair overhead per chunk
- **Large buffer (64KB)**: ~64KB + repair overhead per chunk
- **Streaming advantage**: O(buffer_size) vs O(file_size) for non-streaming

### Processing Speed

- Comparable to non-streaming repair for files that fit in memory
- Significantly better for files > available RAM
- Chunk processing allows for parallel optimization in future versions

## Test Coverage

### Unit Tests (8 tests)
- Basic JSON/YAML/Markdown repair
- Custom buffer sizes
- Large file simulation (100 items)
- Auto-detection
- Empty input handling

### Integration Tests (26 tests)
- Multiline content for all formats
- Small and large buffer sizes
- Many lines processing (1000+ lines)
- Trailing comma handling
- Mixed content types
- CSV quoted fields
- XML attributes
- Markdown code blocks
- TOML arrays
- INI comments
- Whitespace handling

### Test Results
- **Total Tests**: 34 (8 unit + 26 integration)
- **Pass Rate**: 100% (34/34)
- **Formats Covered**: 7/7 (JSON, YAML, Markdown, XML, TOML, CSV, INI)

## Advantages Over Non-Streaming

| Aspect | Non-Streaming | Streaming |
|--------|---------------|-----------|
| Memory Usage | O(file_size) | O(buffer_size) |
| Large Files | May fail/OOM | Works reliably |
| Small Files | Optimal | Slight overhead |
| Simplicity | Simple | Requires BufRead/Write |
| Parallelization | Difficult | Easier per-chunk |

## Use Cases

1. **Large LLM Outputs**: Process multi-GB API responses
2. **Log File Repair**: Fix corrupted log files without loading entirely
3. **Data Pipeline**: Stream repair as part of ETL process
4. **Memory-Constrained Environments**: Embedded systems, serverless functions
5. **Real-Time Processing**: Process data as it arrives

## Future Enhancements

- [ ] Parallel chunk processing with rayon
- [ ] Progress callbacks for monitoring
- [ ] Adaptive buffer sizing based on available memory
- [ ] Streaming validation without full repair
- [ ] Partial repair mode for incremental fixes

## Backward Compatibility

✅ **Fully backward compatible** - No breaking changes to existing API
- All existing repair functions unchanged
- New `StreamingRepair` is additive only
- Existing code continues to work without modification

## Related Files

- **Implementation**: `src/streaming.rs`
- **CLI Integration**: `src/main.rs` (Stream command)
- **Tests**: `tests/streaming_tests.rs`
- **Documentation**: `README.md` (Streaming section)
