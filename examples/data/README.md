# Test Data Files

This directory contains sample and malformed data files organized by format for testing the anyrepair library.

## Directory Structure

```
data/
├── json/
│   ├── sample/          # Valid JSON files
│   ├── malformed/       # Basic malformed JSON files
│   └── complex/         # Complex JSON files (valid and malformed)
├── yaml/
│   ├── sample/
│   └── malformed/
├── markdown/
│   ├── sample/
│   └── malformed/
├── xml/
│   ├── sample/
│   └── malformed/
├── toml/
│   ├── sample/
│   └── malformed/
├── csv/
│   ├── sample/
│   └── malformed/
├── ini/
│   ├── sample/
│   └── malformed/
└── diff/
    ├── sample/          # Valid unified diff files
    ├── malformed/       # Basic malformed diff files
    └── complex/         # Complex diff files with multiple issues
```

## File Organization

### JSON Files (`json/`)

#### Sample Files (`json/sample/`)
- **sample.json** - Valid JSON with nested objects and arrays

#### Malformed Files (`json/malformed/`)
- **malformed.json** - JSON with trailing commas
- **malformed_unquoted_keys.json** - Object keys without quotes
- **malformed_unquoted_values.json** - String values without quotes
- **malformed_missing_opening_brace.json** - Missing opening `{` at start
- **malformed_missing_closing_brace.json** - Missing closing `}` at end
- **malformed_extra_closing_brace.json** - Extra `}` at end
- **malformed_missing_opening_bracket.json** - Missing opening `[` in arrays
- **malformed_missing_closing_bracket.json** - Missing closing `]` in arrays
- **malformed_extra_closing_bracket.json** - Extra `]` in arrays

#### Complex Files (`json/complex/`)
- **complex_api_response.json** - Complex API response structure
- **complex_config.json** - Complex configuration structure
- **complex_ecommerce.json** - Complex e-commerce order structure
- **mailform_response.json** - Mail form response structure
- **malformed_complex_api.json** - Complex API response with trailing commas throughout
- **malformed_complex_ecommerce.json** - Complex e-commerce order with multiple trailing commas

### YAML Files (`yaml/`)

#### Sample Files (`yaml/sample/`)
- **sample.yaml** - Valid YAML with nested structures

#### Malformed Files (`yaml/malformed/`)
- **malformed.yaml** - YAML with missing colons and bad indentation

### Markdown Files (`markdown/`)

#### Sample Files (`markdown/sample/`)
- **sample.md** - Valid Markdown with headers, lists, and code blocks

#### Malformed Files (`markdown/malformed/`)
- **malformed.md** - Markdown with missing closing tags and formatting issues

### XML Files (`xml/`)

#### Sample Files (`xml/sample/`)
- **sample.xml** - Valid XML with proper formatting

#### Malformed Files (`xml/malformed/`)
- **malformed.xml** - XML with unclosed tags

### TOML Files (`toml/`)

#### Sample Files (`toml/sample/`)
- **sample.toml** - Valid TOML with sections and key-value pairs

#### Malformed Files (`toml/malformed/`)
- **malformed.toml** - TOML with syntax errors

### CSV Files (`csv/`)

#### Sample Files (`csv/sample/`)
- **sample.csv** - Valid CSV with headers and data rows

#### Malformed Files (`csv/malformed/`)
- **malformed.csv** - CSV with missing fields and extra columns

### INI Files (`ini/`)

#### Sample Files (`ini/sample/`)
- **sample.ini** - Valid INI with sections and properties

#### Malformed Files (`ini/malformed/`)
- **malformed.ini** - INI with missing equals signs and unclosed sections

### Diff Files (`diff/`)

#### Sample Files (`diff/sample/`)
- **sample.diff** - Valid unified diff with multiple hunks and proper formatting

#### Malformed Files (`diff/malformed/`)
- **malformed.diff** - Diff with missing file headers
- **malformed_missing_hunk_header.diff** - Diff with missing hunk headers
- **malformed_missing_file_headers.diff** - Diff without --- and +++ headers
- **malformed_incorrect_prefixes.diff** - Diff lines without proper +, -, or space prefixes
- **malformed_malformed_hunk_range.diff** - Diff with invalid hunk range numbers
- **malformed_inconsistent_spacing.diff** - Diff with excessive spacing in hunk headers
- **malformed_missing_newline.diff** - Diff without trailing newline

#### Complex Files (`diff/complex/`)
- **malformed_complex.diff** - Complex diff with multiple hunks, configuration changes, and strategy additions
- **multi_file_complex.diff** - Multi-file diff showing changes across multiple source files
- **large_hunk_complex.diff** - Large hunk with extensive code changes, statistics tracking, and configuration
- **mixed_changes_complex.diff** - Mixed additions, deletions, and modifications with priority system
- **real_world_git.diff** - Real-world git diff format with multiple files, new files, and feature flags

## Usage in Tests

### Loading Files in Tests

```rust
use std::fs;
use std::path::Path;

#[test]
fn test_with_sample_data() {
    let sample_path = Path::new("examples/data/json/sample/sample.json");
    if !sample_path.exists() {
        // Skip if examples directory not available (e.g., in published crate)
        return;
    }
    
    let content = fs::read_to_string(sample_path)
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
    let malformed_path = Path::new("examples/data/json/malformed/malformed.json");
    if !malformed_path.exists() {
        return;
    }
    
    let malformed = fs::read_to_string(malformed_path)
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

### Testing All Malformed Files

```rust
#[test]
fn test_all_malformed_files() {
    let malformed_files = vec![
        "malformed.json",
        "malformed_unquoted_keys.json",
        "malformed_unquoted_values.json",
        // ... etc
    ];
    
    for filename in malformed_files {
        let file_path = Path::new("examples/data/json/malformed").join(filename);
        if !file_path.exists() {
            continue;
        }
        
        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", filename));
        
        let mut repairer = JsonRepairer::new();
        let result = repairer.repair(&content);
        
        assert!(result.is_ok(), "Failed to repair {}", filename);
    }
}
```

## Common Issues by Format

### JSON Issues
- Trailing commas in objects and arrays
- Missing quotes around keys
- Unescaped special characters
- Missing or extra braces/brackets

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

### Diff Issues
- Missing hunk headers (@@)
- Missing file headers (--- and +++)
- Incorrect line prefixes (should be +, -, or space)
- Malformed hunk range numbers
- Inconsistent spacing in hunk headers
- Missing trailing newlines

## Adding New Test Files

When adding new test files:

1. Place files in the appropriate format directory (`json/`, `yaml/`, etc.)
2. Use `sample/` subdirectory for valid files
3. Use `malformed/` subdirectory for files with errors
4. Use `complex/` subdirectory for complex or large files
5. Document the specific issues in the malformed version
6. Ensure the files are representative of real-world data
7. Update this README with descriptions

## Path Resolution

When using these files in tests, use relative paths from the project root:

```rust
// From project root
let path = Path::new("examples/data/json/sample/sample.json");

// Or use env!("CARGO_MANIFEST_DIR") for absolute paths
let manifest_dir = env!("CARGO_MANIFEST_DIR");
let path = Path::new(manifest_dir).join("examples/data/json/sample/sample.json");
```
