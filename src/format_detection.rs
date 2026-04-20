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
        || (trimmed.starts_with('[') && (trimmed.ends_with(']') || trimmed.contains(',')))
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
    if trimmed.starts_with('[') {
        return true;
    }
    if trimmed.starts_with('{') || trimmed.starts_with('<') || trimmed.starts_with('#') {
        return false;
    }
    trimmed.contains('=')
        || trimmed.lines().any(|line| {
            let line = line.trim();
            line.starts_with('[') && line.ends_with(']')
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
    if trimmed.starts_with('[') && trimmed.contains(']') {
        return true;
    }
    if trimmed.starts_with('{')
        || trimmed.starts_with('<')
        || trimmed.starts_with('#')
        || trimmed.starts_with("<?xml")
        || trimmed.contains(',')
        || trimmed.contains(':')
    {
        return false;
    }
    trimmed.contains('=')
        || trimmed.lines().any(|line| {
            let line = line.trim();
            line.starts_with('[') && line.contains(']') && !line.contains(',')
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
        assert!(is_toml_like("name = John"));
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
        assert!(is_ini_like("name = John\nage = 30"));
        assert!(!is_ini_like(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_is_markdown_like() {
        assert!(is_markdown_like("# Header"));
        assert!(is_markdown_like("**bold**"));
        assert!(is_markdown_like("```code```"));
        assert!(!is_markdown_like(r#"{"key": "value"}"#));
    }
}
