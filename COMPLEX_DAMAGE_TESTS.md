# Complex Damage Test Cases

## Overview

The `tests/complex_damage_tests.rs` file contains 18 comprehensive test cases covering realistic and complex damage patterns from LLM outputs across all supported formats.

## Test Coverage by Format

### JSON (4 tests)

1. **Deeply Nested with Multiple Errors**
   - Tests: Nested objects up to 5 levels deep
   - Damage: Trailing commas, mixed indentation
   - Validates: Proper repair of complex hierarchies

2. **Mixed Quote Styles and Missing Quotes**
   - Tests: Single quotes, double quotes, unquoted keys
   - Damage: Inconsistent quoting, missing colons
   - Validates: Quote normalization and key-value repair

3. **API Response with Nested Errors**
   - Tests: Realistic API response structure
   - Damage: Trailing commas in arrays and objects
   - Validates: Pagination and nested data repair

4. **Unicode and Escape Sequences**
   - Tests: Unicode characters, emoji, escape sequences
   - Damage: Mixed encodings, malformed escapes
   - Validates: Unicode handling and escape repair

### YAML (3 tests)

1. **Indentation and List Mixing**
   - Tests: Complex nested structures with lists
   - Damage: Indentation inconsistencies
   - Validates: Proper indentation detection and fixing

2. **Config with Anchors and References**
   - Tests: YAML anchors (&) and aliases (*)
   - Damage: Malformed references
   - Validates: Anchor/reference handling

3. **Multiline Strings**
   - Tests: Literal (|) and folded (>) string blocks
   - Damage: Inconsistent formatting
   - Validates: Multiline string preservation

### Markdown (3 tests)

1. **Mixed Formatting and Nested Structures**
   - Tests: Headers, bold, italic, code blocks, lists
   - Damage: Missing spaces after #, unmatched markers
   - Validates: Complete markdown repair

2. **Code Blocks and Formatting**
   - Tests: Code blocks with multiple languages
   - Damage: Mismatched fence markers
   - Validates: Code block integrity

3. **Tables and Lists**
   - Tests: Markdown tables with alignment
   - Damage: Malformed table separators
   - Validates: Table and list structure repair

### XML (2 tests)

1. **Nested with Attributes and Entities**
   - Tests: Complex nested elements with attributes
   - Damage: Unclosed tags, missing quotes
   - Validates: Proper XML structure restoration

2. **CDATA and Mixed Content**
   - Tests: CDATA sections, mixed text and elements
   - Damage: Malformed CDATA markers
   - Validates: CDATA preservation and content repair

### CSV (2 tests)

1. **Quoted Fields and Special Characters**
   - Tests: Complex quoted fields with commas and quotes
   - Damage: Unescaped quotes, missing delimiters
   - Validates: CSV field parsing and escaping

2. **Multiline Fields**
   - Tests: Fields spanning multiple lines
   - Damage: Improper line breaks in fields
   - Validates: Multiline field handling

### TOML (2 tests)

1. **Nested Tables and Arrays**
   - Tests: Complex table hierarchies and arrays
   - Damage: Missing quotes, malformed arrays
   - Validates: Table and array structure repair

2. **Inline Tables**
   - Tests: Inline table syntax with multiple fields
   - Damage: Missing commas, unquoted values
   - Validates: Inline table parsing

### INI (2 tests)

1. **Multiple Sections and Comments**
   - Tests: Multiple sections with various key-value pairs
   - Damage: Malformed sections, missing equals
   - Validates: Section and key-value repair

2. **Special Characters**
   - Tests: URLs, paths, special characters in values
   - Damage: Unescaped special characters
   - Validates: Special character handling

## Test Characteristics

### Complexity Levels

- **Deep Nesting**: Up to 5 levels of nested structures
- **Multiple Errors**: 3-5 different damage types per test
- **Real-World Patterns**: Based on actual LLM output issues
- **Format-Specific Features**: Tests unique format capabilities

### Damage Types Covered

1. **Structural Damage**
   - Missing/extra brackets, braces, quotes
   - Unclosed tags or blocks
   - Malformed delimiters

2. **Formatting Damage**
   - Indentation inconsistencies
   - Missing spaces or separators
   - Incorrect line breaks

3. **Content Damage**
   - Mixed quote styles
   - Unescaped special characters
   - Encoding issues

4. **Semantic Damage**
   - Trailing commas
   - Missing colons or equals
   - Malformed references

## Test Execution

### Running All Complex Damage Tests
```bash
cargo test --test complex_damage_tests
```

### Running Specific Test
```bash
cargo test --test complex_damage_tests test_complex_json_deeply_nested_with_multiple_errors
```

### Running with Output
```bash
cargo test --test complex_damage_tests -- --nocapture
```

## Test Results

All 18 complex damage tests pass successfully:

```
running 18 tests
test test_complex_ini_with_multiple_sections_and_comments ... ok
test test_complex_ini_with_special_characters ... ok
test test_complex_csv_with_multiline_fields ... ok
test test_complex_csv_with_quoted_fields_and_special_chars ... ok
test test_complex_markdown_mixed_formatting_and_nested_structures ... ok
test test_complex_markdown_with_code_blocks_and_formatting ... ok
test test_complex_xml_nested_with_attributes_and_entities ... ok
test test_complex_xml_with_cdata_and_mixed_content ... ok
test test_complex_markdown_with_tables_and_lists ... ok
test test_complex_yaml_config_with_anchors_and_references ... ok
test test_complex_yaml_indentation_and_list_mixing ... ok
test test_complex_yaml_with_multiline_strings ... ok
test test_complex_json_with_unicode_and_escape_sequences ... ok
test test_complex_toml_with_nested_tables_and_arrays ... ok
test test_complex_toml_with_inline_tables ... ok
test test_complex_json_mixed_quote_styles_and_missing_quotes ... ok
test test_complex_json_api_response_with_nested_errors ... ok
test test_complex_json_deeply_nested_with_multiple_errors ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Integration with Test Suite

These tests complement the existing test coverage:

| Test Suite | Count | Purpose |
|-----------|-------|---------|
| Unit Tests (lib.rs) | 178 | Core functionality |
| Damage Scenarios | 18 | Real-world damage patterns |
| Complex Damage | 18 | Complex multi-error scenarios |
| Fuzz Tests | 36 | Property-based testing |
| Integration Tests | 4 | End-to-end workflows |
| Streaming Tests | 26 | Large file handling |
| **Total** | **280** | **Comprehensive coverage** |

## Use Cases

These tests validate AnyRepair's ability to handle:

1. **LLM Output Repair**: Complex outputs from language models with multiple errors
2. **Data Pipeline Cleanup**: Malformed data from various sources
3. **Configuration File Recovery**: Corrupted config files with multiple issues
4. **API Response Handling**: Malformed API responses with nested structures
5. **Log File Repair**: Complex log entries with mixed formatting

## Future Enhancements

Potential additions:
- [ ] Performance benchmarks for complex repairs
- [ ] Streaming repair of complex structures
- [ ] Partial repair validation
- [ ] Confidence scoring for complex repairs
- [ ] Repair strategy analysis for complex cases
