//! Advanced confidence scoring algorithms for repair operations

use serde::{Deserialize, Serialize};

/// Confidence score components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceComponents {
    /// Structure validity score (0.0-1.0)
    pub structure_score: f64,
    /// Content completeness score (0.0-1.0)
    pub completeness_score: f64,
    /// Syntax correctness score (0.0-1.0)
    pub syntax_score: f64,
    /// Format-specific score (0.0-1.0)
    pub format_score: f64,
    /// Repair strategy effectiveness (0.0-1.0)
    pub repair_effectiveness: f64,
}

/// Advanced confidence scorer
pub struct ConfidenceScorer;

impl ConfidenceScorer {
    /// Calculate comprehensive confidence score for JSON
    pub fn score_json(content: &str) -> f64 {
        let mut components = ConfidenceComponents {
            structure_score: 0.0,
            completeness_score: 0.0,
            syntax_score: 0.0,
            format_score: 0.0,
            repair_effectiveness: 0.0,
        };

        // Structure validation
        let open_braces = content.matches('{').count();
        let close_braces = content.matches('}').count();
        let open_brackets = content.matches('[').count();
        let close_brackets = content.matches(']').count();

        if open_braces == close_braces && open_brackets == close_brackets {
            components.structure_score = 0.4;
        } else if (open_braces as i32 - close_braces as i32).abs() <= 1 {
            components.structure_score = 0.2;
        }

        // Completeness check
        let quote_count = content.matches('"').count();
        if quote_count % 2 == 0 && quote_count > 0 {
            components.completeness_score = 0.3;
        } else if quote_count > 0 {
            components.completeness_score = 0.15;
        }

        // Syntax validation
        if serde_json::from_str::<serde_json::Value>(content).is_ok() {
            components.syntax_score = 1.0;
        } else {
            // Partial syntax check
            let has_colons = content.contains(':');
            let has_commas = content.contains(',');
            if has_colons && has_commas {
                components.syntax_score = 0.4;
            } else if has_colons || has_commas {
                components.syntax_score = 0.2;
            }
        }

        // Format-specific checks
        if content.trim().starts_with('{') || content.trim().starts_with('[') {
            components.format_score = 0.2;
        }

        // Calculate weighted average
        let weights = (0.25, 0.20, 0.35, 0.15, 0.05);
        let score = (components.structure_score * weights.0)
            + (components.completeness_score * weights.1)
            + (components.syntax_score * weights.2)
            + (components.format_score * weights.3)
            + (components.repair_effectiveness * weights.4);

        if score > 1.0 { 1.0 } else { score }
    }

    /// Calculate comprehensive confidence score for YAML
    pub fn score_yaml(content: &str) -> f64 {
        let mut score = 0.0;

        // Check for YAML markers
        if content.contains("---") || content.contains("...") {
            score += 0.2;
        }

        // Check for key-value pairs
        if content.contains(':') {
            score += 0.3;
        }

        // Check for proper indentation
        let lines: Vec<&str> = content.lines().collect();
        let mut indent_consistent = true;
        let mut prev_indent = 0;

        for line in &lines {
            if line.trim().is_empty() {
                continue;
            }
            let indent = line.len() - line.trim_start().len();
            if indent > 0 && prev_indent > 0 {
                if indent % 2 != 0 && indent % 4 != 0 {
                    indent_consistent = false;
                    break;
                }
            }
            prev_indent = indent;
        }

        if indent_consistent {
            score += 0.25;
        }

        // Check for valid YAML structure
        if let Ok(_) = serde_yaml::from_str::<serde_yaml::Value>(content) {
            score = 1.0;
        } else {
            // Partial validity
            if content.lines().count() > 1 && content.contains(':') {
                score += 0.15;
            }
        }

        if score > 1.0 { 1.0 } else { score }
    }

    /// Calculate comprehensive confidence score for XML
    pub fn score_xml(content: &str) -> f64 {
        let mut score = 0.0;

        // Check for XML declaration
        if content.trim().starts_with("<?xml") {
            score += 0.2;
        }

        // Check for opening and closing tags
        let open_tags = content.matches('<').count();
        let close_tags = content.matches('>').count();

        if open_tags == close_tags && open_tags > 0 {
            score += 0.3;
        } else if (open_tags as i32 - close_tags as i32).abs() <= 1 {
            score += 0.15;
        }

        // Check for proper nesting
        let mut tag_stack = Vec::new();
        let mut valid_nesting = true;

        for part in content.split('<') {
            if let Some(tag_name) = part.split('>').next() {
                if tag_name.starts_with('/') {
                    if tag_stack.is_empty() {
                        valid_nesting = false;
                        break;
                    }
                    tag_stack.pop();
                } else if !tag_name.is_empty() && !tag_name.starts_with('?') && !tag_name.starts_with('!') {
                    tag_stack.push(tag_name);
                }
            }
        }

        if valid_nesting && tag_stack.is_empty() {
            score += 0.35;
        } else if valid_nesting {
            score += 0.15;
        }

        // XML is generally valid if it has proper structure
        if tag_stack.is_empty() && valid_nesting {
            score = 1.0;
        }

        if score > 1.0 { 1.0 } else { score }
    }

    /// Calculate comprehensive confidence score for Markdown
    pub fn score_markdown(content: &str) -> f64 {
        let mut score = 0.0;

        // Check for markdown markers
        if content.contains('#') {
            score += 0.15;
        }
        if content.contains("**") || content.contains("__") {
            score += 0.15;
        }
        if content.contains('*') || content.contains('-') {
            score += 0.1;
        }
        if content.contains('[') && content.contains(']') {
            score += 0.1;
        }

        // Check for code blocks
        if content.contains("```") {
            let backtick_count = content.matches("```").count();
            if backtick_count % 2 == 0 {
                score += 0.25;
            } else {
                score += 0.1;
            }
        }

        // Check for proper structure
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() > 1 {
            score += 0.1;
        }

        // Markdown is generally valid if it has content
        if !content.trim().is_empty() {
            score += 0.05;
        }

        if score > 1.0 { 1.0 } else { score }
    }

    /// Calculate comprehensive confidence score for CSV
    pub fn score_csv(content: &str) -> f64 {
        let mut score = 0.0;

        // Check for comma separation
        if content.contains(',') {
            score += 0.3;
        }

        // Check for multiple lines
        let line_count = content.lines().count();
        if line_count > 1 {
            score += 0.2;
        }

        // Check for consistent column count
        let lines: Vec<&str> = content.lines().collect();
        if !lines.is_empty() {
            let first_line_cols = lines[0].split(',').count();
            let mut consistent = true;

            for line in &lines[1..] {
                if !line.trim().is_empty() {
                    let cols = line.split(',').count();
                    if cols != first_line_cols {
                        consistent = false;
                        break;
                    }
                }
            }

            if consistent {
                score += 0.35;
            } else {
                score += 0.1;
            }
        }

        // Check for quoted fields
        if content.contains('"') {
            score += 0.05;
        }

        if score > 1.0 { 1.0 } else { score }
    }

    /// Calculate comprehensive confidence score for TOML
    pub fn score_toml(content: &str) -> f64 {
        let mut score = 0.0;

        // Check for section headers
        if content.contains('[') && content.contains(']') {
            score += 0.25;
        }

        // Check for key-value pairs
        if content.contains('=') {
            score += 0.25;
        }

        // Check for proper structure
        let lines: Vec<&str> = content.lines().collect();
        let mut valid_structure = true;

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                continue;
            }

            if trimmed.contains('=') {
                continue;
            }

            valid_structure = false;
            break;
        }

        if valid_structure {
            score += 0.35;
        } else {
            score += 0.1;
        }

        // Try to parse as TOML
        if toml::from_str::<toml::Value>(content).is_ok() {
            score = 1.0;
        }

        if score > 1.0 { 1.0 } else { score }
    }

    /// Calculate confidence score based on format
    pub fn score(content: &str, format: &str) -> f64 {
        match format.to_lowercase().as_str() {
            "json" => Self::score_json(content),
            "yaml" | "yml" => Self::score_yaml(content),
            "xml" => Self::score_xml(content),
            "markdown" | "md" => Self::score_markdown(content),
            "csv" => Self::score_csv(content),
            "toml" => Self::score_toml(content),
            _ => 0.5, // Default for unknown formats
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_valid_json() {
        let json = r#"{"key": "value"}"#;
        let score = ConfidenceScorer::score_json(json);
        assert!(score > 0.5);
    }

    #[test]
    fn test_score_invalid_json() {
        let json = r#"{"key": "value"#;
        let score = ConfidenceScorer::score_json(json);
        assert!(score < 0.8);
    }

    #[test]
    fn test_score_valid_yaml() {
        let yaml = "key: value\nkey2: value2";
        let score = ConfidenceScorer::score_yaml(yaml);
        assert!(score > 0.5);
    }

    #[test]
    fn test_score_valid_xml() {
        let xml = "<root><item>value</item></root>";
        let score = ConfidenceScorer::score_xml(xml);
        assert!(score > 0.5);
    }

    #[test]
    fn test_score_markdown() {
        let md = "# Header\n**bold** text";
        let score = ConfidenceScorer::score_markdown(md);
        assert!(score > 0.3);
    }

    #[test]
    fn test_score_csv() {
        let csv = "name,age\nJohn,30\nJane,25";
        let score = ConfidenceScorer::score_csv(csv);
        assert!(score > 0.5);
    }

    #[test]
    fn test_score_toml() {
        let toml = "[section]\nkey = \"value\"";
        let score = ConfidenceScorer::score_toml(toml);
        assert!(score > 0.5);
    }
}
