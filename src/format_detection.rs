//! Format detection heuristics
//!
//! This module contains all format-detection logic, separated from the
//! public API surface in `lib.rs` for better separation of concerns.

/// Detect the format of the given content, returns None if unknown
pub fn detect_format(content: &str) -> Option<&'static str> {
    let trimmed = content.trim();
    if is_json_like(trimmed) {
        Some("json")
    } else if is_diff_like(trimmed) {
        // Diff before yaml/csv/ini — diff lines contain colons, commas, etc.
        Some("diff")
    } else if is_yaml_like(trimmed) {
        Some("yaml")
    } else if is_xml_like(trimmed) {
        Some("xml")
    } else if is_toml_like(trimmed) {
        Some("toml")
    } else if is_csv_like(trimmed) {
        Some("csv")
    } else if is_env_like(trimmed) {
        Some("env")
    } else if is_properties_like(trimmed) {
        Some("properties")
    } else if is_ini_like(trimmed) {
        Some("ini")
    } else if is_markdown_like(trimmed) {
        Some("markdown")
    } else {
        None
    }
}

/// All `is_*_like` helpers expect **outer** whitespace already trimmed (as `detect_format` does).
fn is_json_like(trimmed: &str) -> bool {
    (trimmed.starts_with('{') && (trimmed.ends_with('}') || trimmed.contains(':')))
        || trimmed == "[]"
        || (trimmed.starts_with('[') && (trimmed.contains(',') || trimmed.contains('"') || trimmed.contains('\'')))
}

fn is_yaml_like(trimmed: &str) -> bool {
    if trimmed.contains("---") {
        return true;
    }
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return false;
    }
    trimmed.contains(":")
        || trimmed.lines().any(|line| {
            let line = line.trim();
            line.contains(":") && !line.starts_with('"') && !line.starts_with('{')
        })
}

fn is_xml_like(trimmed: &str) -> bool {
    trimmed.starts_with("<?xml")
        || (trimmed.starts_with('<') && trimmed.contains('>') && !trimmed.starts_with('#'))
        || (trimmed.contains('<') && trimmed.contains('>') && trimmed.contains("</"))
}

fn is_toml_like(trimmed: &str) -> bool {
    if trimmed.starts_with('{') || trimmed.starts_with('<') || trimmed.starts_with('#') {
        return false;
    }
    // TOML-specific: quoted values, inline tables, array of tables,
    // or arrays (but not bare [section] which is ambiguous with INI)
    trimmed.lines().any(|line| {
        let line = line.trim();
        line.contains('"')
            || line.contains('\'')
            || line.contains('{')
            || line.contains("[[")
            || (line.contains('[') && line.contains(']') && !line.starts_with('['))
    })
}

fn is_csv_like(trimmed: &str) -> bool {
    if !trimmed.contains(',') {
        return false;
    }
    if trimmed.starts_with('{')
        || trimmed.starts_with('[')
        || trimmed.starts_with('<')
        || trimmed.starts_with('#')
        || trimmed.starts_with("<?xml")
    {
        return false;
    }
    trimmed.lines().count() > 1
}

fn is_ini_like(trimmed: &str) -> bool {
    // INI requires section headers [section]
    trimmed.starts_with('[') && trimmed.contains(']')
        || trimmed.lines().any(|line| {
            let line = line.trim();
            line.starts_with('[') && line.contains(']') && !line.contains(',')
        })
}

fn is_env_like(trimmed: &str) -> bool {
    if !trimmed.contains('=') {
        return false;
    }
    if trimmed.starts_with('[')
        || trimmed.starts_with('{')
        || trimmed.starts_with('<')
        || trimmed.starts_with("<?xml")
        || trimmed.contains(':')
        || trimmed.contains(',')
    {
        return false;
    }
    // Majority of key-value lines have all-uppercase keys with underscores
    let kv_lines: Vec<&str> = trimmed
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter(|l| l.contains('='))
        .collect();
    if kv_lines.is_empty() {
        return false;
    }
    let uppercase_count = kv_lines
        .iter()
        .filter(|l| {
            let key = l.split('=').next().unwrap().trim();
            !key.is_empty()
                && key.chars().all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
        })
        .count();
    uppercase_count * 2 >= kv_lines.len()
}

fn is_properties_like(trimmed: &str) -> bool {
    if !trimmed.contains('=') {
        return false;
    }
    if trimmed.starts_with('[')
        || trimmed.starts_with('{')
        || trimmed.starts_with('<')
        || trimmed.starts_with("<?xml")
        || trimmed.contains(':')
        || trimmed.contains(',')
    {
        return false;
    }
    // Dot-separated keys or ! comments or generic key=value
    trimmed.lines().any(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return false;
        }
        if line.starts_with('!') {
            return true;
        }
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            !key.is_empty()
        } else {
            false
        }
    })
}

fn is_diff_like(trimmed: &str) -> bool {
    let lines: Vec<&str> = trimmed.lines().collect();

    // Check for hunk headers (@@ ... @@)
    if lines
        .iter()
        .any(|line| line.starts_with("@@") && line.matches("@@").count() >= 2)
    {
        return true;
    }

    // Check for paired file headers (--- a/... and +++ b/...)
    // Require both --- and +++ with a space after (not bare "---" which is YAML)
    let has_minus_header = lines
        .iter()
        .any(|line| line.starts_with("--- ") || line.starts_with("---\t"));
    let has_plus_header = lines
        .iter()
        .any(|line| line.starts_with("+++ ") || line.starts_with("+++\t"));
    if has_minus_header && has_plus_header {
        return true;
    }

    // Check for diff line prefixes (+, -, space) in multiple lines
    // Exclude bare "---"/"+++" (YAML separators) and "- item" (YAML lists)
    if lines.len() > 2 {
        let diff_line_count = lines
            .iter()
            .filter(|line| {
                let l = line.trim();
                if l == "---" || l == "+++" {
                    return false;
                }
                // YAML list items start with "- " followed by content
                if l.starts_with("- ")
                    && l.len() > 2
                    && l.chars().nth(2).is_some_and(|c| c.is_alphanumeric())
                {
                    return false;
                }
                l.starts_with('+')
                    || l.starts_with('-')
                    || (l.starts_with(' ') && !l.starts_with("@@"))
            })
            .count();
        if diff_line_count as f64 / lines.len() as f64 > 0.5 {
            return true;
        }
    }
    false
}

fn is_markdown_like(trimmed: &str) -> bool {
    if is_diff_like(trimmed) {
        return false;
    }
    trimmed.starts_with('#')
        || trimmed.contains("```")
        || trimmed.contains("**")
        || trimmed.contains("*")
        || trimmed.contains("`")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(detect_format(r#"{"key": "value"}"#), Some("json"));
        assert_eq!(detect_format(r#"[1, 2, 3]"#), Some("json"));
        assert_eq!(detect_format("key: value"), Some("yaml"));
        assert_eq!(detect_format("---\nkey: value"), Some("yaml"));
        assert_eq!(
            detect_format("<?xml version=\"1.0\"?><root></root>"),
            Some("xml")
        );
        assert_eq!(
            detect_format("<root><item>value</item></root>"),
            Some("xml")
        );
        assert_eq!(detect_format("name,age\nJohn,30"), Some("csv"));
        assert_eq!(detect_format("# Header\n**bold**"), Some("markdown"));
    }

    #[test]
    fn test_format_detection_edge_cases() {
        assert_eq!(detect_format(""), None);
        assert_eq!(detect_format("   \n\t  "), None);
        assert_eq!(
            detect_format(r#"  {"key": "value"}  "#),
            Some("json")
        );
        assert_eq!(
            detect_format(r#"{"key": "value", "nested": {"inner": "value"}}"#),
            Some("json")
        );
        assert_eq!(
            detect_format("key: value\nnested:\n  inner: value"),
            Some("yaml")
        );
    }

    #[test]
    fn test_diff_detection() {
        assert_eq!(
            detect_format("@@ -1,3 +1,4 @@\n-line 1\n+line 2"),
            Some("diff")
        );
        assert_eq!(
            detect_format("--- a/file\n+++ b/file\n@@ -1 +1 @@"),
            Some("diff")
        );
    }

    #[test]
    fn test_is_json_like() {
        assert!(is_json_like(r#"{"key": "value"}"#));
        assert!(is_json_like(r#"[1, 2, 3]"#));
        assert!(!is_json_like("# Header\nContent"));
    }

    #[test]
    fn test_is_yaml_like() {
        assert!(is_yaml_like("key: value"));
        assert!(is_yaml_like("---\nkey: value"));
        assert!(!is_yaml_like(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_is_xml_like() {
        assert!(is_xml_like("<?xml version=\"1.0\"?><root></root>"));
        assert!(is_xml_like("<root><item>value</item></root>"));
        assert!(!is_xml_like(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_is_toml_like() {
        assert!(is_toml_like("[user]\nname = \"John\""));
        assert!(is_toml_like("name = 'John'"));
        assert!(is_toml_like("key = {a=1, b=2}"));
        assert!(!is_toml_like("name = John")); // plain key=value = properties
        assert!(!is_toml_like(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_is_csv_like() {
        assert!(is_csv_like("name,age\nJohn,30"));
        assert!(is_csv_like("John,30,Engineer\nJane,25,Designer"));
        assert!(!is_csv_like(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_is_ini_like() {
        assert!(is_ini_like("[user]\nname = John"));
        assert!(is_ini_like("name = John\n[section]\nage = 30"));
        assert!(!is_ini_like("name = John\nage = 30")); // no sections = properties
        assert!(!is_ini_like(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_is_env_like() {
        assert!(is_env_like("DATABASE_URL=host\nAPI_KEY=secret"));
        assert!(is_env_like("PATH=/usr/bin\nHOME=/home/user"));
        assert!(!is_env_like("key=value\nother=val")); // mixed case = properties
        assert!(!is_env_like("[section]\nkey=value")); // sections = ini
        assert!(!is_env_like("name,age\nJohn,30")); // csv
    }

    #[test]
    fn test_is_properties_like() {
        assert!(is_properties_like("app.name=value\napp.version=1.0"));
        assert!(is_properties_like("key=value\nother=val"));
        assert!(is_properties_like("!comment\nkey=value"));
        assert!(!is_properties_like("[section]\nkey=value")); // sections = ini
        assert!(!is_properties_like("name,age\nJohn,30")); // csv
    }

    #[test]
    fn test_is_markdown_like() {
        assert!(is_markdown_like("# Header"));
        assert!(is_markdown_like("**bold**"));
        assert!(is_markdown_like("```code```"));
        assert!(!is_markdown_like(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_properties_env_detection() {
        assert_eq!(detect_format("DATABASE_URL=host\nAPI_KEY=secret"), Some("env"));
        assert_eq!(detect_format("app.name=value\napp.version=1.0"), Some("properties"));
        assert_eq!(detect_format("key=value\nother=val"), Some("properties"));
        // INI with sections still works
        assert_eq!(detect_format("[section]\nkey=value"), Some("ini"));
        assert_eq!(detect_format("[default]\nkey=value"), Some("ini"));
    }

    #[test]
    fn test_single_line_csv_not_detected() {
        // CSV requires multiple lines
        assert_eq!(detect_format("a,b,c"), None);
        assert_eq!(detect_format("a,b\nc,d"), Some("csv"));
    }

    #[test]
    fn test_yaml_separator_vs_diff() {
        // Bare "---" is YAML, "--- a/file" is diff
        assert_eq!(detect_format("---\nkey: value"), Some("yaml"));
        assert_eq!(detect_format("--- a/file\n+++ b/file"), Some("diff"));
    }

    #[test]
    fn test_ambiguous_inputs() {
        // Plain text with no structure
        assert_eq!(detect_format("just plain text"), None);
        // Single word
        assert_eq!(detect_format("hello"), None);
        // Just whitespace
        assert_eq!(detect_format("   \t\n  "), None);
    }

    #[test]
    fn test_json_edge_cases() {
        // JSON fragment without closing bracket
        assert_eq!(detect_format(r#"{"key": "value""#), Some("json"));
        // Array fragment
        assert_eq!(detect_format("[1, 2, 3"), Some("json"));
    }

    #[test]
    fn test_toml_edge_cases() {
        // TOML key-value
        assert_eq!(detect_format("key = 'value'"), Some("toml"));
        // Not TOML: starts with {
        assert_eq!(detect_format(r#"{"key": "value"}"#), Some("json"));
        // Not TOML: starts with <
        assert_eq!(detect_format("<html></html>"), Some("xml"));
    }

    #[test]
    fn test_ini_edge_cases() {
        // [section] is ambiguous: no quotes/arrays, so detected as INI
        assert_eq!(detect_format("[section]"), Some("ini"));
        // Has comma: rejected by all key-value detectors, returns None
        assert_eq!(detect_format("key=value,other"), None);
        // Not INI: starts with {
        assert_eq!(detect_format(r#"{"key": "value"}"#), Some("json"));
    }

    #[test]
    fn test_xml_self_closing() {
        assert_eq!(detect_format("<root><item/></root>"), Some("xml"));
        assert_eq!(detect_format("<br/>"), Some("xml"));
    }

    #[test]
    fn test_diff_yaml_list_distinguish() {
        // YAML list items without colons: not enough structure to identify format
        assert_eq!(detect_format("- item1\n- item2\n- item3"), None);
        // Diff with removed lines
        assert_eq!(
            detect_format("--- a/file\n+++ b/file\n-removed\n+added"),
            Some("diff")
        );
    }

    #[test]
    fn test_markdown_vs_yaml_list() {
        // Single asterisk line: could be markdown emphasis or YAML-like
        assert_eq!(detect_format("* item"), Some("markdown"));
        // Multiple asterisks: markdown
        assert_eq!(detect_format("**bold** and *italic*"), Some("markdown"));
    }
}
