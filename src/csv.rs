//! CSV repair module

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};

/// Fix unquoted single-line fields that contain spaces.
///
/// Mirrors the regex `^([^",\n]+)$` (no multiline flag, so it only matches
/// when the whole content is a single line, optionally with a trailing
/// newline) with the closure that quotes the field when it contains a space
/// or comma. Since the matched field cannot contain a comma (excluded by the
/// character class), only the space check is effective.
fn fix_unquoted_strings(content: &str) -> String {
    let line = match content.strip_suffix('\n') {
        Some(rest) => rest,
        None => content,
    };
    // Multi-line content (an internal newline) cannot match `^...$`.
    if line.contains('\n') {
        return content.to_string();
    }
    // `[^",\n]+`: non-empty, no quotes, no commas, no newlines.
    if line.is_empty() || line.contains('"') || line.contains(',') {
        return content.to_string();
    }
    let quoted = if line.contains(' ') || line.contains(',') {
        format!("\"{}\"", line)
    } else {
        line.to_string()
    };
    if content.ends_with('\n') {
        format!("{}\n", quoted)
    } else {
        quoted
    }
}

/// Fix malformed quotes by pulling a trailing non-delimiter char inside.
///
/// Mirrors the regex `"([^"]*)"([^",\n])` → `"content+extra"`. A double-quoted
/// run followed by a single char that is not `"`, `,`, or `\n` gets that char
/// merged into the quoted content.
fn fix_malformed_quotes(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;
    while i < n {
        if chars[i] == '"' {
            // Collect until the closing '"'.
            let mut inner = String::new();
            let mut j = i + 1;
            while j < n && chars[j] != '"' {
                inner.push(chars[j]);
                j += 1;
            }
            if j < n && chars[j] == '"' {
                // Closing quote found at j; check the following char.
                if let Some(&extra) = chars.get(j + 1)
                    && extra != '"' && extra != ',' && extra != '\n' {
                        out.push('"');
                        out.push_str(&inner);
                        out.push(extra);
                        out.push('"');
                        i = j + 2;
                        continue;
                    }
                // No merge: emit the quoted string verbatim.
                out.push('"');
                out.push_str(&inner);
                out.push('"');
                i = j + 1;
                continue;
            }
            // No closing quote: emit the opening quote and continue.
            out.push('"');
            i += 1;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Collapse `, ,` (comma, optional whitespace, comma) into a single comma.
///
/// Mirrors the regex `,\s*,` → `,`.
fn fix_extra_commas(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;
    while i < n {
        if chars[i] == ',' {
            // Look ahead: skip whitespace, then expect another comma.
            let mut j = i + 1;
            while j < n && chars[j].is_whitespace() {
                j += 1;
            }
            if j < n && chars[j] == ',' {
                out.push(',');
                i = j + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// CSV repairer that can fix common CSV issues
///
/// Uses trait-based composition with GenericRepairer for better modularity
pub struct CsvRepairer {
    pub inner: crate::repairer_base::GenericRepairer,
}

impl CsvRepairer {
    /// Create a new CSV repairer
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(FixUnquotedStringsStrategy),
            Box::new(FixMalformedQuotesStrategy),
            Box::new(FixMissingQuotesStrategy),
            Box::new(FixExtraCommasStrategy),
            Box::new(FixMissingCommasStrategy),
            Box::new(AddHeadersStrategy),
        ];

        let validator: Box<dyn Validator> = Box::new(CsvValidator);
        let inner = crate::repairer_base::GenericRepairer::new(validator, strategies);

        Self { inner }
    }
}

impl Default for CsvRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for CsvRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        self.inner.repair(content)
    }

    fn needs_repair(&self, content: &str) -> bool {
        self.inner.needs_repair(content)
    }

    fn confidence(&self, content: &str) -> f64 {
        if content.trim().is_empty() {
            return 0.0;
        }

        // Calculate confidence based on CSV-like patterns
        let mut score: f64 = 0.0;

        // Check for commas (CSV delimiter)
        if content.contains(',') {
            score += 0.4;
        }

        // Check for quoted strings
        if content.contains('"') {
            score += 0.2;
        }

        // Check for multiple lines
        if content.lines().count() > 1 {
            score += 0.2;
        }

        // Check for consistent column count
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() > 1 {
            let first_line_commas = lines[0].matches(',').count();
            let consistent_commas = lines
                .iter()
                .all(|line| line.matches(',').count() == first_line_commas);
            if consistent_commas {
                score += 0.2;
            }
        }

        score.min(1.0)
    }
}

/// CSV validator
pub struct CsvValidator;

impl Validator for CsvValidator {
    fn is_valid(&self, content: &str) -> bool {
        csv_structure_valid(content)
    }

    fn validate(&self, content: &str) -> Vec<String> {
        if content.trim().is_empty() {
            return vec!["Empty CSV content".to_string()];
        }
        if csv_structure_valid(content) {
            vec![]
        } else {
            vec!["CSV structure validation failed".to_string()]
        }
    }
}

fn csv_structure_valid(content: &str) -> bool {
    if content.trim().is_empty() {
        return false;
    }

    let lines: Vec<&str> = content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.is_empty() {
        return false;
    }

    let mut column_count = None;
    for line in lines {
        let fields = match parse_csv_fields(line) {
            Ok(f) => f,
            Err(_) => return false,
        };
        if fields.is_empty() {
            return false;
        }
        match column_count {
            None => column_count = Some(fields.len()),
            Some(n) if n != fields.len() => return false,
            _ => {}
        }
    }

    true
}

fn parse_csv_fields(line: &str) -> std::result::Result<Vec<String>, ()> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if !in_quotes => in_quotes = true,
            '"' if in_quotes => {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    current.push('"');
                } else {
                    in_quotes = false;
                }
            }
            ',' if !in_quotes => {
                fields.push(std::mem::take(&mut current));
            }
            c => current.push(c),
        }
    }

    if in_quotes {
        return Err(());
    }
    fields.push(current);
    Ok(fields)
}

fn format_csv_line(fields: &[String]) -> String {
    fields
        .iter()
        .map(|field| {
            let needs_quotes = field.contains(',')
                || field.contains('"')
                || field.contains('\n')
                || field.contains(' ');
            if needs_quotes {
                format!("\"{}\"", field.replace('"', "\"\""))
            } else {
                field.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Strategy to fix unquoted strings that should be quoted
struct FixUnquotedStringsStrategy;

impl RepairStrategy for FixUnquotedStringsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        Ok(fix_unquoted_strings(content))
    }

    fn priority(&self) -> u8 {
        6
    }

    fn name(&self) -> &str {
        "FixUnquotedStringsStrategy"
    }
}

/// Strategy to fix malformed quotes
struct FixMalformedQuotesStrategy;

impl RepairStrategy for FixMalformedQuotesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        Ok(fix_malformed_quotes(content))
    }

    fn priority(&self) -> u8 {
        5
    }

    fn name(&self) -> &str {
        "FixMalformedQuotesStrategy"
    }
}

/// Strategy to fix missing quotes around values with commas
struct FixMissingQuotesStrategy;

impl RepairStrategy for FixMissingQuotesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let mut out = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                out.push(line.to_string());
                continue;
            }
            match parse_csv_fields(trimmed) {
                Ok(fields) => out.push(format_csv_line(&fields)),
                Err(_) => {
                    if trimmed.contains(' ') && trimmed.contains(',') {
                        out.push(format!("\"{}\"", trimmed));
                    } else {
                        out.push(trimmed.to_string());
                    }
                }
            }
        }
        Ok(out.join("\n"))
    }

    fn priority(&self) -> u8 {
        4
    }

    fn name(&self) -> &str {
        "FixMissingQuotesStrategy"
    }
}

/// Strategy to fix extra commas
struct FixExtraCommasStrategy;

impl RepairStrategy for FixExtraCommasStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        Ok(fix_extra_commas(content))
    }

    fn priority(&self) -> u8 {
        3
    }

    fn name(&self) -> &str {
        "FixExtraCommasStrategy"
    }
}

/// Strategy to fix missing commas (space-separated fields without commas)
struct FixMissingCommasStrategy;

impl RepairStrategy for FixMissingCommasStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let mut out = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                out.push(line.to_string());
                continue;
            }
            // Only add commas if line has no commas and multiple whitespace-separated tokens
            if !trimmed.contains(',') {
                let tokens: Vec<&str> = trimmed.split_whitespace().collect();
                if tokens.len() > 1 {
                    out.push(tokens.join(","));
                } else {
                    out.push(trimmed.to_string());
                }
            } else {
                out.push(trimmed.to_string());
            }
        }
        Ok(out.join("\n"))
    }

    fn priority(&self) -> u8 {
        2
    }

    fn name(&self) -> &str {
        "FixMissingCommasStrategy"
    }
}

/// Strategy to add headers if missing
struct AddHeadersStrategy;

impl RepairStrategy for AddHeadersStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Ok(content.to_string());
        }

        let first_line = lines[0].trim();

        // Check if first line looks like data (contains numbers or unquoted strings)
        if first_line.chars().any(|c| c.is_ascii_digit())
            || (!first_line.contains('"') && first_line.contains(','))
        {
            // Add generic headers
            let column_count = first_line.matches(',').count() + 1;
            let headers: Vec<String> = (1..=column_count)
                .map(|i| format!("column_{}", i))
                .collect();
            let header_line = headers.join(",");

            let mut result = vec![header_line];
            result.extend(lines.iter().map(|s| s.to_string()));
            Ok(result.join("\n"))
        } else {
            Ok(content.to_string())
        }
    }

    fn priority(&self) -> u8 {
        1
    }

    fn name(&self) -> &str {
        "AddHeadersStrategy"
    }
}
