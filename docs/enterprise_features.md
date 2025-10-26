# Enterprise Features

AnyRepair includes comprehensive enterprise-grade features for advanced analytics, batch processing, validation rules, and audit logging.

## Advanced Analytics

The analytics module tracks repair operations and provides detailed metrics about repair success rates and performance.

### Features

- **Repair Success Metrics**: Track successful and failed repairs
- **Performance Monitoring**: Measure repair operation timings
- **Format-Specific Metrics**: Get metrics broken down by file format
- **Success Rate Calculation**: Calculate overall success rates

### Usage

```rust
use anyrepair::AnalyticsTracker;
use std::time::Duration;

let tracker = AnalyticsTracker::new();

// Record a repair operation
tracker.record_repair("json", true, Duration::from_millis(10), 0.95);

// Get metrics
let metrics = tracker.get_metrics();
println!("Total repairs: {}", metrics.total_repairs);
println!("Success rate: {}%", tracker.get_success_rate());

// Get format-specific metrics
if let Some(json_metrics) = tracker.get_format_metrics("json") {
    println!("JSON repairs: {}", json_metrics.repair_count);
    println!("JSON success rate: {}%", 
             (json_metrics.success_count as f64 / json_metrics.repair_count as f64) * 100.0);
}
```

## Batch Processing

Process multiple files across different formats with automatic format detection and parallel processing support.

### Features

- **Directory Processing**: Process entire directories recursively
- **Multi-Format Support**: Automatically detects and repairs different formats
- **File Filtering**: Filter files by extension
- **Detailed Results**: Get per-file repair results and statistics
- **Integrated Analytics**: Automatic tracking of batch operations

### Usage

```rust
use anyrepair::BatchProcessor;
use std::path::Path;

let processor = BatchProcessor::new();

// Process all files in a directory
let results = processor.process_directory(
    Path::new("./data"),
    true,  // recursive
    Some(&["json", "yaml", "xml"])  // filter by extensions
)?;

println!("Processed: {}", results.total_files);
println!("Successful: {}", results.successful_files);
println!("Failed: {}", results.failed_files);

// Access individual file results
for file_result in &results.file_results {
    println!("{}: {} ({}ms)", 
             file_result.file_path,
             if file_result.success { "OK" } else { "FAILED" },
             file_result.time_ms);
}

// Get analytics from batch processor
let metrics = processor.analytics().get_metrics();
```

## Custom Validation Rules

Define and enforce custom validation rules for repaired content.

### Features

- **Multiple Rule Types**: Regex, Length, Format, Custom
- **Rule Management**: Add, remove, enable/disable rules
- **Flexible Validation**: Validate against multiple rules simultaneously
- **Detailed Violations**: Get specific violation messages

### Usage

```rust
use anyrepair::ValidationRulesEngine;
use anyrepair::validation_rules::{ValidationRule, RuleType};

let mut engine = ValidationRulesEngine::new();

// Add a length constraint rule
let rule = ValidationRule {
    name: "max_size".to_string(),
    rule_type: RuleType::Length,
    pattern: "10000".to_string(),
    error_message: "Content exceeds maximum size".to_string(),
    enabled: true,
};
engine.add_rule(rule);

// Add a format validation rule
let rule = ValidationRule {
    name: "json_format".to_string(),
    rule_type: RuleType::Format,
    pattern: "json".to_string(),
    error_message: "Invalid JSON format".to_string(),
    enabled: true,
};
engine.add_rule(rule);

// Validate content
let result = engine.validate(r#"{"key": "value"}"#);
if result.passed {
    println!("Validation passed!");
} else {
    for violation in result.violations {
        println!("Violation: {}", violation);
    }
}

// Disable a rule
engine.disable_rule("max_size");
```

## Audit Logging

Comprehensive audit logging for compliance and tracking of all repair operations.

### Features

- **Event Tracking**: Log repairs, validations, batch operations, and configuration changes
- **Detailed Entries**: Timestamp, actor, resource, action, result, and custom details
- **File Persistence**: Optional file-based logging for compliance
- **Query Capabilities**: Filter entries by type or actor
- **JSON Format**: Structured logging for easy parsing and analysis

### Usage

```rust
use anyrepair::AuditLogger;

// Create logger with file output
let logger = AuditLogger::with_file("audit.log");

// Log a repair operation
logger.log_repair("data.json", "json", true, "user@example.com", Some("Automated repair"));

// Log a validation operation
logger.log_validation("data.json", true, "user@example.com", None);

// Log a batch operation
logger.log_batch_operation("batch_001", 100, 98, 2, "system");

// Log a configuration change
logger.log_config_change("max_file_size", "1MB", "10MB", "admin@example.com");

// Query audit entries
let all_entries = logger.get_entries();
let repairs = logger.get_entries_by_type("REPAIR");
let user_actions = logger.get_entries_by_actor("user@example.com");

println!("Total audit entries: {}", logger.entry_count());
```

## Integration Example

Here's a complete example combining all enterprise features:

```rust
use anyrepair::{AnalyticsTracker, BatchProcessor, ValidationRulesEngine, AuditLogger};
use anyrepair::validation_rules::{ValidationRule, RuleType};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize enterprise components
    let batch_processor = BatchProcessor::new();
    let mut validation_engine = ValidationRulesEngine::new();
    let audit_logger = AuditLogger::with_file("audit.log");

    // Add validation rules
    let rule = ValidationRule {
        name: "json_format".to_string(),
        rule_type: RuleType::Format,
        pattern: "json".to_string(),
        error_message: "Invalid JSON".to_string(),
        enabled: true,
    };
    validation_engine.add_rule(rule);

    // Process batch
    let results = batch_processor.process_directory(
        Path::new("./data"),
        true,
        Some(&["json", "yaml"])
    )?;

    // Log batch operation
    audit_logger.log_batch_operation(
        "batch_001",
        results.total_files,
        results.successful_files,
        results.failed_files,
        "system"
    );

    // Get analytics
    let metrics = batch_processor.analytics().get_metrics();
    println!("Success rate: {}%", batch_processor.analytics().get_success_rate());

    Ok(())
}
```

## Performance Considerations

- **Analytics**: Minimal overhead, uses atomic operations for thread-safe tracking
- **Batch Processing**: Supports parallel processing for large file sets
- **Validation**: Efficient regex caching for repeated validations
- **Audit Logging**: Async file I/O to minimize performance impact

## Best Practices

1. **Reuse Trackers**: Create analytics trackers once and reuse them
2. **Filter Extensions**: Use extension filtering in batch processing to avoid unnecessary file reads
3. **Validation Rules**: Disable unused rules to improve validation performance
4. **Audit Retention**: Implement log rotation for long-running systems
5. **Error Handling**: Always check validation results before processing
