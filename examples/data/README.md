# Test Data Files

This directory contains sample and malformed data files for testing the anyrepair library.

## File Structure

### Valid/Sample Files
These files contain well-formed data that should parse without errors:

- **sample.json** - Valid JSON with nested objects and arrays
- **sample.yaml** - Valid YAML with nested structures
- **sample.xml** - Valid XML with proper formatting
- **sample.csv** - Valid CSV with headers and data rows
- **sample.toml** - Valid TOML with sections and key-value pairs
- **sample.ini** - Valid INI with sections and properties
- **sample.md** - Valid Markdown with headers, lists, and code blocks

### Malformed Files
These files contain common errors that anyrepair should repair:

#### Basic Malformed Files
- **malformed.json** - JSON with trailing commas
- **malformed.yaml** - YAML with missing colons and bad indentation
- **malformed.xml** - XML with unclosed tags
- **malformed.csv** - CSV with missing fields and extra columns
- **malformed.toml** - TOML with syntax errors
- **malformed.ini** - INI with missing equals signs and unclosed sections
- **malformed.md** - Markdown with missing closing tags and formatting issues

#### Complex Malformed Files
- **malformed_complex_api.json** - Complex API response with trailing commas throughout
- **malformed_complex_ecommerce.json** - Complex e-commerce order with multiple trailing commas

#### Prefix/Suffix Malformed Files (Structural Issues)
- **malformed_missing_opening_brace.json** - Missing opening `{` at start
- **malformed_missing_closing_brace.json** - Missing closing `}` at end
- **malformed_missing_opening_bracket.json** - Missing opening `[` in arrays
- **malformed_missing_closing_bracket.json** - Missing closing `]` in arrays
- **malformed_extra_closing_brace.json** - Extra `}` at end
- **malformed_extra_closing_bracket.json** - Extra `]` in arrays
- **malformed_unquoted_keys.json** - Object keys without quotes
- **malformed_unquoted_values.json** - String values without quotes

## Usage in Tests

### Loading Files in Tests

```rust
use std::fs;

#[test]
fn test_with_sample_data() {
    let content = fs::read_to_string("examples/data/sample.json")
        .expect("Failed to read sample file");
    
    let mut repairer = JsonRepairer::new();
    let result = repairer.repair(&content);
    assert!(result.is_ok());
}
```

### Testing Repair Functionality

```rust
#[test]
fn test_repair_malformed_data() {
    let malformed = fs::read_to_string("examples/data/malformed.json")
        .expect("Failed to read malformed file");
    
    let mut repairer = JsonRepairer::new();
    let result = repairer.repair(&malformed);
    
    // Should successfully repair
    assert!(result.is_ok());
    
    // Repaired content should be valid
    let repaired = result.unwrap();
    assert!(serde_json::from_str::<serde_json::Value>(&repaired).is_ok());
}
```

## Common Issues in Malformed Files

### JSON Issues
- Trailing commas in objects and arrays
- Missing quotes around keys
- Unescaped special characters

### YAML Issues
- Missing colons after keys
- Incorrect indentation
- Missing list markers

### XML Issues
- Unclosed tags
- Missing closing brackets
- Improper nesting

### CSV Issues
- Missing fields
- Extra columns
- Trailing commas

### TOML Issues
- Missing equals signs
- Unclosed sections
- Trailing commas

### INI Issues
- Missing equals signs
- Unclosed section headers
- Incorrect syntax

### Markdown Issues
- Missing closing markers for bold/italic
- Missing spaces after headers
- Unclosed code blocks

## Adding New Test Files

When adding new test files:

1. Create both a valid (`sample.ext`) and malformed (`malformed.ext`) version
2. Document the specific issues in the malformed version
3. Ensure the files are representative of real-world data
4. Update this README with descriptions
