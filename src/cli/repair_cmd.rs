//! Repair command handler

use std::io::{self, IsTerminal};

/// ANSI color codes for terminal output.
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// Determine whether color output should be used based on the --color flag.
fn should_use_color(color: &str) -> bool {
    match color {
        "always" => true,
        "never" => false,
        _ => std::io::stdout().is_terminal(),
    }
}

/// Unified repair handler for all formats.
/// When format is Some, uses that format directly via the registry.
/// When format is None, uses auto-detection.
pub fn handle_repair(
    input: Option<&str>,
    output: Option<&str>,
    show_confidence: bool,
    verbose: bool,
    format: Option<&str>,
    show_diff: bool,
    dry_run: bool,
    json_output: bool,
    min_confidence: Option<f64>,
    explain: bool,
    color: &str,
) -> io::Result<()> {
    let content = super::read_input(input)?;

    let (repaired, confidence, detected_format, explanations) = if let Some(fmt) = format {
        if verbose {
            eprintln!("Repairing content as {}...", fmt);
        }
        if explain {
            let (r, names) = anyrepair::repair_with_explanations(&content, fmt)
                .map_err(|e| io::Error::other(e.to_string()))?;
            let c = anyrepair::create_validator(fmt)
                .map_err(|e| io::Error::other(e.to_string()))?
                .is_valid(&r);
            let conf = if c { 1.0 } else { 0.0 };
            (r, conf, fmt, names)
        } else {
            let (r, c) = repair_format(&content, fmt)?;
            (r, c, fmt, Vec::new())
        }
    } else {
        if verbose {
            eprintln!("Repairing content (auto-detect format)...");
        }
        let detected = anyrepair::detect_format(&content);
        if verbose
            && let Some(fmt) = detected {
                eprintln!("Detected format: {}", fmt);
            }
        match detected {
            Some(fmt) => {
                if explain {
                    let (r, names) = anyrepair::repair_with_explanations(&content, fmt)
                        .map_err(|e| io::Error::other(e.to_string()))?;
                    let c = anyrepair::create_validator(fmt)
                        .map_err(|e| io::Error::other(e.to_string()))?
                        .is_valid(&r);
                    let conf = if c { 1.0 } else { 0.0 };
                    (r, conf, fmt, names)
                } else {
                    let (r, c) = repair_format(&content, fmt)?;
                    (r, c, fmt, Vec::new())
                }
            }
            None => {
                let repaired = anyrepair::repair(&content)
                    .map_err(|e| io::Error::other(e.to_string()))?;
                (repaired, 0.0, "markdown", Vec::new())
            }
        }
    };

    if verbose {
        eprintln!("Repair completed");
    }

    if let Some(threshold) = min_confidence {
        if confidence < threshold {
            eprintln!(
                "Confidence {:.2}% is below threshold {:.2}%",
                confidence * 100.0,
                threshold * 100.0
            );
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Confidence {:.2}% below minimum threshold {:.2}%",
                    confidence * 100.0,
                    threshold * 100.0
                ),
            ));
        }
    }

    let had_changes = content != repaired;

    let use_color = should_use_color(color);

    if explain {
        if explanations.is_empty() {
            eprintln!("No strategies were applied (content was already valid or no changes needed).");
        } else {
            eprintln!("Applied repair strategies:");
            for name in &explanations {
                if use_color {
                    eprintln!("  - {CYAN}{name}{RESET}");
                } else {
                    eprintln!("  - {}", name);
                }
            }
        }
    }

    if json_output {
        let json = build_json_result(
            detected_format,
            confidence,
            had_changes,
            content.len(),
            repaired.len(),
            output,
            &repaired,
        );
        println!("{}", json);
        if !dry_run {
            super::write_output(&repaired, output)?;
        }
        return Ok(());
    }

    if show_confidence {
        println!("Confidence: {:.2}%", confidence * 100.0);
    }

    if show_diff {
        let diff = generate_diff(&content, &repaired, use_color);
        if diff.is_empty() {
            eprintln!("No changes needed.");
        } else {
            print!("{}", diff);
        }
    }

    if dry_run {
        if !show_diff && !explain {
            eprintln!("Dry run — no output written.");
        }
        return Ok(());
    }

    super::write_output(&repaired, output)
}

/// Repair content with a specific format, returning (repaired, confidence)
fn repair_format(content: &str, format: &str) -> io::Result<(String, f64)> {
    let mut repairer = anyrepair::create_repairer(format)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
    let repaired = repairer.repair(content)
        .map_err(|e| io::Error::other(e.to_string()))?;
    let confidence = repairer.confidence(&repaired);
    Ok((repaired, confidence))
}

/// Build a machine-readable JSON result string for CI usage.
fn build_json_result(
    format: &str,
    confidence: f64,
    repaired_needed: bool,
    original_len: usize,
    repaired_len: usize,
    output: Option<&str>,
    repaired_content: &str,
) -> String {
    let output_field = match output {
        Some(p) => anyrepair::json_util::json_string(p),
        None => "null".to_string(),
    };
    let content_field = if output.is_none() {
        format!(r#","content":{}"#, anyrepair::json_util::json_string(repaired_content))
    } else {
        String::new()
    };

    format!(
        r#"{{"format":{},"confidence":{},"repaired":{},"original_length":{},"repaired_length":{},"output":{}{}}}"#,
        anyrepair::json_util::json_string(format),
        confidence,
        repaired_needed,
        original_len,
        repaired_len,
        output_field,
        content_field,
    )
}

/// Generate a simple unified diff between original and repaired content.
fn generate_diff(original: &str, repaired: &str, use_color: bool) -> String {
    let orig_lines: Vec<&str> = original.lines().collect();
    let new_lines: Vec<&str> = repaired.lines().collect();

    if orig_lines == new_lines {
        return String::new();
    }

    let mut result = String::new();
    if use_color {
        result.push_str(&format!("{BOLD}--- original{RESET}\n{BOLD}+++ repaired{RESET}\n"));
    } else {
        result.push_str("--- original\n+++ repaired\n");
    }

    let max_len = orig_lines.len().max(new_lines.len());
    for i in 0..max_len {
        let orig = orig_lines.get(i).copied().unwrap_or("");
        let new = new_lines.get(i).copied().unwrap_or("");
        if orig != new {
            if i < orig_lines.len() {
                if use_color {
                    result.push_str(&format!("{RED}-{orig}{RESET}\n"));
                } else {
                    result.push_str(&format!("-{}\n", orig));
                }
            }
            if i < new_lines.len() {
                if use_color {
                    result.push_str(&format!("{GREEN}+{new}{RESET}\n"));
                } else {
                    result.push_str(&format!("+{}\n", new));
                }
            }
        } else if i < orig_lines.len() {
            result.push_str(&format!(" {}\n", orig));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_diff_no_changes() {
        let content = "line1\nline2\nline3";
        let diff = generate_diff(content, content, false);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_generate_diff_with_changes() {
        let original = "line1\nline2\nline3";
        let repaired = "line1\nmodified\nline3";
        let diff = generate_diff(original, repaired, false);
        assert!(diff.contains("--- original"));
        assert!(diff.contains("+++ repaired"));
        assert!(diff.contains("-line2"));
        assert!(diff.contains("+modified"));
        assert!(diff.contains(" line1"));
        assert!(diff.contains(" line3"));
    }

    #[test]
    fn test_generate_diff_added_line() {
        let original = "a\nb";
        let repaired = "a\nb\nc";
        let diff = generate_diff(original, repaired, false);
        assert!(diff.contains("+c"));
    }

    #[test]
    fn test_generate_diff_removed_line() {
        let original = "a\nb\nc";
        let repaired = "a\nc";
        let diff = generate_diff(original, repaired, false);
        assert!(diff.contains("-b"));
    }

    #[test]
    fn test_generate_diff_empty_original() {
        let diff = generate_diff("", "new line", false);
        assert!(diff.contains("+new line"));
    }

    #[test]
    fn test_generate_diff_colored() {
        let original = "line1\nline2";
        let repaired = "line1\nmodified";
        let diff = generate_diff(original, repaired, true);
        assert!(diff.contains("\x1b[31m")); // red
        assert!(diff.contains("\x1b[32m")); // green
        assert!(diff.contains("\x1b[0m"));  // reset
    }

    #[test]
    fn test_should_use_color() {
        assert!(should_use_color("always"));
        assert!(!should_use_color("never"));
    }

    #[test]
    fn test_dry_run_does_not_write_output() {
        let mut tmp = std::env::temp_dir();
        tmp.push("anyrepair_dryrun_unit.json");
        std::fs::write(&tmp, r#"{"key": "value",}"#).unwrap();
        let path = tmp.to_str().unwrap();

        let out = std::env::temp_dir().join("anyrepair_dryrun_unit_out.json");
        let out_path = out.to_str().unwrap();

        let result = handle_repair(
            Some(path),
            Some(out_path),
            false,
            false,
            Some("json"),
            false,
            true,
            false,
            None,
            false,
            "never",
        );
        assert!(result.is_ok());
        assert!(!out.exists(), "dry_run should not write output file");
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn test_diff_with_dry_run_shows_changes() {
        let mut tmp = std::env::temp_dir();
        tmp.push("anyrepair_diff_unit.json");
        std::fs::write(&tmp, r#"{"key": "value",}"#).unwrap();
        let path = tmp.to_str().unwrap();

        let result = handle_repair(
            Some(path),
            None,
            false,
            false,
            Some("json"),
            true,
            true,
            false,
            None,
            false,
            "never",
        );
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_diff_no_changes() {
        let mut tmp = std::env::temp_dir();
        tmp.push("anyrepair_nodiff_unit.json");
        std::fs::write(&tmp, r#"{"key": "value"}"#).unwrap();
        let path = tmp.to_str().unwrap();

        let result = handle_repair(
            Some(path),
            None,
            false,
            false,
            Some("json"),
            true,
            true,
            false,
            None,
            false,
            "never",
        );
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_json_output_mode() {
        let mut tmp = std::env::temp_dir();
        tmp.push("anyrepair_json_mode.json");
        std::fs::write(&tmp, r#"{"key": "value",}"#).unwrap();
        let path = tmp.to_str().unwrap();

        let result = handle_repair(
            Some(path),
            None,
            false,
            false,
            Some("json"),
            false,
            true,
            true,
            None,
            false,
            "never",
        );
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_min_confidence_passes() {
        let mut tmp = std::env::temp_dir();
        tmp.push("anyrepair_minconf_pass.json");
        std::fs::write(&tmp, r#"{"key": "value"}"#).unwrap();
        let path = tmp.to_str().unwrap();

        let result = handle_repair(
            Some(path),
            None,
            false,
            false,
            Some("json"),
            false,
            true,
            false,
            Some(0.0),
            false,
            "never",
        );
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_min_confidence_fails() {
        let mut tmp = std::env::temp_dir();
        tmp.push("anyrepair_minconf_fail.json");
        std::fs::write(&tmp, r#"{"key": "value"}"#).unwrap();
        let path = tmp.to_str().unwrap();

        let result = handle_repair(
            Some(path),
            None,
            false,
            false,
            Some("json"),
            false,
            true,
            false,
            Some(2.0),
            false,
            "never",
        );
        assert!(result.is_err());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_build_json_result_with_output() {
        let json = build_json_result("json", 0.95, true, 20, 18, Some("out.json"), "{}");
        assert!(json.contains(r#""format":"json""#));
        assert!(json.contains(r#""confidence":0.95"#));
        assert!(json.contains(r#""repaired":true"#));
        assert!(json.contains(r#""original_length":20"#));
        assert!(json.contains(r#""repaired_length":18"#));
        assert!(json.contains(r#""output":"out.json""#));
        assert!(!json.contains(r#""content""#));
    }

    #[test]
    fn test_build_json_result_without_output() {
        let json = build_json_result("yaml", 0.8, false, 10, 10, None, "key: val");
        assert!(json.contains(r#""format":"yaml""#));
        assert!(json.contains(r#""repaired":false"#));
        assert!(json.contains(r#""output":null"#));
        assert!(json.contains(r#""content":"key: val""#));
    }

    #[test]
    fn test_explain_with_changes() {
        let mut tmp = std::env::temp_dir();
        tmp.push("anyrepair_explain_changes.json");
        std::fs::write(&tmp, r#"{"key": "value",}"#).unwrap();
        let path = tmp.to_str().unwrap();

        let result = handle_repair(
            Some(path),
            None,
            false,
            false,
            Some("json"),
            false,
            true,
            false,
            None,
            true,
            "never",
        );
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_explain_no_changes() {
        let mut tmp = std::env::temp_dir();
        tmp.push("anyrepair_explain_nochange.json");
        std::fs::write(&tmp, r#"{"key": "value"}"#).unwrap();
        let path = tmp.to_str().unwrap();

        let result = handle_repair(
            Some(path),
            None,
            false,
            false,
            Some("json"),
            false,
            true,
            false,
            None,
            true,
            "never",
        );
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_repair_with_explanations_json() {
        let (repaired, names) = anyrepair::repair_with_explanations(
            r#"{"key": "value",}"#,
            "json",
        )
        .unwrap();
        assert!(!repaired.ends_with(','));
        assert!(!names.is_empty(), "at least one strategy should have been applied");
    }

    #[test]
    fn test_repair_with_explanations_already_valid() {
        let (_, names) =
            anyrepair::repair_with_explanations(r#"{"key": "value"}"#, "json").unwrap();
        assert!(names.is_empty(), "no strategies should be applied to valid content");
    }
}
