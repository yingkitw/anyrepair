//! Golden master tests — checked-in input→expected output pairs.
//!
//! These tests protect against regressions: if a repair strategy changes
//! behavior, the golden master will fail and force a conscious update.

use anyrepair::repair_with_format;

// --- JSON ---

#[test]
fn golden_json_trailing_comma() {
    let input = r#"{"name": "Alice", "age": 30,}"#;
    let expected = r#"{"name": "Alice", "age": 30}"#;
    let result = repair_with_format(input, "json").unwrap();
    assert_eq!(result, expected);
}

#[test]
fn golden_json_unquoted_keys() {
    let input = r#"{name: "Alice", age: 30}"#;
    let result = repair_with_format(input, "json").unwrap();
    assert!(result.contains(r#""name""#));
    assert!(result.contains(r#""age""#));
    assert!(result.contains("Alice"));
    assert!(result.contains("30"));
}

#[test]
fn golden_json_single_quotes() {
    let input = r#"{'key': 'value'}"#;
    let result = repair_with_format(input, "json").unwrap();
    assert!(result.contains(r#""key""#));
    assert!(result.contains(r#""value""#));
}

#[test]
fn golden_json_missing_closing_brace() {
    let input = r#"{"key": "value""#;
    let result = repair_with_format(input, "json").unwrap();
    assert!(result.ends_with('}'));
    assert!(result.contains(r#""key""#));
    assert!(result.contains("value"));
}

// --- YAML ---

#[test]
fn golden_yaml_basic_key_value() {
    let input = "name: John\nage: 30";
    let result = repair_with_format(input, "yaml").unwrap();
    assert!(result.contains("name: John"));
    assert!(result.contains("age: 30"));
}

#[test]
fn golden_yaml_missing_colon() {
    let input = "name John\nage 30";
    let result = repair_with_format(input, "yaml").unwrap();
    // Should add colons to make valid YAML key-value pairs
    assert!(result.contains("John"));
    assert!(result.contains("30"));
}

// --- XML ---

#[test]
fn golden_xml_unclosed_tag() {
    let input = "<root><item>text</root>";
    let result = repair_with_format(input, "xml").unwrap();
    assert!(result.contains("</item>"));
    assert!(result.contains("</root>"));
    assert!(result.contains("text"));
}

#[test]
fn golden_xml_missing_declaration() {
    let input = "<root></root>";
    let result = repair_with_format(input, "xml").unwrap();
    assert!(result.contains("<root>"));
    assert!(result.contains("</root>"));
}

// --- TOML ---

#[test]
fn golden_toml_unquoted_key() {
    let input = "name = John\nage = 30";
    let result = repair_with_format(input, "toml").unwrap();
    assert!(result.contains("name"));
    assert!(result.contains("John"));
    assert!(result.contains("age"));
    assert!(result.contains("30"));
}

#[test]
fn golden_toml_trailing_comma_array() {
    let input = "items = [1, 2, 3,]";
    let result = repair_with_format(input, "toml").unwrap();
    assert!(result.contains("items"));
    assert!(result.contains("1"));
    assert!(result.contains("2"));
    assert!(result.contains("3"));
}

// --- CSV ---

#[test]
fn golden_csv_basic() {
    let input = "name,age\nJohn,30\nJane,25";
    let result = repair_with_format(input, "csv").unwrap();
    assert!(result.contains("name,age"));
    assert!(result.contains("John,30"));
    assert!(result.contains("Jane,25"));
}

#[test]
fn golden_csv_unquoted_comma_field() {
    let input = "name,description\nJohn,Hello, World";
    let result = repair_with_format(input, "csv").unwrap();
    // The field with a comma should be quoted
    assert!(result.contains("John"));
    assert!(result.contains("Hello"));
}

// --- INI ---

#[test]
fn golden_ini_missing_equals() {
    let input = "[user]\nname John\nage = 30";
    let result = repair_with_format(input, "ini").unwrap();
    assert!(result.contains("[user]"));
    assert!(result.contains("name"));
    assert!(result.contains("John"));
    assert!(result.contains("age"));
    assert!(result.contains("30"));
}

#[test]
fn golden_ini_unclosed_section() {
    let input = "[user\nname = John";
    let result = repair_with_format(input, "ini").unwrap();
    assert!(result.contains("[user]"));
    assert!(result.contains("name=John") || result.contains("name = John"));
}

// --- Properties ---

#[test]
fn golden_properties_missing_equals() {
    let input = "server.port 8080\ndb.host localhost";
    let result = repair_with_format(input, "properties").unwrap();
    assert!(result.contains("server.port=8080"));
    assert!(result.contains("db.host=localhost"));
}

#[test]
fn golden_properties_comments_preserved() {
    let input = "# comment\nkey=value\n! another\nfoo=bar";
    let result = repair_with_format(input, "properties").unwrap();
    assert!(result.contains("# comment"));
    assert!(result.contains("! another"));
    assert!(result.contains("key=value"));
    assert!(result.contains("foo=bar"));
}

// --- Env ---

#[test]
fn golden_env_missing_equals() {
    let input = "API_KEY secret123\nDEBUG=true";
    let result = repair_with_format(input, "env").unwrap();
    assert!(result.contains("API_KEY=secret123"));
    assert!(result.contains("DEBUG=true"));
}

#[test]
fn golden_env_comment_preserved() {
    let input = "# config\nKEY=value\nFOO=bar";
    let result = repair_with_format(input, "env").unwrap();
    assert!(result.contains("# config"));
    assert!(result.contains("KEY=value"));
    assert!(result.contains("FOO=bar"));
}

// --- Markdown ---

#[test]
fn golden_md_header_spacing() {
    let input = "#Header\nSome text.";
    let result = repair_with_format(input, "markdown").unwrap();
    assert!(result.contains("# Header"));
    assert!(result.contains("Some text."));
}

#[test]
fn golden_md_code_fence() {
    let input = "```python\nprint('hello')\n```\nText after.";
    let result = repair_with_format(input, "markdown").unwrap();
    assert!(result.contains("```python"));
    assert!(result.contains("print('hello')"));
    assert!(result.contains("```"));
    assert!(result.contains("Text after."));
}

// --- Diff ---

#[test]
fn golden_diff_basic() {
    let input = "--- a/file.txt\n+++ b/file.txt\n@@ -1,2 +1,2 @@\n-old line\n+new line\n context\n";
    let result = repair_with_format(input, "diff").unwrap();
    assert!(result.contains("--- a/file.txt"));
    assert!(result.contains("+++ b/file.txt"));
    assert!(result.contains("@@ -1,2 +1,2 @@"));
    assert!(result.contains("-old line"));
    assert!(result.contains("+new line"));
    assert!(result.contains(" context"));
}

// --- Auto-detect via repair() ---

#[test]
fn golden_auto_detect_json() {
    let input = r#"{"key": "value",}"#;
    let result = anyrepair::repair(input).unwrap();
    assert!(!result.ends_with(','));
    assert!(result.contains(r#""key""#));
    assert!(result.contains("value"));
}

#[test]
fn golden_auto_detect_yaml() {
    let input = "name: John\nage: 30";
    let result = anyrepair::repair(input).unwrap();
    assert!(result.contains("name: John"));
    assert!(result.contains("age: 30"));
}

// --- Idempotency golden tests ---

#[test]
fn golden_json_idempotent() {
    let input = r#"{"key": "value",}"#;
    let once = repair_with_format(input, "json").unwrap();
    let twice = repair_with_format(&once, "json").unwrap();
    assert_eq!(once, twice, "repair should be idempotent");
}

#[test]
fn golden_yaml_idempotent() {
    let input = "name: John\nage: 30";
    let once = repair_with_format(input, "yaml").unwrap();
    let twice = repair_with_format(&once, "yaml").unwrap();
    assert_eq!(once, twice, "repair should be idempotent");
}

#[test]
fn golden_ini_idempotent() {
    let input = "[user]\nname John\nage = 30";
    let once = repair_with_format(input, "ini").unwrap();
    let twice = repair_with_format(&once, "ini").unwrap();
    assert_eq!(once, twice, "repair should be idempotent");
}
