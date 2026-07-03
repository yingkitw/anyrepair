//! JSON repair module
//!
//! Provides comprehensive JSON repair functionality with multiple strategies
//! for fixing common JSON issues from LLM outputs.

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
#[cfg(not(feature = "strict"))]
use crate::json_util::{is_valid_json, validate_json_errors};
use regex::Regex;
use std::sync::OnceLock;

// ============================================================================
// JSON Validator
// ============================================================================

/// JSON validator
pub struct JsonValidator;

impl Validator for JsonValidator {
    fn is_valid(&self, content: &str) -> bool {
        #[cfg(feature = "strict")]
        {
            serde_json::from_str::<serde_json::Value>(content.trim()).is_ok()
        }
        #[cfg(not(feature = "strict"))]
        {
            is_valid_json(content)
        }
    }

    fn validate(&self, content: &str) -> Vec<String> {
        #[cfg(feature = "strict")]
        {
            match serde_json::from_str::<serde_json::Value>(content.trim()) {
                Ok(_) => vec![],
                Err(e) => vec![e.to_string()],
            }
        }
        #[cfg(not(feature = "strict"))]
        {
            validate_json_errors(content)
        }
    }
}

#[cfg(test)]
mod validator_tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let validator = JsonValidator;
        assert!(validator.is_valid(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_invalid_json() {
        let validator = JsonValidator;
        assert!(!validator.is_valid(r#"{"key": "value",}"#));
    }

    #[test]
    fn test_validate_errors() {
        let validator = JsonValidator;
        let errors = validator.validate(r#"{"key": "value",}"#);
        assert!(!errors.is_empty());
    }
}

// ============================================================================
// Regex Cache
// ============================================================================

/// Cached regex patterns for JSON repair
pub struct RegexCache {
    pub missing_quotes: Regex,
    pub trailing_commas: Regex,
    pub unescaped_quotes: Regex,
    pub single_quotes: Regex,
    pub malformed_numbers_leading_zeros: Regex,
    pub malformed_numbers_trailing_dots: Regex,
    pub malformed_numbers_multiple_dots: Regex,
    pub malformed_numbers_scientific: Regex,
    pub boolean_values: Regex,
    pub boolean_variants: Regex,
    pub null_values: Regex,
    pub undefined_values: Regex,
    pub smart_quotes: Regex,
}

impl RegexCache {
    pub fn new() -> Result<Self> {
        Ok(Self {
            missing_quotes: Regex::new(r#"(^|\s|,|\{)\s*(\w+)\s*:"#)?,
            trailing_commas: Regex::new(r#",(\s*[}\]])"#)?,
            unescaped_quotes: Regex::new(r#""([^"\\]|\\.)*"[^,}\]]*"#)?,
            single_quotes: Regex::new(r#"'([^']*)'"#)?,
            malformed_numbers_leading_zeros: Regex::new(r#"\b0+(\d+)\b"#)?,
            malformed_numbers_trailing_dots: Regex::new(r#"\b(\d+)\.\s*([,}\]])"#)?,
            malformed_numbers_multiple_dots: Regex::new(r#"\b(\d+\.\d+)\.(\d+)\b"#)?,
            malformed_numbers_scientific: Regex::new(r#"\b(\d+)\s*(\+|-)\s*(\d+)\b"#)?,
            boolean_values: Regex::new(r#"\b(True|False|TRUE|FALSE|true|false)\b"#)?,
            boolean_variants: Regex::new(r#"\b(yes|no|on|off|Yes|No|On|Off|YES|NO|ON|OFF)\b"#)?,
            null_values: Regex::new(r#"\b(Null|NULL|null|None|NONE|none|nil|NIL)\b"#)?,
            undefined_values: Regex::new(r#"\b(undefined|Undefined|UNDEFINED)\b"#)?,
            smart_quotes: Regex::new(r#"[\u201c\u201d\u2018\u2019]"#)?,
        })
    }
}

static REGEX_CACHE: OnceLock<RegexCache> = OnceLock::new();

pub fn get_regex_cache() -> &'static RegexCache {
    REGEX_CACHE.get_or_init(|| RegexCache::new().expect("Failed to initialize regex cache"))
}

// ============================================================================
// Repair Strategies
// ============================================================================

/// Strategy to strip trailing content after JSON closes
pub struct StripTrailingContentStrategy;

impl RepairStrategy for StripTrailingContentStrategy {
    fn name(&self) -> &str {
        "StripTrailingContent"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = String::new();
        let mut brace_count = 0;
        let mut bracket_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut found_json_end = false;
        let chars: Vec<char> = content.chars().collect();
        let len = chars.len();

        for i in 0..len {
            let ch = chars[i];

            if escape_next {
                result.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => {
                    result.push(ch);
                    escape_next = true;
                }
                '"' => {
                    result.push(ch);
                    in_string = !in_string;
                }
                '{' if !in_string => {
                    result.push(ch);
                    brace_count += 1;
                }
                '}' if !in_string => {
                    result.push(ch);
                    brace_count -= 1;
                    if brace_count == 0 && bracket_count == 0 {
                        let mut j = i + 1;
                        while j < len
                            && (chars[j] == ' '
                                || chars[j] == '\n'
                                || chars[j] == '\t'
                                || chars[j] == '\r')
                        {
                            j += 1;
                        }

                        if j < len && (chars[j] == ',' || chars[j] == '{' || chars[j] == '[') {
                            found_json_end = false;
                        } else if j >= len || (!chars[j].is_alphanumeric() && chars[j] != '"') {
                            found_json_end = true;
                        }
                    }
                }
                '[' if !in_string => {
                    result.push(ch);
                    bracket_count += 1;
                }
                ']' if !in_string => {
                    result.push(ch);
                    bracket_count -= 1;
                    if brace_count == 0 && bracket_count == 0 {
                        let mut j = i + 1;
                        while j < len
                            && (chars[j] == ' '
                                || chars[j] == '\n'
                                || chars[j] == '\t'
                                || chars[j] == '\r')
                        {
                            j += 1;
                        }

                        if j < len && (chars[j] == ',' || chars[j] == '{' || chars[j] == '[') {
                            found_json_end = false;
                        } else if j >= len || (!chars[j].is_alphanumeric() && chars[j] != '"') {
                            found_json_end = true;
                        }
                    }
                }
                _ => {
                    if !found_json_end {
                        result.push(ch);
                    }
                }
            }
        }

        Ok(result)
    }

    fn priority(&self) -> u8 {
        100
    }
}

/// Strategy to fix trailing commas
pub struct FixTrailingCommasStrategy;

impl RepairStrategy for FixTrailingCommasStrategy {
    fn name(&self) -> &str {
        "FixTrailingCommas"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        Ok(cache.trailing_commas.replace_all(content, "$1").to_string())
    }

    fn priority(&self) -> u8 {
        90
    }
}

/// Strategy to fix single quotes
pub struct FixSingleQuotesStrategy;

impl RepairStrategy for FixSingleQuotesStrategy {
    fn name(&self) -> &str {
        "FixSingleQuotes"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        Ok(cache
            .single_quotes
            .replace_all(content, "\"$1\"")
            .to_string())
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Strategy to add missing quotes around keys
pub struct AddMissingQuotesStrategy;

impl RepairStrategy for AddMissingQuotesStrategy {
    fn name(&self) -> &str {
        "AddMissingQuotes"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        Ok(cache
            .missing_quotes
            .replace_all(content, "$1\"$2\":")
            .to_string())
    }

    fn priority(&self) -> u8 {
        80
    }
}

/// Strategy to fix malformed numbers
pub struct FixMalformedNumbersStrategy;

impl RepairStrategy for FixMalformedNumbersStrategy {
    fn name(&self) -> &str {
        "FixMalformedNumbers"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        let mut result = content.to_string();

        result = cache
            .malformed_numbers_leading_zeros
            .replace_all(&result, "$1")
            .to_string();
        result = cache
            .malformed_numbers_trailing_dots
            .replace_all(&result, "$1$2")
            .to_string();
        result = cache
            .malformed_numbers_multiple_dots
            .replace_all(&result, "$1$2")
            .to_string();
        result = cache
            .malformed_numbers_scientific
            .replace_all(&result, "$1e$2$3")
            .to_string();

        Ok(result)
    }

    fn priority(&self) -> u8 {
        75
    }
}

/// Strategy to fix boolean and null values
pub struct FixBooleanNullStrategy;

impl RepairStrategy for FixBooleanNullStrategy {
    fn name(&self) -> &str {
        "FixBooleanNull"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        let mut result = content.to_string();

        result = cache
            .boolean_values
            .replace_all(&result, |caps: &regex::Captures| {
                match caps[0].to_lowercase().as_str() {
                    "true" | "false" => caps[0].to_lowercase(),
                    _ => "true".to_string(),
                }
            })
            .to_string();

        result = cache.null_values.replace_all(&result, "null").to_string();
        result = cache
            .undefined_values
            .replace_all(&result, "null")
            .to_string();

        Ok(result)
    }

    fn priority(&self) -> u8 {
        70
    }
}

/// Strategy to normalize smart/curly quotes to straight quotes
pub struct FixSmartQuotesStrategy;

impl RepairStrategy for FixSmartQuotesStrategy {
    fn name(&self) -> &str {
        "FixSmartQuotes"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        Ok(cache
            .smart_quotes
            .replace_all(content, |c: &regex::Captures| {
                match &c[0] {
                    "\u{201c}" | "\u{201d}" => "\"".to_string(),
                    "\u{2018}" | "\u{2019}" => "'".to_string(),
                    other => other.to_string(),
                }
            })
            .to_string())
    }

    fn priority(&self) -> u8 {
        90
    }
}

/// Strategy to recognize boolean variants (yes/no, on/off, 1/0 as bare words)
pub struct FixBooleanVariantsStrategy;

impl RepairStrategy for FixBooleanVariantsStrategy {
    fn name(&self) -> &str {
        "FixBooleanVariants"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        Ok(cache
            .boolean_variants
            .replace_all(content, |caps: &regex::Captures| {
                match caps[0].to_lowercase().as_str() {
                    "yes" | "on" => "true".to_string(),
                    "no" | "off" => "false".to_string(),
                    other => other.to_string(),
                }
            })
            .to_string())
    }

    fn priority(&self) -> u8 {
        68
    }
}

/// Strategy to extract JSON from surrounding prose/preamble
pub struct ExtractJsonFromProseStrategy;

impl RepairStrategy for ExtractJsonFromProseStrategy {
    fn name(&self) -> &str {
        "ExtractJsonFromProse"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let trimmed = content.trim();

        // If already starts with { or [, no extraction needed
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            return Ok(trimmed.to_string());
        }

        // Only extract if there's actual prose text before the JSON block.
        // Find the first { or [ and check that preceding text is prose, not a JSON fragment.
        if let Some(pos) = trimmed.find('{').or_else(|| trimmed.find('[')) {
            let prefix = &trimmed[..pos];
            // Prose detection: prefix must NOT contain double quotes (JSON fragments always do)
            // and must have 3+ consecutive alphabetic chars (a real word/sentence).
            // This prevents false positives on streaming JSON chunks where key names
            // like "name" or "profile" precede a nested {.
            let has_prose = !prefix.contains('"')
                && prefix
                    .split(|c: char| !c.is_alphabetic())
                    .any(|word| word.len() >= 3);

            if !has_prose {
                return Ok(content.to_string());
            }

            let extracted = &trimmed[pos..];
            // Trim trailing non-JSON content
            let mut brace_depth = 0i32;
            let mut bracket_depth = 0i32;
            let mut end_pos = 0usize;

            for (i, ch) in extracted.char_indices() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => {
                        brace_depth -= 1;
                        if brace_depth == 0 && bracket_depth == 0 {
                            end_pos = i + 1;
                            break;
                        }
                    }
                    '[' => bracket_depth += 1,
                    ']' => {
                        bracket_depth -= 1;
                        if brace_depth == 0 && bracket_depth == 0 {
                            end_pos = i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            // Only extract if we found a balanced JSON structure.
            // If braces don't balance, this is a JSON fragment (e.g. from streaming), not prose+JSON.
            if end_pos > 0 {
                return Ok(extracted[..end_pos].to_string());
            }

            return Ok(content.to_string());
        }

        Ok(content.to_string())
    }

    fn priority(&self) -> u8 {
        95
    }
}

/// Strategy to add missing braces
pub struct AddMissingBracesStrategy;

impl RepairStrategy for AddMissingBracesStrategy {
    fn name(&self) -> &str {
        "AddMissingBraces"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let trimmed = content.trim();

        if trimmed.is_empty() {
            return Ok("{}".to_string());
        }

        let mut result = trimmed.to_string();
        let open_braces = trimmed.matches('{').count();
        let close_braces = trimmed.matches('}').count();
        let open_brackets = trimmed.matches('[').count();
        let close_brackets = trimmed.matches(']').count();

        if open_braces > close_braces {
            result.push_str(&"}".repeat(open_braces - close_braces));
        }

        if open_brackets > close_brackets {
            result.push_str(&"]".repeat(open_brackets - close_brackets));
        }

        if !result.starts_with('{') && !result.starts_with('[') {
            result = format!("{{{}}}", result);
        }

        Ok(result)
    }

    fn priority(&self) -> u8 {
        60
    }
}

/// Strategy for agentic AI response repair
pub struct FixAgenticAiResponseStrategy;

impl RepairStrategy for FixAgenticAiResponseStrategy {
    fn name(&self) -> &str {
        "FixAgenticAiResponse"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_regex_cache();
        let mut result = content.to_string();

        result = cache
            .undefined_values
            .replace_all(&result, "null")
            .to_string();
        result = cache.trailing_commas.replace_all(&result, "$1").to_string();
        result = cache
            .single_quotes
            .replace_all(&result, "\"$1\"")
            .to_string();

        Ok(result)
    }

    fn priority(&self) -> u8 {
        50
    }
}

/// Strategy to strip JavaScript-style comments from JSON
pub struct StripJsCommentsStrategy;

impl RepairStrategy for StripJsCommentsStrategy {
    fn name(&self) -> &str {
        "StripJsComments"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = String::new();
        let mut in_string = false;
        let mut escaped = false;
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\\' if in_string => {
                    // Toggle escape state
                    escaped = !escaped;
                    result.push(c);
                }
                '"' if !escaped => {
                    in_string = !in_string;
                    result.push(c);
                }
                '/' if !in_string => {
                    if let Some(&'/') = chars.peek() {
                        // Single-line comment: //
                        while chars.next() != Some('\n') && chars.peek().is_some() {
                            // Skip until newline
                        }
                    } else if let Some(&'*') = chars.peek() {
                        // Multi-line comment: /*
                        chars.next(); // consume '*'
                        loop {
                            match chars.next() {
                                Some('*') => {
                                    if chars.peek() == Some(&'/') {
                                        chars.next(); // consume '/'
                                        break;
                                    }
                                }
                                Some(_) => continue,
                                None => break,
                            }
                        }
                    } else {
                        result.push(c);
                    }
                    escaped = false;
                }
                _ => {
                    result.push(c);
                    // Reset escape state for non-backslash characters
                    if c != '\\' {
                        escaped = false;
                    }
                }
            }
        }

        Ok(result)
    }

    fn priority(&self) -> u8 {
        95
    }
}

// ============================================================================
// JSON Repairer
// ============================================================================

/// JSON repairer that can fix common JSON issues
///
/// Uses trait-based composition with GenericRepairer for better modularity
pub struct JsonRepairer {
    pub inner: crate::repairer_base::GenericRepairer,
}

impl JsonRepairer {
    /// Create a new JSON repairer
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(ExtractJsonFromProseStrategy),
            Box::new(StripTrailingContentStrategy),
            Box::new(StripJsCommentsStrategy),
            Box::new(FixSmartQuotesStrategy),
            Box::new(AddMissingQuotesStrategy),
            Box::new(FixTrailingCommasStrategy),
            Box::new(AddMissingBracesStrategy),
            Box::new(FixSingleQuotesStrategy),
            Box::new(FixMalformedNumbersStrategy),
            Box::new(FixBooleanNullStrategy),
            Box::new(FixBooleanVariantsStrategy),
            Box::new(FixAgenticAiResponseStrategy),
        ];

        let validator: Box<dyn Validator> = Box::new(JsonValidator);
        let inner = crate::repairer_base::GenericRepairer::new(validator, strategies);

        Self { inner }
    }
}

impl Default for JsonRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for JsonRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        self.inner.repair(content)
    }

    fn needs_repair(&self, content: &str) -> bool {
        self.inner.needs_repair(content)
    }

    fn confidence(&self, content: &str) -> f64 {
        // Use custom confidence calculation for JSON
        if self.inner.validator().is_valid(content) {
            return 1.0;
        }

        let mut score: f64 = 0.0;

        if content.contains('{') || content.contains('[') {
            score += 0.3;
        }

        if content.contains(':') {
            score += 0.2;
        }

        if content.contains('"') {
            score += 0.2;
        }

        if content.contains(',') {
            score += 0.1;
        }

        let open_braces = content.matches('{').count();
        let close_braces = content.matches('}').count();
        let open_brackets = content.matches('[').count();
        let close_brackets = content.matches(']').count();

        if open_braces == close_braces && open_brackets == close_brackets {
            score += 0.2;
        }

        score.min(1.0_f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_repairer_creation() {
        let repairer = JsonRepairer::new();
        assert!(!repairer.inner.strategies().is_empty());
    }

    #[test]
    fn test_json_repairer_default() {
        let repairer = JsonRepairer::default();
        assert!(!repairer.inner.strategies().is_empty());
    }

    #[test]
    fn test_json_confidence_valid() {
        let repairer = JsonRepairer::new();
        let confidence = repairer.confidence(r#"{"key": "value"}"#);
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_json_confidence_invalid() {
        let repairer = JsonRepairer::new();
        let confidence = repairer.confidence(r#"{"key": value}"#);
        assert!(confidence < 1.0);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_json_needs_repair() {
        let repairer = JsonRepairer::new();
        assert!(!repairer.needs_repair(r#"{"key": "value"}"#));
        assert!(repairer.needs_repair(r#"{"key": "value",}"#));
    }

    #[test]
    fn test_strip_js_comments() {
        let strategy = StripJsCommentsStrategy;
        // Single-line comment
        let input = r#"{"key": "value", // comment\n}"#;
        let result = strategy.apply(input).unwrap();
        assert!(!result.contains("//"));
        assert!(result.contains("value"));

        // Multi-line comment
        let input2 = r#"{"key": "value", /* multi-line
        comment */}"#;
        let result2 = strategy.apply(input2).unwrap();
        assert!(!result2.contains("/*"));

        // Comment in string should be preserved
        let input3 = r#"{"text": "not a // comment"}"#;
        let result3 = strategy.apply(input3).unwrap();
        assert!(result3.contains("//"));
    }

    #[test]
    fn test_json_with_js_comments_repair() {
        let mut repairer = JsonRepairer::new();
        let input = r#"{"key": "value", // this is a comment
        "another": "field" /* multi-line */}"#;
        let result = repairer.repair(input).unwrap();
        assert!(result.contains("key"));
        assert!(result.contains("value"));
        assert!(!result.contains("//"));
        assert!(!result.contains("/*"));
    }

    #[test]
    fn test_strip_js_comments_edge_cases() {
        let strategy = StripJsCommentsStrategy;

        // Comment at the start
        let input1 = r#"// comment at start
{"key": "value"}"#;
        let result1 = strategy.apply(input1).unwrap();
        assert!(!result1.contains("//"));
        assert!(result1.contains("key"));

        // Multiple single-line comments
        let input2 = r#"{"a": 1, // comment 1
"b": 2, // comment 2
"c": 3}"#;
        let result2 = strategy.apply(input2).unwrap();
        assert_eq!(result2.matches("//").count(), 0);

        // Comment with special characters
        let input3 = r#"{"key": "value", // comment with @#$%^&*()
}"#;
        let result3 = strategy.apply(input3).unwrap();
        assert!(!result3.contains("//"));

        // Empty comment
        let input4 = r#"{"key": "value", /**/}"#;
        let result4 = strategy.apply(input4).unwrap();
        assert!(!result4.contains("/*"));

        // Multi-line comment spanning multiple lines
        let input5 = r#"{
  "key": "value", /* this is a
  multi-line comment */"another": "field"}"#;
        let result5 = strategy.apply(input5).unwrap();
        assert!(!result5.contains("/*"));
        assert!(result5.contains("another"));

        // Comment with escaped quotes in string (should preserve)
        let input6 = r#"{"text": "not // a comment", "quote": "\"test\""}"#;
        let result6 = strategy.apply(input6).unwrap();
        assert!(result6.contains("//"));
        assert!(result6.contains("\\\"test\\\""));
    }

    #[test]
    fn test_json_with_various_comment_styles() {
        let mut repairer = JsonRepairer::new();

        // Real-world JSON with JS-style comments
        let input = r#"{
  // Configuration settings
  "apiVersion": "v1",
  "kind": "Config", /* Config kind */
  "metadata": {
    "name": "test-config", // Config name
    "namespace": "default"
  },
  // Data section
  "data": {
    "key": "value", /* Data key */
    "number": 42 // Answer to everything
  }
}"#;

        let result = repairer.repair(input).unwrap();
        assert!(result.contains("apiVersion"));
        assert!(result.contains("Config"));
        assert!(result.contains("test-config"));
        assert!(result.contains("data"));
        assert!(result.contains("key"));
        assert!(!result.contains("//"));
        assert!(!result.contains("/*"));

        // Verify it's valid JSON
        assert!(crate::json_util::is_valid_json(&result));
    }

    #[test]
    fn test_json_comments_preserve_string_content() {
        let mut repairer = JsonRepairer::new();

        // URLs with slashes should be preserved
        let input = r#"{"url": "https://example.com/path"}"#;
        let result = repairer.repair(input).unwrap();
        assert!(result.contains("https://"));

        // String with comment-like patterns
        let input2 = r#"{"text": "This is // not a comment", "code": "x = 1; // y = 2"}"#;
        let result2 = repairer.repair(input2).unwrap();
        assert!(result2.contains("This is // not"));
        assert!(result2.contains("x = 1; // y = 2"));

        // Note: Keys that start with // but are inside quotes are preserved
        // The StripJsCommentsStrategy correctly preserves content inside strings
        let input3 = r#"{"//comment": "remove me"}"#;
        let result3 = repairer.repair(input3).unwrap();
        // After AddMissingQuotesStrategy runs, the key gets quoted: "//comment" -> preserved
        // This is correct behavior - comments inside strings are preserved
        assert!(result3.contains(r#""//comment":"#));

        // However, actual line comments outside strings should be removed
        let input4 = r#"{"key": "value", // this is a real comment
        }"#;
        let result4 = repairer.repair(input4).unwrap();
        assert!(!result4.contains("// this is a real comment"));
    }

    #[test]
    fn test_json_comments_with_trailing_commas() {
        let mut repairer = JsonRepairer::new();

        // Combined issues: comments + trailing commas
        let input = r#"{
  "key1": "value1", // comment 1
  "key2": "value2", /* comment 2 */
  "key3": "value3",
}"#;

        let result = repairer.repair(input).unwrap();
        assert!(!result.contains("//"));
        assert!(!result.contains("/*"));
        assert!(!result.contains(",\n}"));
        assert!(result.contains("key1"));
        assert!(result.contains("key2"));
        assert!(result.contains("key3"));

        // Verify valid JSON
        assert!(crate::json_util::is_valid_json(&result));
    }

    #[test]
    fn test_smart_quotes_normalization() {
        let strategy = FixSmartQuotesStrategy;
        let input = "\u{201c}hello\u{201d}: \u{2018}world\u{2019}";
        let result = strategy.apply(input).unwrap();
        assert!(result.contains("\"hello\""));
        assert!(result.contains("'world'"));
        assert!(!result.contains('\u{201c}'));
        assert!(!result.contains('\u{201d}'));
    }

    #[test]
    fn test_smart_quotes_in_json_repair() {
        let mut repairer = JsonRepairer::new();
        let input = r#"{"name": "Alice \u201cBob\u201d"}"#;
        let result = repairer.repair(input).unwrap();
        assert!(!result.contains('\u{201c}'));
        assert!(!result.contains('\u{201d}'));
    }

    #[test]
    fn test_boolean_variants_yes_no() {
        let strategy = FixBooleanVariantsStrategy;
        let input = r#"{"enabled": yes, "disabled": no}"#;
        let result = strategy.apply(input).unwrap();
        assert!(result.contains("true"));
        assert!(result.contains("false"));
        assert!(!result.contains("yes"));
        assert!(!result.contains("no"));
    }

    #[test]
    fn test_boolean_variants_on_off() {
        let strategy = FixBooleanVariantsStrategy;
        let input = r#"{"power": on, "sleep": off}"#;
        let result = strategy.apply(input).unwrap();
        assert!(result.contains("true"));
        assert!(result.contains("false"));
    }

    #[test]
    fn test_boolean_variants_case_insensitive() {
        let strategy = FixBooleanVariantsStrategy;
        let input = r#"{"a": YES, "b": OFF}"#;
        let result = strategy.apply(input).unwrap();
        assert!(result.contains("true"));
        assert!(result.contains("false"));
    }

    #[test]
    fn test_extract_json_from_prose() {
        let strategy = ExtractJsonFromProseStrategy;
        let input = "Here is the result: {\"key\": \"value\"} as requested.";
        let result = strategy.apply(input).unwrap();
        assert!(result.starts_with('{'));
        assert!(result.ends_with('}'));
        assert!(!result.contains("Here is"));
        assert!(!result.contains("as requested"));
    }

    #[test]
    fn test_extract_json_array_from_prose() {
        let strategy = ExtractJsonFromProseStrategy;
        let input = "Sure! [1, 2, 3] is the array.";
        let result = strategy.apply(input).unwrap();
        assert!(result.starts_with('['));
        assert!(result.ends_with(']'));
    }

    #[test]
    fn test_extract_json_no_prose() {
        let strategy = ExtractJsonFromProseStrategy;
        let input = r#"{"key": "value"}"#;
        let result = strategy.apply(input).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_extract_json_nested_from_prose() {
        let strategy = ExtractJsonFromProseStrategy;
        let input = "Output: {\"a\": {\"b\": [1, 2]}} done.";
        let result = strategy.apply(input).unwrap();
        assert!(result.starts_with('{'));
        assert!(result.ends_with('}'));
        assert!(result.contains("\"b\""));
    }
}
