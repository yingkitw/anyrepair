//! TOML repair module

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};

/// Effective end index for a `^...$` match: the end of the string, or just
/// before a single trailing newline (which regex `$` treats as the end).
fn effective_end(chars: &[char]) -> usize {
    let n = chars.len();
    if n > 0 && chars[n - 1] == '\n' {
        n - 1
    } else {
        n
    }
}

/// Consume `[a-zA-Z_][a-zA-Z0-9_]*` at `pos` (capped at `end`). Returns the
/// identifier and advances `pos`, or returns `None` without advancing.
fn eat_ident(chars: &[char], pos: &mut usize, end: usize) -> Option<String> {
    if *pos >= end {
        return None;
    }
    let c = chars[*pos];
    if !(c.is_ascii_alphabetic() || c == '_') {
        return None;
    }
    let start = *pos;
    *pos += 1;
    while *pos < end && (chars[*pos].is_ascii_alphanumeric() || chars[*pos] == '_') {
        *pos += 1;
    }
    Some(chars[start..*pos].iter().collect())
}

/// Consume `\s*` (whitespace, including newlines) up to `end`.
fn eat_ws(chars: &[char], pos: &mut usize, end: usize) {
    while *pos < end && chars[*pos].is_whitespace() {
        *pos += 1;
    }
}

/// `^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*([^"'\s].*[^"'\s])\s*$` → `indent + key + " = \"" + value + "\""`
fn fix_toml_missing_quotes(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let eff_end = effective_end(&chars);
    let mut pos = 0;
    eat_ws(&chars, &mut pos, eff_end);
    let indent: String = chars[..pos].iter().collect();
    let key = match eat_ident(&chars, &mut pos, eff_end) {
        Some(k) => k,
        None => return content.to_string(),
    };
    eat_ws(&chars, &mut pos, eff_end);
    if pos >= eff_end || chars[pos] != '=' {
        return content.to_string();
    }
    pos += 1;
    eat_ws(&chars, &mut pos, eff_end);
    let vchar = |c: char| c != '"' && c != '\'' && !c.is_whitespace();
    if pos >= eff_end || !vchar(chars[pos]) {
        return content.to_string();
    }
    let value_start = pos;
    // `.*` cannot cross a newline; the value's line must be the last line.
    let mut line_end = pos;
    while line_end < eff_end && chars[line_end] != '\n' {
        line_end += 1;
    }
    if line_end != eff_end {
        return content.to_string();
    }
    // Trim trailing whitespace; the last char must be a non-quote, non-ws char.
    let mut value_end = line_end;
    while value_end > value_start && chars[value_end - 1].is_whitespace() {
        value_end -= 1;
    }
    if value_end == value_start || !vchar(chars[value_end - 1]) {
        return content.to_string();
    }
    let value: String = chars[value_start..value_end].iter().collect();
    let mut out = format!("{}{} = \"{}\"", indent, key, value);
    if eff_end < n {
        out.push('\n');
    }
    out
}

/// `\[([^,\]]+),\]` → `[content]` (remove trailing comma in single-element array).
fn fix_toml_malformed_arrays(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;
    while i < n {
        if chars[i] == '[' {
            // [^,\]]+ (content with no comma or ])
            let mut j = i + 1;
            let content_start = j;
            while j < n && chars[j] != ',' && chars[j] != ']' {
                j += 1;
            }
            if j > content_start && j < n && chars[j] == ',' {
                // ,]
                if j + 1 < n && chars[j + 1] == ']' {
                    let inner: String = chars[content_start..j].iter().collect();
                    out.push('[');
                    out.push_str(&inner);
                    out.push(']');
                    i = j + 2;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// `^(\s*)\[([^]]+)\]\s*$` → `indent + "[" + name + "]"` (trims trailing whitespace after `]`).
fn fix_toml_malformed_tables(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let eff_end = effective_end(&chars);
    let mut pos = 0;
    eat_ws(&chars, &mut pos, eff_end);
    let indent: String = chars[..pos].iter().collect();
    if pos >= eff_end || chars[pos] != '[' {
        return content.to_string();
    }
    pos += 1;
    // [^]]+ (one or more chars that are not ']')
    let name_start = pos;
    while pos < eff_end && chars[pos] != ']' {
        pos += 1;
    }
    if pos == name_start || pos >= eff_end || chars[pos] != ']' {
        return content.to_string();
    }
    let name: String = chars[name_start..pos].iter().collect();
    pos += 1; // consume ']'
    eat_ws(&chars, &mut pos, eff_end);
    if pos != eff_end {
        return content.to_string();
    }
    let mut out = format!("{}[{}]", indent, name);
    if eff_end < n {
        out.push('\n');
    }
    out
}

/// `^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*'([^']*)'\s*$` → `indent + key + " = \"" + value + "\""`
fn fix_toml_malformed_strings(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let eff_end = effective_end(&chars);
    let mut pos = 0;
    eat_ws(&chars, &mut pos, eff_end);
    let indent: String = chars[..pos].iter().collect();
    let key = match eat_ident(&chars, &mut pos, eff_end) {
        Some(k) => k,
        None => return content.to_string(),
    };
    eat_ws(&chars, &mut pos, eff_end);
    if pos >= eff_end || chars[pos] != '=' {
        return content.to_string();
    }
    pos += 1;
    eat_ws(&chars, &mut pos, eff_end);
    if pos >= eff_end || chars[pos] != '\'' {
        return content.to_string();
    }
    pos += 1;
    // [^']* (zero or more non-' chars)
    let value_start = pos;
    while pos < eff_end && chars[pos] != '\'' {
        pos += 1;
    }
    if pos >= eff_end || chars[pos] != '\'' {
        return content.to_string();
    }
    let value: String = chars[value_start..pos].iter().collect();
    pos += 1; // consume closing '
    eat_ws(&chars, &mut pos, eff_end);
    if pos != eff_end {
        return content.to_string();
    }
    let mut out = format!("{}{} = \"{}\"", indent, key, value);
    if eff_end < n {
        out.push('\n');
    }
    out
}

/// `^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(\d+\.\d*\.\d+)` → `indent + key + " = " + fixed`
/// (prefix match; `fixed` collapses `..` into `.`).
fn fix_toml_malformed_numbers(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut pos = 0;
    eat_ws(&chars, &mut pos, n);
    let indent: String = chars[..pos].iter().collect();
    let key = match eat_ident(&chars, &mut pos, n) {
        Some(k) => k,
        None => return content.to_string(),
    };
    eat_ws(&chars, &mut pos, n);
    if pos >= n || chars[pos] != '=' {
        return content.to_string();
    }
    pos += 1;
    eat_ws(&chars, &mut pos, n);
    let num_start = pos;
    // \d+
    let d1 = pos;
    while pos < n && chars[pos].is_ascii_digit() {
        pos += 1;
    }
    if pos == d1 {
        return content.to_string();
    }
    if pos >= n || chars[pos] != '.' {
        return content.to_string();
    }
    pos += 1;
    // \d*
    while pos < n && chars[pos].is_ascii_digit() {
        pos += 1;
    }
    if pos >= n || chars[pos] != '.' {
        return content.to_string();
    }
    pos += 1;
    // \d+
    let d3 = pos;
    while pos < n && chars[pos].is_ascii_digit() {
        pos += 1;
    }
    if pos == d3 {
        return content.to_string();
    }
    let number: String = chars[num_start..pos].iter().collect();
    let fixed = number.replace("..", ".");
    let mut out = format!("{}{} = {}", indent, key, fixed);
    out.push_str(&chars[pos..].iter().collect::<String>());
    out
}

/// `^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})` → `indent + key + " = \"" + date + "\""`
/// (prefix match).
fn fix_toml_malformed_dates(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut pos = 0;
    eat_ws(&chars, &mut pos, n);
    let indent: String = chars[..pos].iter().collect();
    let key = match eat_ident(&chars, &mut pos, n) {
        Some(k) => k,
        None => return content.to_string(),
    };
    eat_ws(&chars, &mut pos, n);
    if pos >= n || chars[pos] != '=' {
        return content.to_string();
    }
    pos += 1;
    eat_ws(&chars, &mut pos, n);
    let date_start = pos;
    // \d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}
    if !eat_n_digits_then(&chars, &mut pos, n, 4, '-')
        || !eat_n_digits_then(&chars, &mut pos, n, 2, '-')
        || !eat_n_digits_then(&chars, &mut pos, n, 2, 'T')
        || !eat_n_digits_then(&chars, &mut pos, n, 2, ':')
        || !eat_n_digits_then(&chars, &mut pos, n, 2, ':')
        || !eat_n_digits(&chars, &mut pos, n, 2)
    {
        return content.to_string();
    }
    let date: String = chars[date_start..pos].iter().collect();
    let mut out = format!("{}{} = \"{}\"", indent, key, date);
    out.push_str(&chars[pos..].iter().collect::<String>());
    out
}

/// Consume exactly `count` ASCII digits then `sep` (if non-null). Returns false on mismatch.
fn eat_n_digits_then(chars: &[char], pos: &mut usize, n: usize, count: usize, sep: char) -> bool {
    if !eat_n_digits(chars, pos, n, count) {
        return false;
    }
    if *pos >= n || chars[*pos] != sep {
        return false;
    }
    *pos += 1;
    true
}

/// Consume exactly `count` ASCII digits.
fn eat_n_digits(chars: &[char], pos: &mut usize, n: usize, count: usize) -> bool {
    if *pos + count > n {
        return false;
    }
    for k in 0..count {
        if !chars[*pos + k].is_ascii_digit() {
            return false;
        }
    }
    *pos += count;
    true
}

/// TOML repairer that can fix common TOML issues
///
/// Uses trait-based composition with GenericRepairer for better modularity
pub struct TomlRepairer {
    pub inner: crate::repairer_base::GenericRepairer,
}

impl TomlRepairer {
    /// Create a new TOML repairer
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(FixMissingQuotesStrategy),
            Box::new(FixMalformedArraysStrategy),
            Box::new(FixMalformedTablesStrategy),
            Box::new(FixMalformedStringsStrategy),
            Box::new(FixMalformedNumbersStrategy),
            Box::new(FixMalformedDatesStrategy),
            Box::new(AddTableHeadersStrategy),
        ];

        let validator: Box<dyn Validator> = Box::new(TomlValidator);
        let inner = crate::repairer_base::GenericRepairer::new(validator, strategies);

        Self { inner }
    }
}

impl Default for TomlRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for TomlRepairer {
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

        // Calculate confidence based on TOML-like patterns
        let mut score: f64 = 0.0;

        // Check for table headers
        if content.contains('[') && content.contains(']') {
            score += 0.3;
        }

        // Check for key-value pairs
        if content.contains('=') {
            score += 0.3;
        }

        // Check for arrays
        if content.contains('[') && content.contains(',') {
            score += 0.2;
        }

        // Check for strings
        if content.contains('"') || content.contains("'") {
            score += 0.1;
        }

        // Check for numbers
        if content.chars().any(|c| c.is_ascii_digit()) {
            score += 0.1;
        }

        score.min(1.0)
    }
}

/// TOML validator
pub struct TomlValidator;

impl Validator for TomlValidator {
    fn is_valid(&self, content: &str) -> bool {
        toml_structure_valid(content)
    }

    fn validate(&self, content: &str) -> Vec<String> {
        if content.trim().is_empty() {
            return vec!["Empty TOML content".to_string()];
        }
        if toml_structure_valid(content) {
            vec![]
        } else {
            vec!["TOML structure validation failed".to_string()]
        }
    }
}

fn toml_structure_valid(content: &str) -> bool {
    if content.trim().is_empty() {
        return false;
    }

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
            continue;
        }

        if trimmed.contains('=') {
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let value = parts[1].trim();
                if !value.starts_with('"')
                    && !value.starts_with('\'')
                    && !value.starts_with('[')
                    && !value.starts_with('{')
                    && value.parse::<i64>().is_err()
                    && value.parse::<f64>().is_err()
                    && value != "true"
                    && value != "false"
                {
                    return false;
                }
            }
        }
    }

    true
}

/// Strategy to fix missing quotes around string values
struct FixMissingQuotesStrategy;

impl RepairStrategy for FixMissingQuotesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        Ok(fix_toml_missing_quotes(content))
    }

    fn priority(&self) -> u8 {
        6
    }

    fn name(&self) -> &str {
        "FixMissingQuotesStrategy"
    }
}

/// Strategy to fix malformed arrays
struct FixMalformedArraysStrategy;

impl RepairStrategy for FixMalformedArraysStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        Ok(fix_toml_malformed_arrays(content))
    }

    fn priority(&self) -> u8 {
        5
    }

    fn name(&self) -> &str {
        "FixMalformedArraysStrategy"
    }
}

/// Strategy to fix malformed table headers
struct FixMalformedTablesStrategy;

impl RepairStrategy for FixMalformedTablesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        Ok(fix_toml_malformed_tables(content))
    }

    fn priority(&self) -> u8 {
        4
    }

    fn name(&self) -> &str {
        "FixMalformedTablesStrategy"
    }
}

/// Strategy to fix malformed strings
struct FixMalformedStringsStrategy;

impl RepairStrategy for FixMalformedStringsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        Ok(fix_toml_malformed_strings(content))
    }

    fn priority(&self) -> u8 {
        3
    }

    fn name(&self) -> &str {
        "FixMalformedStringsStrategy"
    }
}

/// Strategy to fix malformed numbers
struct FixMalformedNumbersStrategy;

impl RepairStrategy for FixMalformedNumbersStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        Ok(fix_toml_malformed_numbers(content))
    }

    fn priority(&self) -> u8 {
        2
    }

    fn name(&self) -> &str {
        "FixMalformedNumbersStrategy"
    }
}

/// Strategy to fix malformed dates
struct FixMalformedDatesStrategy;

impl RepairStrategy for FixMalformedDatesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        Ok(fix_toml_malformed_dates(content))
    }

    fn priority(&self) -> u8 {
        1
    }

    fn name(&self) -> &str {
        "FixMalformedDatesStrategy"
    }
}

/// Strategy to add table headers if missing
struct AddTableHeadersStrategy;

impl RepairStrategy for AddTableHeadersStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut has_table_header = false;

        for line in lines {
            let trimmed = line.trim();

            // Check if this is a key-value pair without a table header
            if trimmed.contains('=') && !trimmed.starts_with('[') && !has_table_header {
                result.push("[root]".to_string());
                has_table_header = true;
            }

            result.push(line.to_string());
        }

        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        0
    }

    fn name(&self) -> &str {
        "AddTableHeadersStrategy"
    }
}
