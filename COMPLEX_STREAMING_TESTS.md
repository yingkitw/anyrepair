# Complex Streaming Test Cases

## Overview

The `tests/complex_streaming_tests.rs` file contains 18 comprehensive test cases for streaming repair with complex damage patterns and large file scenarios.

## Test Coverage

### Large File Scenarios (7 tests)

1. **Complex JSON Large Nested Structure**
   - Size: 50 nested objects with multiple damage types
   - Damage: Trailing commas, mixed indentation
   - Buffer: 2KB
   - Validates: Streaming through large nested hierarchies

2. **Complex YAML Large Config with Errors**
   - Size: 30 services with endpoints and configs
   - Damage: Indentation inconsistencies
   - Buffer: 4KB
   - Validates: Large YAML structure streaming

3. **Complex Markdown Large Document**
   - Size: 20 sections with code blocks and lists
   - Damage: Formatting inconsistencies
   - Buffer: 3KB
   - Validates: Large markdown document streaming

4. **Complex CSV Large Dataset**
   - Size: 100 rows with quoted fields
   - Damage: Special characters, escaping
   - Buffer: 2KB
   - Validates: Large CSV streaming

5. **Complex XML Large Nested**
   - Size: 25 items with 3 levels of nesting
   - Damage: Unclosed tags, malformed attributes
   - Buffer: 2KB
   - Validates: Large XML structure streaming

6. **Complex TOML Large Config**
   - Size: 20 sections with arrays
   - Damage: Missing quotes, malformed arrays
   - Buffer: 2KB
   - Validates: Large TOML config streaming

7. **Complex INI Large Config**
   - Size: 30 sections with key-value pairs
   - Damage: Malformed sections
   - Buffer: 2KB
   - Validates: Large INI config streaming

### Buffer Size Testing (2 tests)

1. **Very Small Buffer with Complex JSON**
   - Size: 100 JSON objects
   - Buffer: 256 bytes (forces many iterations)
   - Validates: Streaming with minimal buffer

2. **Large Buffer with Complex YAML**
   - Size: 50 YAML items
   - Buffer: 64KB
   - Validates: Streaming with large buffer

### Complex Damage Patterns (3 tests)

1. **Mixed Damage JSON Large**
   - Size: 40 user objects
   - Damage: Single quotes, trailing commas, nested errors
   - Buffer: 3KB
   - Validates: Multiple damage types in streaming

2. **Performance Many Small Chunks**
   - Size: 500 lines
   - Buffer: 512 bytes
   - Validates: Performance with many iterations

3. **Buffer Boundary Alignment**
   - Size: 5 JSON objects
   - Buffers: 64, 128, 256, 512, 1024 bytes
   - Validates: Repairs work at any buffer boundary

### Auto-Detection Tests (2 tests)

1. **Auto-Detect Large JSON**
   - Size: 30 key-value pairs
   - Format: Auto-detection
   - Validates: Format detection in streaming

2. **Auto-Detect Large YAML**
   - Size: 30 key-value pairs
   - Format: Auto-detection
   - Validates: YAML auto-detection in streaming

### Special Content Tests (3 tests)

1. **Unicode Large JSON**
   - Content: Multiple languages (Chinese, Arabic, Russian, emoji)
   - Size: 50 items with unicode
   - Validates: Unicode handling in streaming

2. **Multiline Content Large Markdown**
   - Size: 15 sections with multiline paragraphs
   - Validates: Multiline content streaming

3. **Complex CSV with Escaping**
   - Size: 50 rows with escaped quotes and commas
   - Validates: CSV escaping in streaming

### Advanced Tests (1 test)

1. **Large Nested XML**
   - Structure: 3 levels deep (20 × 3 × 2 = 120 elements)
   - Validates: Deep nesting in streaming

## Test Characteristics

### Complexity Metrics

- **Total Items**: 50-500 per test
- **Nesting Depth**: Up to 3 levels
- **Damage Types**: 2-5 per test
- **Buffer Sizes**: 256 bytes to 64KB

### Streaming Efficiency

- **Small Buffer (256B)**: Forces 100+ iterations
- **Medium Buffer (2KB)**: Typical streaming
- **Large Buffer (64KB)**: Optimal performance

### Real-World Scenarios

- API response streaming (JSON)
- Configuration file streaming (YAML, TOML, INI)
- Log file streaming (Markdown)
- Data export streaming (CSV)
- Document streaming (XML)

## Test Execution

### Run All Complex Streaming Tests
```bash
cargo test --test complex_streaming_tests
```

### Run Specific Test
```bash
cargo test --test complex_streaming_tests test_streaming_complex_json_large_nested_structure
```

### Run with Verbose Output
```bash
cargo test --test complex_streaming_tests -- --nocapture
```

### Run Buffer Boundary Test
```bash
cargo test --test complex_streaming_tests test_streaming_buffer_boundary_alignment
```

## Test Results

All 18 complex streaming tests pass successfully:

```
running 18 tests
test test_streaming_auto_detect_large_yaml ... ok
test test_streaming_complex_csv_with_escaping ... ok
test test_streaming_complex_toml_large_config ... ok
test test_streaming_complex_ini_large_config ... ok
test test_streaming_complex_yaml_large_config_with_errors ... ok
test test_streaming_large_nested_xml ... ok
test test_streaming_complex_csv_large_dataset ... ok
test test_streaming_large_buffer_with_complex_yaml ... ok
test test_streaming_multiline_content_large_markdown ... ok
test test_streaming_complex_markdown_large_document ... ok
test test_streaming_auto_detect_large_json ... ok
test test_streaming_unicode_large_json ... ok
test test_streaming_complex_xml_large_nested ... ok
test test_streaming_performance_many_small_chunks ... ok
test test_streaming_mixed_damage_json_large ... ok
test test_streaming_buffer_boundary_alignment ... ok
test test_streaming_complex_json_large_nested_structure ... ok
test test_streaming_very_small_buffer_with_complex_json ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Integration with Test Suite

Complete test coverage breakdown:

| Test Suite | Count | Purpose |
|-----------|-------|---------|
| Unit Tests (lib.rs) | 178 | Core functionality |
| Damage Scenarios | 18 | Real-world damage patterns |
| Complex Damage | 18 | Complex multi-error scenarios |
| Complex Streaming | 18 | Streaming with complex damage |
| Fuzz Tests | 36 | Property-based testing |
| Integration Tests | 4 | End-to-end workflows |
| Streaming Tests | 26 | Basic streaming scenarios |
| **Total** | **298** | **Comprehensive coverage** |

## Performance Insights

### Buffer Size Impact

- **256B buffer**: Many iterations, validates robustness
- **2KB buffer**: Typical production use
- **64KB buffer**: Optimal for large files

### Streaming Advantages

- **Memory**: O(buffer_size) vs O(file_size)
- **Speed**: Comparable to non-streaming
- **Scalability**: Handles files > RAM

### Damage Handling

- Streaming maintains repair quality
- Buffer boundaries don't affect repairs
- Multiple damage types handled correctly

## Use Cases Validated

1. **Large API Responses**: JSON streaming with nested errors
2. **Configuration Management**: YAML/TOML/INI streaming
3. **Data Processing**: CSV streaming with escaping
4. **Document Processing**: Markdown/XML streaming
5. **Performance Testing**: Buffer boundary alignment
6. **Unicode Support**: Multilingual content streaming
7. **Edge Cases**: Very small and large buffers

## Future Enhancements

- [ ] Parallel chunk processing
- [ ] Adaptive buffer sizing
- [ ] Progress callbacks
- [ ] Partial repair mode
- [ ] Streaming validation without repair
- [ ] Memory usage profiling
