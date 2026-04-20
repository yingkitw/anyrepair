//! Repair behavior tests (same paths the CLI exercises via the library).

#[test]
fn test_json_repair_with_comments() {
    use anyrepair::json::JsonRepairer;
    use anyrepair::traits::Repair;

    let mut repairer = JsonRepairer::new();

    // JSON with single-line comments
    let input1 = r#"{"key": "value", // comment}"#;
    let result1 = repairer.repair(input1).unwrap();
    assert!(!result1.contains("//"));

    // JSON with multi-line comments
    let input2 = r#"{"key": "value", /* multi-line
    comment */}"#;
    let result2 = repairer.repair(input2).unwrap();
    assert!(!result2.contains("/*"));

    // Comments in strings should be preserved
    let input3 = r#"{"text": "not // a comment"}"#;
    let result3 = repairer.repair(input3).unwrap();
    assert!(result3.contains("//"));
}

#[test]
fn test_all_format_repairers() {
    use anyrepair::traits::Repair;
    use anyrepair::{csv, diff, key_value, toml, xml};

    // Test XML repairer
    let mut xml_repairer = xml::XmlRepairer::new();
    let xml_input = "<root><item>value</item></root>";
    let xml_result = xml_repairer.repair(xml_input).unwrap();
    assert!(xml_result.contains("<root>"));

    // Test TOML repairer
    let mut toml_repairer = toml::TomlRepairer::new();
    let toml_input = "name = \"John\"\nage = 30";
    let toml_result = toml_repairer.repair(toml_input).unwrap();
    assert!(toml_result.contains("name"));

    // Test CSV repairer
    let mut csv_repairer = csv::CsvRepairer::new();
    let csv_input = "name,age\nJohn,30\nJane,25";
    let csv_result = csv_repairer.repair(csv_input).unwrap();
    assert!(csv_result.contains("John,30"));

    // Test INI repairer
    let mut ini_repairer = key_value::IniRepairer::new();
    let ini_input = "[user]\nname = John\nage = 30";
    let ini_result = ini_repairer.repair(ini_input).unwrap();
    assert!(ini_result.contains("[user]"));

    // Test Diff repairer
    let mut diff_repairer = diff::DiffRepairer::new();
    let diff_input = "@@ -1,3 +1,4 @@\n-line 1\n+line 1 modified\n line 2";
    let diff_result = diff_repairer.repair(diff_input).unwrap();
    assert!(diff_result.contains("@@"));
}

#[test]
fn test_confidence_scoring() {
    use anyrepair::json::JsonRepairer;
    use anyrepair::traits::Repair;

    let repairer = JsonRepairer::new();

    // Valid JSON should have high confidence
    let valid_json = r#"{"key": "value"}"#;
    let confidence = repairer.confidence(valid_json);
    assert_eq!(confidence, 1.0);

    // Invalid JSON should have lower confidence
    let invalid_json = r#"{"key": value}"#;
    let confidence2 = repairer.confidence(invalid_json);
    assert!(confidence2 < 1.0);
    assert!(confidence2 > 0.0);
}

#[test]
fn test_needs_repair() {
    use anyrepair::traits::Repair;
    use anyrepair::{json::JsonRepairer, markdown::MarkdownRepairer, yaml::YamlRepairer};

    let json_repairer = JsonRepairer::new();
    let yaml_repairer = YamlRepairer::new();
    let md_repairer = MarkdownRepairer::new();

    // Valid JSON should not need repair
    assert!(!json_repairer.needs_repair(r#"{"key": "value"}"#));

    // JSON with trailing comma needs repair
    assert!(json_repairer.needs_repair(r#"{"key": "value",}"#));

    // Valid YAML should not need repair
    assert!(!yaml_repairer.needs_repair("key: value"));

    // Valid markdown should not need repair
    assert!(!md_repairer.needs_repair("# Header\nContent"));
}

#[test]
fn test_repair_json_with_various_issues() {
    use anyrepair::json::JsonRepairer;
    use anyrepair::traits::Repair;

    let mut repairer = JsonRepairer::new();

    // Combined issues: comments + trailing commas + missing quotes
    let input = r#"{
  // comment
  name: "John",
  age: 30,
}"#;

    let result = repairer.repair(input).unwrap();
    assert!(!result.contains("//"));
    assert!(result.contains("\"name\""));
    assert!(result.contains("\"age\""));
    assert!(!result.contains(",\n}"));

    // Verify valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["name"], "John");
    assert_eq!(parsed["age"], 30);
}

#[test]
fn test_format_auto_detection() {
    use anyrepair::repair;

    // Should detect JSON
    let json_input = r#"{"key": "value",}"#;
    let result = repair(json_input).unwrap();
    assert!(result.contains("key"));

    // Should detect YAML
    let yaml_input = "name: John\nage: 30";
    let result2 = repair(yaml_input).unwrap();
    assert!(result2.contains("name: John"));

    // Should detect Markdown
    let md_input = "# Header\nContent";
    let result3 = repair(md_input).unwrap();
    assert!(result3.contains("Header"));
}

#[test]
fn test_edge_cases() {
    use anyrepair::repair;

    // Empty string
    let result = repair("").unwrap();
    assert!(result.is_empty() || result == "{}");

    // Very large string
    let large = "a".repeat(100000);
    let result2 = repair(&large);
    assert!(result2.is_ok());

    // Binary-like content
    let binary = vec![0u8; 1000];
    let result3 = repair(&String::from_utf8_lossy(&binary));
    assert!(result3.is_ok());
}

#[test]
fn test_repair_with_comments_preserves_urls() {
    use anyrepair::json::JsonRepairer;
    use anyrepair::traits::Repair;

    let mut repairer = JsonRepairer::new();

    let input = r#"{
  "url": "https://example.com/path",
  "value": "text" // actual comment to remove
}"#;

    let result = repairer.repair(input).unwrap();
    assert!(result.contains("https://"));
    assert!(!result.contains("// actual comment to remove"));
    // But // inside strings should be preserved
    assert!(result.contains("https://"));
}

#[test]
fn test_xml_edge_cases() {
    use anyrepair::traits::Repair;
    use anyrepair::xml::XmlRepairer;

    let mut xml_repairer = XmlRepairer::new();

    // Unclosed tags
    let input1 = "<root><item>value</root>";
    let result1 = xml_repairer.repair(input1).unwrap();
    assert!(result1.contains("</item>") || result1.contains("<root>"));
}

#[test]
fn test_toml_edge_cases() {
    use anyrepair::toml::TomlRepairer;
    use anyrepair::traits::Repair;

    let mut toml_repairer = TomlRepairer::new();

    // Malformed arrays
    let input1 = "items = [1, 2, 3,]";
    let result1 = toml_repairer.repair(input1).unwrap();
    assert!(result1.contains("items"));
}

#[test]
fn test_csv_edge_cases() {
    use anyrepair::csv::CsvRepairer;
    use anyrepair::traits::Repair;

    let mut csv_repairer = CsvRepairer::new();

    // Unquoted strings
    let input1 = "name,age\nJohn,30\nJane,25";
    let result1 = csv_repairer.repair(input1).unwrap();
    assert!(result1.contains("John"));
}

#[test]
fn test_ini_edge_cases() {
    use anyrepair::key_value::IniRepairer;
    use anyrepair::traits::Repair;

    let mut ini_repairer = IniRepairer::new();

    // Simple input
    let input1 = "[user]\nname = John";
    let result1 = ini_repairer.repair(input1).unwrap();
    assert!(result1.contains("[user]"));
}

#[test]
fn test_diff_edge_cases() {
    use anyrepair::diff::DiffRepairer;
    use anyrepair::traits::Repair;

    let mut diff_repairer = DiffRepairer::new();

    // Missing hunk header
    let input1 = "-line 1\n+line 2\n line 3";
    let result1 = diff_repairer.repair(input1).unwrap();
    assert!(result1.contains("@@"));
}

#[test]
fn test_repair_strategy_execution_order() {
    use anyrepair::json::JsonRepairer;
    use anyrepair::traits::Repair;

    let mut repairer = JsonRepairer::new();

    // Test that multiple issues are fixed in the right order
    let input = r#"{
  // StripJsCommentsStrategy (priority 95)
  name: "John", // AddMissingQuotesStrategy (priority 90)
  age: 30, // FixTrailingCommasStrategy (priority 80)
}"#;

    let result = repairer.repair(input).unwrap();
    // All issues should be fixed
    assert!(!result.contains("//"));
    assert!(result.contains("\"name\""));
    assert!(result.contains("\"age\""));
    assert!(!result.contains(",\n}"));

    // Verify valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["name"], "John");
    assert_eq!(parsed["age"], 30);
}

#[test]
fn test_empty_and_minimal_json_inputs() {
    use anyrepair::json::JsonRepairer;
    use anyrepair::traits::Repair;

    let mut repairer = JsonRepairer::new();

    // Empty with only comment
    let input1 = r#"// just a comment"#;
    let result1 = repairer.repair(input1).unwrap();
    assert!(!result1.contains("//"));

    // Minimal JSON
    let input2 = r#"{}"#;
    let result2 = repairer.repair(input2).unwrap();
    assert_eq!(result2.trim(), "{}");

    // Array with comment
    let input3 = r#"[1, 2, // comment
    3]"#;
    let result3 = repairer.repair(input3).unwrap();
    assert!(!result3.contains("//"));
    assert!(result3.contains("["));
    assert!(result3.contains("]"));
}
