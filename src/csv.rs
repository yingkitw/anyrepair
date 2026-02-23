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
    CSV_REGEX_CACHE.get_or_init(|| CsvRegexCache::new().expect("Failed to initialize CSV regex cache"))
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
            let consistent_commas = lines.iter().all(|line| line.matches(',').count() == first_line_commas);
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
        if content.trim().is_empty() {
            return false;
        }
        
        // Check for fields with spaces that should be quoted
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            // Split by comma and check each field
            let fields: Vec<&str> = trimmed.split(',').collect();
            for field in fields {
                let field = field.trim();
                // If field contains spaces and is not quoted, it's invalid
                if field.contains(' ') && !field.starts_with('"') && !field.ends_with('"') {
                    return false;
                }
            }
        }
        
        // Basic CSV validation using csv crate
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content.as_bytes());
        
        reader.records().all(|record| record.is_ok())
    }
    
    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        if content.trim().is_empty() {
            errors.push("Empty CSV content".to_string());
            return errors;
        }
        
        // Try to parse with csv crate
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content.as_bytes());
        
        for (line_num, result) in reader.records().enumerate() {
            match result {
                Ok(_) => {}, // Valid record
                Err(e) => {
                    errors.push(format!("CSV parsing error at line {}: {}", line_num + 1, e));
                }
            }
        }
        
        errors
    }
}

/// Strategy to fix unquoted strings that should be quoted
struct FixUnquotedStringsStrategy;

impl RepairStrategy for FixUnquotedStringsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_csv_regex_cache();
        let result = cache.unquoted_strings.replace_all(content, |caps: &regex::Captures| {
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
        let result = cache.malformed_quotes.replace_all(content, |caps: &regex::Captures| {
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
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content.as_bytes());
        
        let mut writer = csv::WriterBuilder::new()
            .from_writer(Vec::new());
        
        for result in reader.records() {
            match result {
                Ok(record) => {
                    let mut fixed_record = Vec::new();
                    for field in record.iter() {
                        // If field contains spaces and is not quoted, add quotes
                        if field.contains(' ') && !field.starts_with('"') && !field.ends_with('"') {
                            fixed_record.push(format!("\"{}\"", field));
                        } else {
                            fixed_record.push(field.to_string());
                        }
                    }
                    writer.write_record(&fixed_record)?;
                }
                Err(_) => {
                    // If parsing fails, try to fix the line manually
                    let lines: Vec<&str> = content.lines().collect();
                    let mut result = Vec::new();
                    
                    for line in lines {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            result.push(line.to_string());
                            continue;
                        }
                        
                        // Simple approach: if line contains spaces and commas, quote the whole line
                        if trimmed.contains(' ') && trimmed.contains(',') {
                            result.push(format!("\"{}\"", trimmed));
                        } else {
                            result.push(trimmed.to_string());
                        }
                    }
                    
                    return Ok(result.join("\n"));
                }
            }
        }
        
        Ok(String::from_utf8(writer.into_inner()?)?)
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
        let result = cache.missing_commas.replace_all(content, |caps: &regex::Captures| {
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
        if first_line.chars().any(|c| c.is_ascii_digit()) || 
           (!first_line.contains('"') && first_line.contains(',')) {
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
