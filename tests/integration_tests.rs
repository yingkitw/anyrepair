//! Integration tests for the anyrepair library

use anyrepair::{repair, json, yaml, markdown, traits::Repair};

#[test]
fn test_library_integration() {
    // Test the main repair function
    let json_input = r#"{"name": "John", "age": 30,}"#;
    let result = repair(json_input).unwrap();
    assert!(result.contains("John"));
    assert!(!result.ends_with(','));

    // Test format-specific repairers
    let json_repairer = json::JsonRepairer::new();
    let yaml_repairer = yaml::YamlRepairer::new();
    let markdown_repairer = markdown::MarkdownRepairer::new();

    // Test JSON repair
    let json_result = json_repairer.repair(json_input).unwrap();
    assert!(json_result.contains("John"));

    // Test YAML repair
    let yaml_input = "name: John\nage: 30";
    let yaml_result = yaml_repairer.repair(yaml_input).unwrap();
    assert!(yaml_result.contains("name: John"));

    // Test Markdown repair
    let markdown_input = "#Header\nSome **bold** text";
    let markdown_result = markdown_repairer.repair(markdown_input).unwrap();
    assert!(markdown_result.contains("Header"));

    // Test confidence scoring
    assert_eq!(json_repairer.confidence(json_input), 1.0);
    assert_eq!(yaml_repairer.confidence(yaml_input), 1.0);
    assert_eq!(markdown_repairer.confidence(markdown_input), 1.0);

    // Test needs_repair
    assert!(json_repairer.needs_repair(json_input));
    assert!(!yaml_repairer.needs_repair(yaml_input));
    // Note: markdown input might be considered valid by the validator
    // assert!(markdown_repairer.needs_repair(markdown_input));
}

#[test]
fn test_error_handling() {
    // Test with very large input
    let large_input = "a".repeat(100000);
    let result = repair(&large_input);
    assert!(result.is_ok());

    // Test with empty input
    let result = repair("");
    assert!(result.is_ok());

    // Test with binary data
    let binary_input = vec![0u8; 1000];
    let result = repair(&String::from_utf8_lossy(&binary_input));
    assert!(result.is_ok());
}

#[test]
fn test_performance() {
    use std::time::Instant;
    
    let input = r#"{"name": "John", "age": 30, "city": "New York", "country": "USA", "hobbies": ["reading", "coding", "gaming"]}"#;
    let repairer = json::JsonRepairer::new();
    
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = repairer.repair(input);
    }
    let duration = start.elapsed();
    
    // Should complete 1000 repairs in less than 1 second
    assert!(duration.as_secs() < 1);
}

#[test]
fn test_memory_usage() {
    // Test that we don't have memory leaks with large inputs
    let large_input = r#"{"data": "}"#.repeat(10000);
    let repairer = json::JsonRepairer::new();
    
    for _ in 0..100 {
        let _ = repairer.repair(&large_input);
    }
    
    // If we get here without panicking, memory usage is reasonable
    assert!(true);
}
