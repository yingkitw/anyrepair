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

/// Options for the repair subcommand, grouped to keep `handle_repair` signature small.
pub struct RepairOptions<'a> {
    pub input: Option<&'a str>,
    pub output: Option<&'a str>,
    pub format: Option<&'a str>,
    pub show_confidence: bool,
    pub show_diff: bool,
    pub dry_run: bool,
    pub json_output: bool,
    pub min_confidence: Option<f64>,
    pub explain: bool,
    pub color: &'a str,
}

/// Fields needed to build the machine-readable JSON result, grouped to keep
/// `build_json_result` signature small.
struct JsonResultInput<'a> {
    format: &'a str,
    confidence: f64,
    detection_confidence: f64,
    repaired_needed: bool,
    original_len: usize,
    repaired_len: usize,
    output: Option<&'a str>,
    repaired_content: &'a str,
}

/// Unified repair handler for all formats.
/// When `opts.format` is Some, uses that format directly via the registry.
/// When `opts.format` is None, uses auto-detection.
pub fn handle_repair(opts: &RepairOptions<'_>, verbose: bool) -> io::Result<()> {
    let content = super::read_input(opts.input)?;
    let format = opts.format;

    let (repaired, confidence, detected_format, explanations, detection_confidence) = if let Some(fmt) = format {
        if verbose {
            eprintln!("Repairing content as {}...", fmt);
        }
        if opts.explain {
            let (r, names) = anyrepair::repair_with_explanations(&content, fmt)
                .map_err(|e| io::Error::other(e.to_string()))?;
            let c = anyrepair::create_validator(fmt)
                .map_err(|e| io::Error::other(e.to_string()))?
                .is_valid(&r);
            let conf = if c { 1.0 } else { 0.0 };
            (r, conf, fmt, names, 1.0)
        } else {
            let (r, c) = repair_format(&content, fmt)?;
            (r, c, fmt, Vec::new(), 1.0)
        }
    } else {
        if verbose {
            eprintln!("Repairing content (auto-detect format)...");
        }
        let detected = anyrepair::detect_format_with_confidence(&content);
        if verbose
            && let Some(ref dr) = detected {
                eprintln!("Detected format: {} (confidence: {:.0}%)", dr.format, dr.confidence * 100.0);
            }
        match detected {
            Some(dr) => {
                let fmt = dr.format;
                let det_conf = dr.confidence;
                if opts.explain {
                    let (r, names) = anyrepair::repair_with_explanations(&content, fmt)
                        .map_err(|e| io::Error::other(e.to_string()))?;
                    let c = anyrepair::create_validator(fmt)
                        .map_err(|e| io::Error::other(e.to_string()))?
                        .is_valid(&r);
                    let conf = if c { 1.0 } else { 0.0 };
                    (r, conf, fmt, names, det_conf)
                } else {
                    let (r, c) = repair_format(&content, fmt)?;
                    (r, c, fmt, Vec::new(), det_conf)
                }
            }
            None => {
                let repaired = anyrepair::repair(&content)
                    .map_err(|e| io::Error::other(e.to_string()))?;
                (repaired, 0.0, "markdown", Vec::new(), 0.0)
            }
        }
    };

    if verbose {
        eprintln!("Repair completed");
    }

    if let Some(threshold) = opts.min_confidence
        && confidence < threshold {
            eprintln!(
                "Confidence {:.2}% is below threshold {:.2}%",
                confidence * 100.0,
                threshold * 100.0
            );
            return Err(io::Error::other(
                format!(
                    "Confidence {:.2}% below minimum threshold {:.2}%",
                    confidence * 100.0,
                    threshold * 100.0
                ),
            ));
        }

    let had_changes = content != repaired;

    let use_color = should_use_color(opts.color);

    if opts.explain {
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

    if opts.json_output {
        let json = build_json_result(&JsonResultInput {
            format: detected_format,
            confidence,
            detection_confidence,
            repaired_needed: had_changes,
            original_len: content.len(),
            repaired_len: repaired.len(),
            output: opts.output,
            repaired_content: &repaired,
        });
        println!("{}", json);
        if !opts.dry_run {
            super::write_output(&repaired, opts.output)?;
        }
        return Ok(());
    }

    if opts.show_confidence {
        println!("Confidence: {:.2}%", confidence * 100.0);
    }

    if opts.show_diff {
        let diff = generate_diff(&content, &repaired, use_color);
        if diff.is_empty() {
            eprintln!("No changes needed.");
        } else {
            print!("{}", diff);
        }
    }

    if opts.dry_run {
        if !opts.show_diff && !opts.explain {
            eprintln!("Dry run — no output written.");
        }
        return Ok(());
    }

    super::write_output(&repaired, opts.output)
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
fn build_json_result(d: &JsonResultInput<'_>) -> String {
    let output_field = match d.output {
        Some(p) => anyrepair::json_util::json_string(p),
        None => "null".to_string(),
    };
    let content_field = if d.output.is_none() {
        format!(r#","content":{}"#, anyrepair::json_util::json_string(d.repaired_content))
    } else {
        String::new()
    };

    format!(
        r#"{{"format":{},"confidence":{},"detection_confidence":{},"repaired":{},"original_length":{},"repaired_length":{},"output":{}{}}}"#,
        anyrepair::json_util::json_string(d.format),
        d.confidence,
        d.detection_confidence,
        d.repaired_needed,
        d.original_len,
        d.repaired_len,
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
            &RepairOptions {
                input: Some(path),
                output: Some(out_path),
                format: Some("json"),
                show_confidence: false,
                show_diff: false,
                dry_run: true,
                json_output: false,
                min_confidence: None,
                explain: false,
                color: "never",
            },
            false,
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
            &RepairOptions {
                input: Some(path),
                output: None,
                format: Some("json"),
                show_confidence: false,
                show_diff: true,
                dry_run: true,
                json_output: false,
                min_confidence: None,
                explain: false,
                color: "never",
            },
            false,
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
            &RepairOptions {
                input: Some(path),
                output: None,
                format: Some("json"),
                show_confidence: false,
                show_diff: true,
                dry_run: true,
                json_output: false,
                min_confidence: None,
                explain: false,
                color: "never",
            },
            false,
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
            &RepairOptions {
                input: Some(path),
                output: None,
                format: Some("json"),
                show_confidence: false,
                show_diff: false,
                dry_run: true,
                json_output: true,
                min_confidence: None,
                explain: false,
                color: "never",
            },
            false,
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
            &RepairOptions {
                input: Some(path),
                output: None,
                format: Some("json"),
                show_confidence: false,
                show_diff: false,
                dry_run: true,
                json_output: false,
                min_confidence: Some(0.0),
                explain: false,
                color: "never",
            },
            false,
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
            &RepairOptions {
                input: Some(path),
                output: None,
                format: Some("json"),
                show_confidence: false,
                show_diff: false,
                dry_run: true,
                json_output: false,
                min_confidence: Some(2.0),
                explain: false,
                color: "never",
            },
            false,
        );
        assert!(result.is_err());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_build_json_result_with_output() {
        let json = build_json_result(&JsonResultInput {
            format: "json",
            confidence: 0.95,
            detection_confidence: 0.9,
            repaired_needed: true,
            original_len: 20,
            repaired_len: 18,
            output: Some("out.json"),
            repaired_content: "{}",
        });
        assert!(json.contains(r#""format":"json""#));
        assert!(json.contains(r#""confidence":0.95"#));
        assert!(json.contains(r#""detection_confidence":0.9"#));
        assert!(json.contains(r#""repaired":true"#));
        assert!(json.contains(r#""original_length":20"#));
        assert!(json.contains(r#""repaired_length":18"#));
        assert!(json.contains(r#""output":"out.json""#));
        assert!(!json.contains(r#""content""#));
    }

    #[test]
    fn test_build_json_result_without_output() {
        let json = build_json_result(&JsonResultInput {
            format: "yaml",
            confidence: 0.8,
            detection_confidence: 0.75,
            repaired_needed: false,
            original_len: 10,
            repaired_len: 10,
            output: None,
            repaired_content: "key: val",
        });
        assert!(json.contains(r#""format":"yaml""#));
        assert!(json.contains(r#""detection_confidence":0.75"#));
        assert!(json.contains(r#""repaired":false"#));
        assert!(json.contains(r#""output":null"#));
        assert!(json.contains(r#""content":"key: val""#));
    }

    #[test]
    fn test_build_json_result_explicit_format_detection_confidence() {
        // When format is explicitly specified, detection_confidence should be 1.0
        let json = build_json_result(&JsonResultInput {
            format: "json",
            confidence: 0.9,
            detection_confidence: 1.0,
            repaired_needed: true,
            original_len: 15,
            repaired_len: 14,
            output: None,
            repaired_content: "{}",
        });
        assert!(json.contains(r#""detection_confidence":1"#));
    }

    #[test]
    fn test_explain_with_changes() {
        let mut tmp = std::env::temp_dir();
        tmp.push("anyrepair_explain_changes.json");
        std::fs::write(&tmp, r#"{"key": "value",}"#).unwrap();
        let path = tmp.to_str().unwrap();

        let result = handle_repair(
            &RepairOptions {
                input: Some(path),
                output: None,
                format: Some("json"),
                show_confidence: false,
                show_diff: false,
                dry_run: true,
                json_output: false,
                min_confidence: None,
                explain: true,
                color: "never",
            },
            false,
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
            &RepairOptions {
                input: Some(path),
                output: None,
                format: Some("json"),
                show_confidence: false,
                show_diff: false,
                dry_run: true,
                json_output: false,
                min_confidence: None,
                explain: true,
                color: "never",
            },
            false,
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
