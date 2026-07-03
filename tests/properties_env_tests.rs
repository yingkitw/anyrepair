//! Dedicated integration tests for .properties and .env repair formats.

use anyrepair::key_value::{EnvRepairer, PropertiesRepairer};
use anyrepair::traits::{Repair, Validator};
use anyrepair::{create_repairer, detect_format, repair};

// --- Properties tests ---

#[test]
fn test_properties_basic_repair() {
    let mut repairer = PropertiesRepairer::new();
    let input = "server.port=8080\ndb.host localhost";
    let result = repairer.repair(input).unwrap();
    assert!(result.contains("server.port=8080"));
    assert!(result.contains("db.host=localhost"));
}

#[test]
fn test_properties_missing_equals() {
    let mut repairer = PropertiesRepairer::new();
    let input = "key value\nfoo=bar";
    let result = repairer.repair(input).unwrap();
    assert!(result.contains("key=value"));
    assert!(result.contains("foo=bar"));
}

#[test]
fn test_properties_comments_preserved() {
    let mut repairer = PropertiesRepairer::new();
    let input = "# This is a comment\nkey=value\n! Another comment\nfoo=bar";
    let result = repairer.repair(input).unwrap();
    assert!(result.contains("# This is a comment"));
    assert!(result.contains("! Another comment"));
    assert!(result.contains("key=value"));
    assert!(result.contains("foo=bar"));
}

#[test]
fn test_properties_whitespace_around_equals() {
    let mut repairer = PropertiesRepairer::new();
    let input = "key  =  value\nfoo   =   bar";
    let result = repairer.repair(input).unwrap();
    // Validator considers whitespace-padded key=value as valid, so it passes through
    assert!(result.contains("key"));
    assert!(result.contains("value"));
}

#[test]
fn test_properties_dot_keys() {
    let mut repairer = PropertiesRepairer::new();
    let input = "app.name=MyApp\napp.version=1.0\ndatabase.url=localhost";
    let result = repairer.repair(input).unwrap();
    assert!(result.contains("app.name=MyApp"));
    assert!(result.contains("app.version=1.0"));
    assert!(result.contains("database.url=localhost"));
}

#[test]
fn test_properties_empty_input() {
    let mut repairer = PropertiesRepairer::new();
    let result = repairer.repair("").unwrap();
    assert!(result.is_empty() || result.trim().is_empty());
}

#[test]
fn test_properties_valid_passes_through() {
    let mut repairer = PropertiesRepairer::new();
    let input = "key=value\nfoo=bar";
    let result = repairer.repair(input).unwrap();
    assert_eq!(result, input);
}

#[test]
fn test_properties_via_registry() {
    let mut repairer = create_repairer("properties").unwrap();
    let input = "key value\nfoo=bar";
    let result = repairer.repair(input).unwrap();
    assert!(result.contains("key=value"));
}

#[test]
fn test_properties_format_detection() {
    let input = "app.name=MyApp\napp.version=1.0\n# comment\ndb.host=localhost";
    let detected = detect_format(input);
    assert_eq!(detected, Some("properties"));
}

#[test]
fn test_properties_repair_via_top_level() {
    let input = "key value\nfoo=bar";
    let result = repair(input).unwrap();
    assert!(result.contains("key=value"));
}

// --- Env tests ---

#[test]
fn test_env_basic_repair() {
    let mut repairer = EnvRepairer::new();
    let input = "DATABASE_URL=postgres://localhost\nPORT 8080";
    let result = repairer.repair(input).unwrap();
    assert!(result.contains("DATABASE_URL=postgres://localhost"));
    assert!(result.contains("PORT=8080"));
}

#[test]
fn test_env_missing_equals() {
    let mut repairer = EnvRepairer::new();
    let input = "API_KEY secret123\nDEBUG=true";
    let result = repairer.repair(input).unwrap();
    assert!(result.contains("API_KEY=secret123"));
    assert!(result.contains("DEBUG=true"));
}

#[test]
fn test_env_comments_preserved() {
    let mut repairer = EnvRepairer::new();
    let input = "# Production config\nKEY=value\n# Another comment\nFOO=bar";
    let result = repairer.repair(input).unwrap();
    assert!(result.contains("# Production config"));
    assert!(result.contains("# Another comment"));
    assert!(result.contains("KEY=value"));
    assert!(result.contains("FOO=bar"));
}

#[test]
fn test_env_mixed_case_keys_preserved() {
    let mut repairer = EnvRepairer::new();
    let input = "database_url=localhost\nPORT=8080";
    let result = repairer.repair(input).unwrap();
    // Env repairer does not uppercase keys; it just fixes structural issues
    assert!(result.contains("database_url=localhost"));
    assert!(result.contains("PORT=8080"));
}

#[test]
fn test_env_whitespace_around_equals() {
    let mut repairer = EnvRepairer::new();
    let input = "KEY  =  value\nFOO   =   bar";
    let result = repairer.repair(input).unwrap();
    // Validator considers whitespace-padded key=value as valid, so it passes through
    assert!(result.contains("KEY"));
    assert!(result.contains("value"));
}

#[test]
fn test_env_empty_input() {
    let mut repairer = EnvRepairer::new();
    let result = repairer.repair("").unwrap();
    assert!(result.is_empty() || result.trim().is_empty());
}

#[test]
fn test_env_valid_passes_through() {
    let mut repairer = EnvRepairer::new();
    let input = "KEY=value\nFOO=bar";
    let result = repairer.repair(input).unwrap();
    assert_eq!(result, input);
}

#[test]
fn test_env_via_registry() {
    let mut repairer = create_repairer("env").unwrap();
    let input = "API_KEY secret\nDEBUG=true";
    let result = repairer.repair(input).unwrap();
    assert!(result.contains("API_KEY=secret"));
}

#[test]
fn test_env_format_detection() {
    let input = "DATABASE_URL=localhost\nPORT=8080\nSECRET_KEY=abc123";
    let detected = detect_format(input);
    assert_eq!(detected, Some("env"));
}

#[test]
fn test_env_repair_via_top_level() {
    let input = "API_KEY secret\nDEBUG=true";
    let result = repair(input).unwrap();
    assert!(result.contains("API_KEY=secret"));
}

// --- Validator tests ---

#[test]
fn test_properties_validator_valid() {
    let validator = anyrepair::key_value::PropertiesValidator;
    assert!(validator.is_valid("key=value\nfoo=bar"));
}

#[test]
fn test_properties_validator_invalid_missing_equals() {
    let validator = anyrepair::key_value::PropertiesValidator;
    assert!(!validator.is_valid("key value\nfoo=bar"));
}

#[test]
fn test_env_validator_valid() {
    let validator = anyrepair::key_value::EnvValidator;
    assert!(validator.is_valid("KEY=value\nFOO=bar"));
}

#[test]
fn test_env_validator_accepts_mixed_case() {
    let validator = anyrepair::key_value::EnvValidator;
    // EnvValidator checks structure (has =, non-empty key), not case
    assert!(validator.is_valid("key=value\nFOO=bar"));
}

#[test]
fn test_env_validator_invalid_missing_equals() {
    let validator = anyrepair::key_value::EnvValidator;
    assert!(!validator.is_valid("KEY value\nFOO=bar"));
}
