//! Tests for strict validation mode (requires `--features strict`).
//! Run with: `cargo test --features strict --test strict_validation_tests`

#![cfg(feature = "strict")]
#![allow(unused_imports)]

use anyrepair::create_validator;
use anyrepair::traits::Validator;

#[test]
fn strict_json_valid() {
    let v = create_validator("json").unwrap();
    assert!(v.is_valid(r#"{"key": "value"}"#));
    assert!(v.is_valid(r#"[1, 2, 3]"#));
    assert!(v.is_valid("42"));
    assert!(v.is_valid(r#""hello""#));
    assert!(v.is_valid("true"));
    assert!(v.is_valid("null"));
}

#[test]
fn strict_json_invalid_trailing_comma() {
    let v = create_validator("json").unwrap();
    assert!(!v.is_valid(r#"{"key": "value",}"#));
}

#[test]
fn strict_json_invalid_unquoted_keys() {
    let v = create_validator("json").unwrap();
    assert!(!v.is_valid(r#"{name: "Alice"}"#));
}

#[test]
fn strict_json_invalid_single_quotes() {
    let v = create_validator("json").unwrap();
    assert!(!v.is_valid(r#"{'key': 'value'}"#));
}

#[test]
fn strict_json_validate_returns_serde_error() {
    let v = create_validator("json").unwrap();
    let errors = v.validate(r#"{"key": "value",}"#);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("trailing") || errors[0].contains("comma") || !errors.is_empty());
}

#[test]
fn strict_json_validate_valid_returns_empty() {
    let v = create_validator("json").unwrap();
    let errors = v.validate(r#"{"key": "value"}"#);
    assert!(errors.is_empty());
}

#[test]
fn strict_json_repair_still_works() {
    let input = r#"{"key": "value",}"#;
    let result = anyrepair::repair_with_format(input, "json").unwrap();
    // After repair, the result should be valid JSON
    let v = create_validator("json").unwrap();
    assert!(v.is_valid(&result), "repaired output should be valid JSON: {}", result);
}

#[test]
fn strict_json_nested_structures() {
    let v = create_validator("json").unwrap();
    assert!(v.is_valid(r#"{"a": {"b": {"c": [1, 2, {"d": true}]}}}"#));
    assert!(!v.is_valid(r#"{"a": {"b": {"c": [1, 2, {"d": true,}]}}}"#));
}
