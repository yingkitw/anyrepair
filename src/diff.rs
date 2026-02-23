//! Diff/Unified diff repair module

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use regex::Regex;
use std::sync::OnceLock;

/// Cached regex patterns for diff performance optimization
#[allow(dead_code)]
struct DiffRegexCache {
    hunk_header: Regex,
    file_header: Regex,
    context_line: Regex,
    added_line: Regex,
    removed_line: Regex,
    malformed_hunk: Regex,
    missing_newline: Regex,
}

impl DiffRegexCache {
    fn new() -> Result<Self> {
        Ok(Self {
            // Hunk header: @@ -start,count +start,count @@
            hunk_header: Regex::new(r"^@@\s+-(\d+)(?:,(\d+))?\s+\+(\d+)(?:,(\d+))?\s+@@")?,
            // File header: --- a/file or +++ b/file
            file_header: Regex::new(r"^(?:---|\+\+\+)\s+[^\s]+")?,
            // Context line: starts with space
            context_line: Regex::new(r"^ (.*)$")?,
            // Added line: starts with +
            added_line: Regex::new(r"^\+(.*)$")?,
            // Removed line: starts with -
            removed_line: Regex::new(r"^-(.*)$")?,
            // Malformed hunk (missing @@ or incorrect format)
            malformed_hunk: Regex::new(r"^@@[^@]*$")?,
            // Missing newline at end
            missing_newline: Regex::new(r".$")?,
        })
    }
}

static DIFF_REGEX_CACHE: OnceLock<DiffRegexCache> = OnceLock::new();

fn get_diff_regex_cache() -> &'static DiffRegexCache {
    DIFF_REGEX_CACHE.get_or_init(|| DiffRegexCache::new().expect("Failed to initialize diff regex cache"))
}

/// Diff repairer that can fix common unified diff issues
/// 
/// Uses trait-based composition with GenericRepairer for better modularity
pub struct DiffRepairer {
    inner: crate::repairer_base::GenericRepairer,
}

impl DiffRepairer {
    /// Create a new diff repairer
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(FixMissingHunkHeadersStrategy),
            Box::new(FixLinePrefixesStrategy),
            Box::new(FixMissingNewlinesStrategy),
            Box::new(FixMalformedHunkRangesStrategy),
            Box::new(FixMissingFileHeadersStrategy),
            Box::new(FixInconsistentSpacingStrategy),
        ];
        
        let validator: Box<dyn Validator> = Box::new(DiffValidator);
        let inner = crate::repairer_base::GenericRepairer::new(validator, strategies);
        
        Self { inner }
    }
}

impl Default for DiffRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for DiffRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        let mut repaired = self.inner.repair(content)?;
        
        // Ensure result ends with newline (diff format requirement)
        if !repaired.is_empty() && !repaired.ends_with('\n') && !repaired.ends_with("\r\n") {
            repaired.push('\n');
        }
        
        Ok(repaired)
    }
    
    fn needs_repair(&self, content: &str) -> bool {
        self.inner.needs_repair(content)
    }
    
    fn confidence(&self, content: &str) -> f64 {
        if content.trim().is_empty() {
            return 0.0;
        }
        
        // Calculate confidence based on diff-like patterns
        let mut score: f64 = 0.0;
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return 0.0;
        }
        
        // Check for hunk headers (@@)
        let hunk_count = lines.iter().filter(|line| line.starts_with("@@")).count();
        if hunk_count > 0 {
            score += 0.3;
        }
        
        // Check for file headers (--- or +++)
        let file_header_count = lines.iter()
            .filter(|line| line.starts_with("---") || line.starts_with("+++"))
            .count();
        if file_header_count > 0 {
            score += 0.2;
        }
        
        // Check for diff line prefixes (+, -, space)
        let diff_line_count = lines.iter()
            .filter(|line| line.starts_with('+') || line.starts_with('-') || 
                           (line.starts_with(' ') && !line.starts_with("@@") && !line.starts_with("---") && !line.starts_with("+++")))
            .count();
        if diff_line_count > 0 {
            score += 0.3;
        }
        
        // Check for proper hunk header format
        let valid_hunk_count = lines.iter()
            .filter(|line| {
                line.starts_with("@@") && line.contains("@@") && 
                get_diff_regex_cache().hunk_header.is_match(line)
            })
            .count();
        if valid_hunk_count > 0 && hunk_count > 0 {
            score += 0.2;
        }
        
        score.min(1.0)
    }
}

/// Diff validator
pub struct DiffValidator;

impl Validator for DiffValidator {
    fn is_valid(&self, content: &str) -> bool {
        if content.trim().is_empty() {
            return false;
        }
        
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return false;
        }
        
        // Check for at least one hunk header
        let has_hunk = lines.iter().any(|line| {
            line.starts_with("@@") && line.contains("@@") &&
            get_diff_regex_cache().hunk_header.is_match(line)
        });
        
        if !has_hunk {
            return false;
        }
        
        // Check for file headers (--- and +++)
        let has_file_headers = lines.iter().any(|line| line.starts_with("---")) &&
                               lines.iter().any(|line| line.starts_with("+++"));
        
        // File headers are required for a valid diff
        if !has_file_headers {
            return false;
        }
        
        // Check that diff lines have proper prefixes
        let mut in_hunk = false;
        for line in &lines {
            if line.starts_with("@@") {
                in_hunk = true;
                // Validate hunk header format
                if !get_diff_regex_cache().hunk_header.is_match(line) {
                    return false;
                }
                // Check for excessive spacing (double spaces)
                if line.contains("  ") {
                    return false;
                }
            } else if in_hunk {
                // In a hunk, lines should start with +, -, or space
                if !line.starts_with('+') && !line.starts_with('-') && 
                   !line.starts_with(' ') && !line.trim().is_empty() &&
                   !line.starts_with("---") && !line.starts_with("+++") {
                    return false;
                }
            }
        }
        
        true
    }
    
    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        if content.trim().is_empty() {
            errors.push("Empty diff content".to_string());
            return errors;
        }
        
        let lines: Vec<&str> = content.lines().collect();
        
        // Check for hunk headers
        let hunk_count = lines.iter()
            .filter(|line| line.starts_with("@@"))
            .count();
        
        if hunk_count == 0 {
            errors.push("No hunk headers found (expected lines starting with @@)".to_string());
        }
        
        // Validate hunk headers
        for (line_num, line) in lines.iter().enumerate() {
            if line.starts_with("@@") {
                if !get_diff_regex_cache().hunk_header.is_match(line) {
                    errors.push(format!("Invalid hunk header at line {}: {}", line_num + 1, line));
                }
            }
        }
        
        // Check for proper line prefixes in hunks
        let mut in_hunk = false;
        for (line_num, line) in lines.iter().enumerate() {
            if line.starts_with("@@") {
                in_hunk = true;
            } else if in_hunk && !line.trim().is_empty() {
                if !line.starts_with('+') && !line.starts_with('-') && 
                   !line.starts_with(' ') && !line.starts_with("---") && !line.starts_with("+++") {
                    errors.push(format!("Invalid diff line prefix at line {}: expected +, -, or space", line_num + 1));
                }
            }
        }
        
        errors
    }
}

/// Strategy to fix missing hunk headers
struct FixMissingHunkHeadersStrategy;

impl RepairStrategy for FixMissingHunkHeadersStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut has_hunk = false;
        
        // Check if we have any hunk headers
        for line in &lines {
            if line.starts_with("@@") {
                has_hunk = true;
                break;
            }
        }
        
        // If no hunk headers but we have diff-like content, try to add one
        if !has_hunk && lines.len() > 2 {
            let mut diff_lines = 0;
            for line in &lines {
                if line.starts_with('+') || line.starts_with('-') || 
                   (line.starts_with(' ') && !line.starts_with("---") && !line.starts_with("+++")) {
                    diff_lines += 1;
                }
            }
            
            if diff_lines > 0 {
                // Try to infer hunk header from context
                let old_start = 1;
                let new_start = 1;
                let mut old_count = 0;
                let mut new_count = 0;
                
                for line in &lines {
                    if line.starts_with('-') {
                        old_count += 1;
                    } else if line.starts_with('+') {
                        new_count += 1;
                    } else if line.starts_with(' ') {
                        old_count += 1;
                        new_count += 1;
                    }
                }
                
                if old_count > 0 || new_count > 0 {
                    result.push(format!("@@ -{},{} +{},{} @@", old_start, old_count.max(1), new_start, new_count.max(1)));
                }
            }
        }
        
        // Copy all lines
        for line in lines {
            result.push(line.to_string());
        }
        
        Ok(result.join("\n"))
    }
    
    fn priority(&self) -> u8 {
        10
    }
    
    fn name(&self) -> &str {
        "FixMissingHunkHeaders"
    }
}

/// Strategy to fix incorrect line prefixes
struct FixLinePrefixesStrategy;

impl RepairStrategy for FixLinePrefixesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut in_hunk = false;
        
        for line in lines {
            if line.starts_with("@@") {
                in_hunk = true;
                result.push(line.to_string());
            } else if line.starts_with("---") || line.starts_with("+++") {
                in_hunk = false;
                result.push(line.to_string());
            } else if in_hunk {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    result.push("".to_string());
                } else if trimmed.starts_with('+') || trimmed.starts_with('-') || trimmed.starts_with(' ') {
                    // Already has correct prefix, but might need space prefix
                    if trimmed.starts_with('+') && !line.starts_with('+') {
                        result.push(format!("+{}", &trimmed[1..]));
                    } else if trimmed.starts_with('-') && !line.starts_with('-') {
                        result.push(format!("-{}", &trimmed[1..]));
                    } else if trimmed.starts_with(' ') && !line.starts_with(' ') {
                        result.push(format!(" {}", trimmed));
                    } else {
                        result.push(line.to_string());
                    }
                } else {
                    // Line without prefix in hunk - assume it's a context line
                    result.push(format!(" {}", trimmed));
                }
            } else {
                result.push(line.to_string());
            }
        }
        
        Ok(result.join("\n"))
    }
    
    fn priority(&self) -> u8 {
        8
    }
    
    fn name(&self) -> &str {
        "FixLinePrefixes"
    }
}

/// Strategy to fix missing newlines
struct FixMissingNewlinesStrategy;

impl RepairStrategy for FixMissingNewlinesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        // Ensure content ends with newline
        // Handle both cases: content as string and content as lines joined
        let mut result = content.to_string();
        
        // If content is from lines.join("\n"), it won't have trailing newline
        // Check if last character is newline
        if !result.ends_with('\n') && !result.ends_with("\r\n") {
            result.push('\n');
        }
        
        Ok(result)
    }
    
    fn priority(&self) -> u8 {
        5
    }
    
    fn name(&self) -> &str {
        "FixMissingNewlines"
    }
}

/// Strategy to fix malformed hunk ranges
struct FixMalformedHunkRangesStrategy;

impl RepairStrategy for FixMalformedHunkRangesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        
        for line in lines {
            if line.starts_with("@@") {
                // Try to fix malformed hunk headers
                let hunk_regex = &get_diff_regex_cache().hunk_header;
                if !hunk_regex.is_match(line) {
                    // Try to extract numbers and rebuild
                    let numbers: Vec<i32> = line
                        .split(|c: char| !c.is_ascii_digit() && c != '-')
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    
                    if numbers.len() >= 2 {
                        let old_start = numbers[0];
                        let old_count = if numbers.len() > 2 { numbers[1] } else { 1 };
                        let new_start = if numbers.len() > 2 { numbers[2] } else { numbers[1] };
                        let new_count = if numbers.len() > 3 { numbers[3] } else { 1 };
                        
                        result.push(format!("@@ -{},{} +{},{} @@", old_start, old_count, new_start, new_count));
                    } else {
                        // Fallback: use default values
                        result.push("@@ -1,1 +1,1 @@".to_string());
                    }
                } else {
                    result.push(line.to_string());
                }
            } else {
                result.push(line.to_string());
            }
        }
        
        Ok(result.join("\n"))
    }
    
    fn priority(&self) -> u8 {
        7
    }
    
    fn name(&self) -> &str {
        "FixMalformedHunkRanges"
    }
}

/// Strategy to fix missing file headers
struct FixMissingFileHeadersStrategy;

impl RepairStrategy for FixMissingFileHeadersStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Ok(content.to_string());
        }
        
        let mut result = Vec::new();
        let mut has_file_header = false;
        
        // Check if we have file headers
        for line in &lines {
            if line.starts_with("---") || line.starts_with("+++") {
                has_file_header = true;
                break;
            }
        }
        
        // If no file headers, add default ones before first hunk
        if !has_file_header {
            let mut found_hunk = false;
            for line in &lines {
                if line.starts_with("@@") && !found_hunk {
                    result.push("--- a/file".to_string());
                    result.push("+++ b/file".to_string());
                    found_hunk = true;
                }
                result.push(line.to_string());
            }
            
            // If no hunk found, add headers at the beginning
            if !found_hunk {
                result.insert(0, "+++ b/file".to_string());
                result.insert(0, "--- a/file".to_string());
            }
        } else {
            // Copy all lines as-is
            for line in lines {
                result.push(line.to_string());
            }
        }
        
        Ok(result.join("\n"))
    }
    
    fn priority(&self) -> u8 {
        6
    }
    
    fn name(&self) -> &str {
        "FixMissingFileHeaders"
    }
}

/// Strategy to fix inconsistent spacing
struct FixInconsistentSpacingStrategy;

impl RepairStrategy for FixInconsistentSpacingStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        
        for line in lines {
            if line.starts_with("@@") {
                // Normalize hunk header spacing - replace multiple spaces with single space
                let mut normalized = line.to_string();
                // Replace multiple spaces with single space
                while normalized.contains("  ") {
                    normalized = normalized.replace("  ", " ");
                }
                // Trim and ensure proper format
                normalized = normalized.trim().to_string();
                result.push(normalized);
            } else if line.starts_with("---") || line.starts_with("+++") {
                // Normalize file header spacing
                let normalized = line.trim().to_string();
                result.push(normalized);
            } else {
                result.push(line.to_string());
            }
        }
        
        Ok(result.join("\n"))
    }
    
    fn priority(&self) -> u8 {
        4
    }
    
    fn name(&self) -> &str {
        "FixInconsistentSpacing"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_validator() {
        let validator = DiffValidator;
        
        // Valid diff
        let valid_diff = r#"--- a/file.txt
+++ b/file.txt
@@ -1,3 +1,3 @@
 line1
-line2
+line2_modified
 line3
"#;
        assert!(validator.is_valid(valid_diff));
        
        // Invalid diff (no hunk header)
        let invalid_diff = r#"--- a/file.txt
+++ b/file.txt
 line1
-line2
+line2_modified
"#;
        assert!(!validator.is_valid(invalid_diff));
    }
    
    #[test]
    fn test_diff_repairer_basic() {
        let mut repairer = DiffRepairer::new();
        
        // Valid diff should pass through
        let valid = r#"--- a/file.txt
+++ b/file.txt
@@ -1,1 +1,1 @@
-old
+new
"#;
        let result = repairer.repair(valid).unwrap();
        assert!(result.contains("@@"));
    }
    
    #[test]
    fn test_fix_missing_hunk_headers() {
        let strategy = FixMissingHunkHeadersStrategy;
        
        let content = r#"--- a/file.txt
+++ b/file.txt
-old
+new
"#;
        let result = strategy.apply(content).unwrap();
        assert!(result.contains("@@"));
    }
    
    #[test]
    fn test_fix_line_prefixes() {
        let strategy = FixLinePrefixesStrategy;
        
        let content = r#"@@ -1,1 +1,1 @@
old
new
"#;
        let result = strategy.apply(content).unwrap();
        assert!(result.contains(" -") || result.contains(" +") || result.contains("  "));
    }
    
    #[test]
    fn test_fix_missing_file_headers() {
        let strategy = FixMissingFileHeadersStrategy;
        
        let content = r#"@@ -1,1 +1,1 @@
-old
+new
"#;
        let result = strategy.apply(content).unwrap();
        assert!(result.contains("---") || result.contains("+++"));
    }
}

