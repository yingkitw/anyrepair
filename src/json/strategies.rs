//! JSON repair strategies
//! Contains all strategy implementations for JSON repair

use crate::error::Result;
use crate::traits::RepairStrategy;
use regex::Regex;
use std::sync::OnceLock;

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
    pub null_values: Regex,
    pub undefined_values: Regex,
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
            null_values: Regex::new(r#"\b(Null|NULL|null|None|NONE|none|nil|NIL)\b"#)?,
            undefined_values: Regex::new(r#"\b(undefined|Undefined|UNDEFINED)\b"#)?,
        })
    }
}

static REGEX_CACHE: OnceLock<RegexCache> = OnceLock::new();

pub fn get_regex_cache() -> &'static RegexCache {
    REGEX_CACHE.get_or_init(|| RegexCache::new().expect("Failed to initialize regex cache"))
}

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
                        while j < len && (chars[j] == ' ' || chars[j] == '\n' || chars[j] == '\t' || chars[j] == '\r') {
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
                        while j < len && (chars[j] == ' ' || chars[j] == '\n' || chars[j] == '\t' || chars[j] == '\r') {
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
        Ok(cache.single_quotes.replace_all(content, "\"$1\"").to_string())
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
        Ok(cache.missing_quotes.replace_all(content, "$1\"$2\":").to_string())
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
        
        result = cache.malformed_numbers_leading_zeros.replace_all(&result, "$1").to_string();
        result = cache.malformed_numbers_trailing_dots.replace_all(&result, "$1$2").to_string();
        result = cache.malformed_numbers_multiple_dots.replace_all(&result, "$1$2").to_string();
        result = cache.malformed_numbers_scientific.replace_all(&result, "$1e$2$3").to_string();
        
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
        
        result = cache.boolean_values.replace_all(&result, |caps: &regex::Captures| {
            match caps[0].to_lowercase().as_str() {
                s if s == "true" => "true".to_string(),
                s if s == "false" => "false".to_string(),
                _ => "true".to_string(),
            }
        }).to_string();
        
        result = cache.null_values.replace_all(&result, "null").to_string();
        result = cache.undefined_values.replace_all(&result, "null").to_string();
        
        Ok(result)
    }
    
    fn priority(&self) -> u8 {
        70
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
        
        result = cache.undefined_values.replace_all(&result, "null").to_string();
        result = cache.trailing_commas.replace_all(&result, "$1").to_string();
        result = cache.single_quotes.replace_all(&result, "\"$1\"").to_string();
        
        Ok(result)
    }
    
    fn priority(&self) -> u8 {
        50
    }
}
