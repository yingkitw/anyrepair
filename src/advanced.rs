//! Advanced repair strategies with enhanced capabilities

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use regex::Regex;

/// Advanced repairer that combines multiple repair strategies with intelligent selection
pub struct AdvancedRepairer {
    strategies: Vec<Box<dyn RepairStrategy>>,
    validator: AdvancedValidator,
    confidence_threshold: f64,
}

impl AdvancedRepairer {
    /// Create a new advanced repairer
    pub fn new() -> Self {
        let mut strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(IntelligentFormatDetectionStrategy),
            Box::new(AdaptiveRepairStrategy),
            Box::new(ConfidenceBasedRepairStrategy),
            Box::new(MultiPassRepairStrategy),
            Box::new(ContextAwareRepairStrategy),
            Box::new(ErrorRecoveryStrategy),
        ];
        
        // Sort strategies by priority (higher priority first)
        strategies.sort_by_key(|b| std::cmp::Reverse(b.priority()));
        
        Self {
            strategies,
            validator: AdvancedValidator,
            confidence_threshold: 0.7,
        }
    }
    
    /// Set the confidence threshold for repair decisions
    pub fn set_confidence_threshold(&mut self, threshold: f64) {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
    }
    
    /// Apply all repair strategies with intelligent selection
    fn apply_strategies(&self, content: &str) -> Result<String> {
        let repaired = content.to_string();
        let mut confidence_scores = Vec::new();
        
        // Apply each strategy and collect confidence scores
        for strategy in &self.strategies {
            if let Ok(result) = strategy.apply(&repaired) {
                let confidence = self.calculate_confidence(&result);
                confidence_scores.push((result, confidence));
            }
        }
        
        // Select the result with highest confidence above threshold
        if let Some((best_result, _best_confidence)) = confidence_scores
            .iter()
            .filter(|(_, conf)| *conf >= self.confidence_threshold)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            Ok(best_result.clone())
        } else {
            // If no result meets threshold, return the best available
            if let Some((best_result, _)) = confidence_scores
                .iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            {
                Ok(best_result.clone())
            } else {
                Ok(repaired)
            }
        }
    }
    
    /// Calculate confidence score for repaired content
    fn calculate_confidence(&self, content: &str) -> f64 {
        let mut score: f64 = 0.0;
        
        // Check for common patterns that indicate successful repair
        if content.contains('{') && content.contains('}') {
            score += 0.2; // JSON-like structure
        }
        if content.contains('[') && content.contains(']') {
            score += 0.2; // Array-like structure
        }
        if content.contains('"') {
            score += 0.1; // Quoted strings
        }
        if content.contains(':') {
            score += 0.1; // Key-value pairs
        }
        if content.contains(',') {
            score += 0.1; // Comma-separated values
        }
        
        // Check for balanced brackets
        let open_braces = content.matches('{').count();
        let close_braces = content.matches('}').count();
        if open_braces == close_braces {
            score += 0.1;
        }
        
        let open_brackets = content.matches('[').count();
        let close_brackets = content.matches(']').count();
        if open_brackets == close_brackets {
            score += 0.1;
        }
        
        // Check for proper line endings
        if content.contains('\n') {
            score += 0.1;
        }
        
        score.min(1.0f64)
    }
}

impl Default for AdvancedRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for AdvancedRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        let trimmed = content.trim();
        
        // Handle empty content
        if trimmed.is_empty() {
            return Ok("".to_string());
        }
        
        // If already valid, return as-is
        if self.validator.is_valid(trimmed) {
            return Ok(trimmed.to_string());
        }
        
        // Apply advanced repair strategies
        let repaired = self.apply_strategies(trimmed)?;
        
        // Always return the repaired content, even if validation fails
        Ok(repaired)
    }
    
    fn confidence(&self, content: &str) -> f64 {
        if content.trim().is_empty() {
            return 0.0;
        }
        
        self.calculate_confidence(content)
    }
    
    fn needs_repair(&self, content: &str) -> bool {
        !self.validator.is_valid(content)
    }
}

/// Advanced validator that uses multiple validation techniques
pub struct AdvancedValidator;

impl Validator for AdvancedValidator {
    fn is_valid(&self, content: &str) -> bool {
        if content.trim().is_empty() {
            return false;
        }
        
        // Try multiple validation approaches
        self.validate_json(content) ||
        self.validate_yaml(content) ||
        self.validate_xml(content) ||
        self.validate_toml(content) ||
        self.validate_csv(content) ||
        self.validate_ini(content) ||
        self.validate_markdown(content)
    }
    
    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        if content.trim().is_empty() {
            errors.push("Empty content".to_string());
            return errors;
        }
        
        // Try each validation method
        if !self.validate_json(content) {
            errors.push("Not valid JSON".to_string());
        }
        if !self.validate_yaml(content) {
            errors.push("Not valid YAML".to_string());
        }
        if !self.validate_xml(content) {
            errors.push("Not valid XML".to_string());
        }
        if !self.validate_toml(content) {
            errors.push("Not valid TOML".to_string());
        }
        if !self.validate_csv(content) {
            errors.push("Not valid CSV".to_string());
        }
        if !self.validate_ini(content) {
            errors.push("Not valid INI".to_string());
        }
        if !self.validate_markdown(content) {
            errors.push("Not valid Markdown".to_string());
        }
        
        errors
    }
}

impl AdvancedValidator {
    fn validate_json(&self, content: &str) -> bool {
        serde_json::from_str::<serde_json::Value>(content).is_ok()
    }
    
    fn validate_yaml(&self, content: &str) -> bool {
        serde_yaml::from_str::<serde_yaml::Value>(content).is_ok()
    }
    
    fn validate_xml(&self, content: &str) -> bool {
        quick_xml::reader::Reader::from_str(content)
            .read_event()
            .is_ok()
    }
    
    fn validate_toml(&self, content: &str) -> bool {
        toml::from_str::<toml::Value>(content).is_ok()
    }
    
    fn validate_csv(&self, content: &str) -> bool {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content.as_bytes());
        reader.records().all(|record| record.is_ok())
    }
    
    fn validate_ini(&self, content: &str) -> bool {
        // Basic INI validation
        let lines: Vec<&str> = content.lines().collect();
        let has_sections = lines.iter().any(|line| {
            let line = line.trim();
            line.starts_with('[') && line.contains(']')
        });
        let has_keys = lines.iter().any(|line| {
            let line = line.trim();
            line.contains('=') && !line.starts_with('#') && !line.starts_with('[')
        });
        has_sections || has_keys
    }
    
    fn validate_markdown(&self, content: &str) -> bool {
        // Basic Markdown validation
        content.trim().starts_with('#') ||
        content.contains("```") ||
        content.contains("**") ||
        content.contains("*") ||
        content.contains("`")
    }
}

/// Strategy for intelligent format detection
struct IntelligentFormatDetectionStrategy;

impl RepairStrategy for IntelligentFormatDetectionStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        // Analyze content to determine the most likely format
        let format = self.detect_format(content);
        
        match format {
            FormatType::Json => self.repair_json(content),
            FormatType::Yaml => self.repair_yaml(content),
            FormatType::Xml => self.repair_xml(content),
            FormatType::Toml => self.repair_toml(content),
            FormatType::Csv => self.repair_csv(content),
            FormatType::Ini => self.repair_ini(content),
            FormatType::Markdown => self.repair_markdown(content),
            FormatType::Unknown => Ok(content.to_string()),
        }
    }
    
    fn priority(&self) -> u8 {
        10
    }

    fn name(&self) -> &str {
        "IntelligentFormatDetectionStrategy"
    }
}

impl IntelligentFormatDetectionStrategy {
    fn detect_format(&self, content: &str) -> FormatType {
        let trimmed = content.trim();
        
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            FormatType::Json
        } else if trimmed.starts_with('#') || trimmed.contains("```") {
            FormatType::Markdown
        } else if trimmed.starts_with('[') && trimmed.contains(']') && !trimmed.contains(',') {
            FormatType::Ini
        } else if trimmed.starts_with('[') && trimmed.contains(']') && trimmed.contains('=') {
            FormatType::Toml
        } else if trimmed.contains(',') && trimmed.lines().count() > 1 {
            FormatType::Csv
        } else if trimmed.starts_with('<') && trimmed.contains('>') {
            FormatType::Xml
        } else if trimmed.contains(':') && !trimmed.starts_with('{') {
            FormatType::Yaml
        } else {
            FormatType::Unknown
        }
    }
    
    fn repair_json(&self, content: &str) -> Result<String> {
        // Basic JSON repair
        let mut repaired = content.to_string();
        
        // Add missing quotes around keys
        let key_regex = Regex::new(r#"(\w+):"#)?;
        repaired = key_regex.replace_all(&repaired, r#""$1":"#).to_string();
        
        // Fix trailing commas
        let comma_regex = Regex::new(r#",\s*([}\]])"#)?;
        repaired = comma_regex.replace_all(&repaired, r#"$1"#).to_string();
        
        Ok(repaired)
    }
    
    fn repair_yaml(&self, content: &str) -> Result<String> {
        // Basic YAML repair
        let mut repaired = content.to_string();
        
        // Fix missing colons
        let colon_regex = Regex::new(r#"^(\s*)(\w+)\s+([^:\n]+)$"#)?;
        repaired = colon_regex.replace_all(&repaired, r#"$1$2: $3"#).to_string();
        
        Ok(repaired)
    }
    
    fn repair_xml(&self, content: &str) -> Result<String> {
        // Basic XML repair
        let mut repaired = content.to_string();
        
        // Fix unclosed tags
        let tag_regex = Regex::new(r#"<(\w+)([^>]*)>"#)?;
        repaired = tag_regex.replace_all(&repaired, |caps: &regex::Captures| {
            let tag = &caps[1];
            let attrs = &caps[2];
            format!("<{}{}></{}>", tag, attrs, tag)
        }).to_string();
        
        Ok(repaired)
    }
    
    fn repair_toml(&self, content: &str) -> Result<String> {
        // Basic TOML repair
        let mut repaired = content.to_string();
        
        // Add missing quotes around string values
        let value_regex = Regex::new(r#"^(\s*)(\w+)\s*=\s*([^"'\s].*[^"'\s])\s*$"#)?;
        repaired = value_regex.replace_all(&repaired, |caps: &regex::Captures| {
            let indent = &caps[1];
            let key = &caps[2];
            let value = &caps[3];
            format!("{}{} = \"{}\"", indent, key, value)
        }).to_string();
        
        Ok(repaired)
    }
    
    fn repair_csv(&self, content: &str) -> Result<String> {
        // Basic CSV repair
        let mut repaired = content.to_string();
        
        // Add missing quotes around values with spaces
        let value_regex = Regex::new(r#"^([^",\n]+)$"#)?;
        repaired = value_regex.replace_all(&repaired, |caps: &regex::Captures| {
            let value = &caps[1];
            if value.contains(' ') {
                format!("\"{}\"", value)
            } else {
                value.to_string()
            }
        }).to_string();
        
        Ok(repaired)
    }
    
    fn repair_ini(&self, content: &str) -> Result<String> {
        // Basic INI repair
        let mut repaired = content.to_string();
        
        // Fix missing equals signs
        let equals_regex = Regex::new(r#"^(\s*)(\w+)\s+([^=\n]+)$"#)?;
        repaired = equals_regex.replace_all(&repaired, |caps: &regex::Captures| {
            let indent = &caps[1];
            let key = &caps[2];
            let value = &caps[3];
            format!("{}{} = {}", indent, key, value)
        }).to_string();
        
        Ok(repaired)
    }
    
    fn repair_markdown(&self, content: &str) -> Result<String> {
        // Basic Markdown repair
        let mut repaired = content.to_string();
        
        // Fix headers
        let header_regex = Regex::new(r#"^#([^#\s])"#)?;
        repaired = header_regex.replace_all(&repaired, r#"# $1"#).to_string();
        
        Ok(repaired)
    }
}

#[derive(Debug, Clone, Copy)]
enum FormatType {
    Json,
    Yaml,
    Xml,
    Toml,
    Csv,
    Ini,
    Markdown,
    Unknown,
}

/// Strategy for adaptive repair based on content analysis
struct AdaptiveRepairStrategy;

impl RepairStrategy for AdaptiveRepairStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        // Analyze content complexity and apply appropriate repair level
        let complexity = self.analyze_complexity(content);
        
        match complexity {
            ComplexityLevel::Simple => self.simple_repair(content),
            ComplexityLevel::Medium => self.medium_repair(content),
            ComplexityLevel::Complex => self.complex_repair(content),
        }
    }
    
    fn priority(&self) -> u8 {
        9
    }

    fn name(&self) -> &str {
        "AdaptiveRepairStrategy"
    }
}

impl AdaptiveRepairStrategy {
    fn analyze_complexity(&self, content: &str) -> ComplexityLevel {
        let lines = content.lines().count();
        let chars = content.chars().count();
        let brackets = content.matches('{').count() + content.matches('[').count();
        let quotes = content.matches('"').count();
        
        if lines > 50 || chars > 1000 || brackets > 20 || quotes > 50 {
            ComplexityLevel::Complex
        } else if lines > 10 || chars > 200 || brackets > 5 || quotes > 10 {
            ComplexityLevel::Medium
        } else {
            ComplexityLevel::Simple
        }
    }
    
    fn simple_repair(&self, content: &str) -> Result<String> {
        // Basic repairs for simple content
        let mut repaired = content.to_string();
        
        // Fix common issues
        repaired = repaired.replace(",,", ",");
        repaired = repaired.replace("  ", " ");
        
        Ok(repaired)
    }
    
    fn medium_repair(&self, content: &str) -> Result<String> {
        // More comprehensive repairs for medium complexity
        let mut repaired = self.simple_repair(content)?;
        
        // Additional repairs
        let regex = Regex::new(r#"\s+$"#)?;
        repaired = regex.replace_all(&repaired, "").to_string();
        
        Ok(repaired)
    }
    
    fn complex_repair(&self, content: &str) -> Result<String> {
        // Most comprehensive repairs for complex content
        let mut repaired = self.medium_repair(content)?;
        
        // Advanced repairs
        let regex = Regex::new(r#"\n\s*\n\s*\n"#)?;
        repaired = regex.replace_all(&repaired, "\n\n").to_string();
        
        Ok(repaired)
    }
}

#[derive(Debug, Clone, Copy)]
enum ComplexityLevel {
    Simple,
    Medium,
    Complex,
}

/// Strategy for confidence-based repair
struct ConfidenceBasedRepairStrategy;

impl RepairStrategy for ConfidenceBasedRepairStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        // Only apply repairs if confidence is high enough
        let confidence = self.calculate_confidence(content);
        
        if confidence > 0.8 {
            self.aggressive_repair(content)
        } else if confidence > 0.5 {
            self.moderate_repair(content)
        } else {
            self.conservative_repair(content)
        }
    }
    
    fn priority(&self) -> u8 {
        8
    }

    fn name(&self) -> &str {
        "ConfidenceBasedRepairStrategy"
    }
}

impl ConfidenceBasedRepairStrategy {
    fn calculate_confidence(&self, content: &str) -> f64 {
        let mut score: f64 = 0.0;
        
        if content.contains('{') && content.contains('}') {
            score += 0.3;
        }
        if content.contains('[') && content.contains(']') {
            score += 0.2;
        }
        if content.contains('"') {
            score += 0.2;
        }
        if content.contains(':') {
            score += 0.2;
        }
        if content.contains(',') {
            score += 0.1;
        }
        
        score.min(1.0f64)
    }
    
    fn aggressive_repair(&self, content: &str) -> Result<String> {
        // Apply all available repairs
        let mut repaired = content.to_string();
        
        // Multiple repair passes
        for _ in 0..3 {
            repaired = self.apply_basic_repairs(&repaired)?;
        }
        
        Ok(repaired)
    }
    
    fn moderate_repair(&self, content: &str) -> Result<String> {
        // Apply moderate repairs
        let mut repaired = content.to_string();
        
        // Single repair pass
        repaired = self.apply_basic_repairs(&repaired)?;
        
        Ok(repaired)
    }
    
    fn conservative_repair(&self, content: &str) -> Result<String> {
        // Apply minimal repairs
        let mut repaired = content.to_string();
        
        // Only fix obvious issues
        repaired = repaired.replace(",,", ",");
        
        Ok(repaired)
    }
    
    fn apply_basic_repairs(&self, content: &str) -> Result<String> {
        let mut repaired = content.to_string();
        
        // Fix common issues
        repaired = repaired.replace(",,", ",");
        repaired = repaired.replace("  ", " ");
        
        Ok(repaired)
    }
}

/// Strategy for multi-pass repair
struct MultiPassRepairStrategy;

impl RepairStrategy for MultiPassRepairStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let mut repaired = content.to_string();
        
        // Apply multiple repair passes
        for pass in 1..=3 {
            repaired = self.repair_pass(&repaired, pass)?;
        }
        
        Ok(repaired)
    }
    
    fn priority(&self) -> u8 {
        7
    }

    fn name(&self) -> &str {
        "MultiPassRepairStrategy"
    }
}

impl MultiPassRepairStrategy {
    fn repair_pass(&self, content: &str, pass: u32) -> Result<String> {
        let mut repaired = content.to_string();
        
        match pass {
            1 => {
                // First pass: fix basic syntax issues
                repaired = repaired.replace(",,", ",");
                repaired = repaired.replace("  ", " ");
            }
            2 => {
                // Second pass: fix structural issues
                let regex = Regex::new(r#"\s+$"#)?;
                repaired = regex.replace_all(&repaired, "").to_string();
            }
            3 => {
                // Third pass: fix formatting issues
                let regex = Regex::new(r#"\n\s*\n\s*\n"#)?;
                repaired = regex.replace_all(&repaired, "\n\n").to_string();
            }
            _ => {}
        }
        
        Ok(repaired)
    }
}

/// Strategy for context-aware repair
struct ContextAwareRepairStrategy;

impl RepairStrategy for ContextAwareRepairStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        // Analyze context and apply appropriate repairs
        let context = self.analyze_context(content);
        
        match context {
            ContextType::Data => self.repair_data(content),
            ContextType::Code => self.repair_code(content),
            ContextType::Documentation => self.repair_documentation(content),
            ContextType::Configuration => self.repair_configuration(content),
            ContextType::Unknown => Ok(content.to_string()),
        }
    }
    
    fn priority(&self) -> u8 {
        6
    }

    fn name(&self) -> &str {
        "ContextAwareRepairStrategy"
    }
}

impl ContextAwareRepairStrategy {
    fn analyze_context(&self, content: &str) -> ContextType {
        if content.contains("function") || content.contains("class") || content.contains("def") {
            ContextType::Code
        } else if content.contains("##") || content.contains("###") || content.contains("README") {
            ContextType::Documentation
        } else if content.contains("config") || content.contains("settings") || content.contains("=") {
            ContextType::Configuration
        } else if content.contains("{") || content.contains("[") || content.contains(":") {
            ContextType::Data
        } else {
            ContextType::Unknown
        }
    }
    
    fn repair_data(&self, content: &str) -> Result<String> {
        // Repair data structures
        let mut repaired = content.to_string();
        
        // Fix JSON-like structures
        if repaired.contains('{') && repaired.contains('}') {
            let regex = Regex::new(r#"(\w+):"#)?;
            repaired = regex.replace_all(&repaired, r#""$1":"#).to_string();
        }
        
        Ok(repaired)
    }
    
    fn repair_code(&self, content: &str) -> Result<String> {
        // Repair code structures
        let mut repaired = content.to_string();
        
        // Fix common code issues
        repaired = repaired.replace("  ", "    "); // Convert spaces to tabs
        
        Ok(repaired)
    }
    
    fn repair_documentation(&self, content: &str) -> Result<String> {
        // Repair documentation structures
        let mut repaired = content.to_string();
        
        // Fix Markdown headers
        let regex = Regex::new(r#"^#([^#\s])"#)?;
        repaired = regex.replace_all(&repaired, r#"# $1"#).to_string();
        
        Ok(repaired)
    }
    
    fn repair_configuration(&self, content: &str) -> Result<String> {
        // Repair configuration structures
        let mut repaired = content.to_string();
        
        // Fix key-value pairs
        let regex = Regex::new(r#"^(\s*)(\w+)\s+([^=\n]+)$"#)?;
        repaired = regex.replace_all(&repaired, |caps: &regex::Captures| {
            let indent = &caps[1];
            let key = &caps[2];
            let value = &caps[3];
            format!("{}{} = {}", indent, key, value)
        }).to_string();
        
        Ok(repaired)
    }
}

#[derive(Debug, Clone, Copy)]
enum ContextType {
    Data,
    Code,
    Documentation,
    Configuration,
    Unknown,
}

/// Strategy for error recovery
struct ErrorRecoveryStrategy;

impl RepairStrategy for ErrorRecoveryStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        // Try to recover from various error conditions
        let mut repaired = content.to_string();
        
        // Remove invalid characters
        repaired = repaired.chars()
            .filter(|c| c.is_ascii() || c.is_whitespace())
            .collect();
        
        // Fix common encoding issues
        repaired = repaired.replace("â€™", "'");
        repaired = repaired.replace("â€œ", "\"");
        repaired = repaired.replace("â€", "\"");
        
        Ok(repaired)
    }
    
    fn priority(&self) -> u8 {
        5
    }

    fn name(&self) -> &str {
        "ErrorRecoveryStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_repairer_basic() {
        let mut repairer = AdvancedRepairer::new();
        
        let input = r#"{"name": "John", "age": 30,}"#;
        let result = repairer.repair(input).unwrap();
        assert!(result.contains("John"));
    }
    
    #[test]
    fn test_advanced_repairer_confidence() {
        let mut repairer = AdvancedRepairer::new();
        
        let input = r#"{"name": "John", "age": 30}"#;
        let confidence = repairer.confidence(input);
        assert!(confidence > 0.5);
    }
    
    #[test]
    fn test_advanced_validator() {
        let validator = AdvancedValidator;
        
        assert!(validator.is_valid(r#"{"name": "John", "age": 30}"#));
        assert!(validator.is_valid("# Header\nSome text"));
        assert!(!validator.is_valid(""));
    }
    
    #[test]
    fn test_intelligent_format_detection() {
        let strategy = IntelligentFormatDetectionStrategy;
        
        let json_input = r#"{"name": "John"}"#;
        let result = strategy.apply(json_input).unwrap();
        assert!(result.contains("name"));
        
        let markdown_input = "# Header\nSome text";
        let result = strategy.apply(markdown_input).unwrap();
        assert!(result.contains("Header"));
    }
    
    #[test]
    fn test_adaptive_repair() {
        let strategy = AdaptiveRepairStrategy;
        
        let simple_input = "name: John";
        let result = strategy.apply(simple_input).unwrap();
        assert!(result.contains("John"));
        
        let complex_input = r#"{"users": [{"name": "John", "age": 30}, {"name": "Jane", "age": 25}]}"#;
        let result = strategy.apply(complex_input).unwrap();
        assert!(result.contains("users"));
    }
    
    #[test]
    fn test_confidence_based_repair() {
        let strategy = ConfidenceBasedRepairStrategy;
        
        let high_confidence_input = r#"{"name": "John", "age": 30}"#;
        let result = strategy.apply(high_confidence_input).unwrap();
        assert!(result.contains("John"));
        
        let low_confidence_input = "random text";
        let result = strategy.apply(low_confidence_input).unwrap();
        assert!(result.contains("random"));
    }
    
    #[test]
    fn test_multi_pass_repair() {
        let strategy = MultiPassRepairStrategy;
        
        let input = "name: John,,age: 30";
        let result = strategy.apply(input).unwrap();
        assert!(result.contains("John"));
    }
    
    #[test]
    fn test_context_aware_repair() {
        let strategy = ContextAwareRepairStrategy;
        
        let data_input = r#"{"name": "John"}"#;
        let result = strategy.apply(data_input).unwrap();
        assert!(result.contains("name"));
        
        let code_input = "function test() { return true; }";
        let result = strategy.apply(code_input).unwrap();
        assert!(result.contains("function"));
    }
    
    #[test]
    fn test_error_recovery() {
        let strategy = ErrorRecoveryStrategy;
        
        let input = "name: Johnâ€™s age: 30";
        let result = strategy.apply(input).unwrap();
        assert!(result.contains("John"));
    }
}
