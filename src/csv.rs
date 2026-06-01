//! CSV repair module

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use regex::Regex;
use std::sync::OnceLock;

/// Cached regex patterns for CSV performance optimization
#[allow(dead_code)]
struct CsvRegexCache {
    unquoted_strings: Regex,
    malformed_quotes: Regex,
    missing_quotes: Regex,
    extra_commas: Regex,
    missing_commas: Regex,
}

impl CsvRegexCache {
    fn new() -> Result<Self> {
        Ok(Self {
            unquoted_strings: Regex::new(r#"^([^",\n]+)$"#)?,
            malformed_quotes: Regex::new(r#""([^"]*)"([^",\n])"#)?,
            missing_quotes: Regex::new(r#"\b([^",\n]*\s+[^",\n]*)\b"#)?,
            extra_commas: Regex::new(r#",\s*,"#)?,
            missing_commas: Regex::new(r#"[^,\n]\s+[^,\n]"#)?,
        })
    }
}

static CSV_REGEX_CACHE: OnceLock<CsvRegexCache> = OnceLock::new();

fn get_csv_regex_cache() -> &'static CsvRegexCache {
    CSV_REGEX_CACHE
        .get_or_init(|| CsvRegexCache::new().expect("Failed to initialize CSV regex cache"))
}

/// CSV repairer that can fix common CSV issues
///
/// Uses trait-based composition with GenericRepairer for better modularity
pub struct CsvRepairer {
    inner: crate::repairer_base::GenericRepairer,
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
        let cache = get_csv_regex_cache();
        let result = cache
            .unquoted_strings
            .replace_all(content, |caps: &regex::Captures| {
                let content = &caps[1];
                // Only quote if it contains spaces or special characters
                if content.contains(' ') || content.contains(',') {
                    format!("\"{}\"", content)
                } else {
                    content.to_string()
                }
            });

        Ok(result.to_string())
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
        let cache = get_csv_regex_cache();
        let result = cache
            .malformed_quotes
            .replace_all(content, |caps: &regex::Captures| {
                let content = &caps[1];
                let extra = &caps[2];
                format!("\"{}{}\"", content, extra)
            });

        Ok(result.to_string())
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
        let cache = get_csv_regex_cache();
        let result = cache.extra_commas.replace_all(content, ",");

        Ok(result.to_string())
    }

    fn priority(&self) -> u8 {
        3
    }

    fn name(&self) -> &str {
        "FixExtraCommasStrategy"
    }
}

/// Strategy to fix missing commas
struct FixMissingCommasStrategy;

impl RepairStrategy for FixMissingCommasStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_csv_regex_cache();
        let result = cache
            .missing_commas
            .replace_all(content, |caps: &regex::Captures| {
                let content = &caps[0];
                content.replace(' ', ",")
            });

        Ok(result.to_string())
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
