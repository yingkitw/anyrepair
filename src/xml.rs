//! XML repair module

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use regex::Regex;
use std::sync::OnceLock;

/// Cached regex patterns for XML performance optimization
#[allow(dead_code)]
struct XmlRegexCache {
    unclosed_tags: Regex,
    malformed_attributes: Regex,
    invalid_characters: Regex,
    missing_quotes: Regex,
    self_closing_tags: Regex,
}

impl XmlRegexCache {
    fn new() -> Result<Self> {
        Ok(Self {
            unclosed_tags: Regex::new(r"<(\w+)([^>]*)>")?,
            malformed_attributes: Regex::new(r#"(\w+)=([^"'\s>]+)"#)?,
            invalid_characters: Regex::new(r"[<>&]")?,
            missing_quotes: Regex::new(r#"(\w+)=([^"'\s>]+)"#)?,
            self_closing_tags: Regex::new(r"<(\w+)([^>]*)/>")?,
        })
    }
}

static XML_REGEX_CACHE: OnceLock<XmlRegexCache> = OnceLock::new();

fn get_xml_regex_cache() -> &'static XmlRegexCache {
    XML_REGEX_CACHE
        .get_or_init(|| XmlRegexCache::new().expect("Failed to initialize XML regex cache"))
}

/// XML repairer that can fix common XML issues
///
/// Uses trait-based composition with GenericRepairer for better modularity
pub struct XmlRepairer {
    inner: crate::repairer_base::GenericRepairer,
}

impl XmlRepairer {
    /// Create a new XML repairer
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(FixUnclosedTagsStrategy),
            Box::new(FixMalformedAttributesStrategy),
            Box::new(FixInvalidCharactersStrategy),
            Box::new(FixMissingQuotesStrategy),
            Box::new(FixSelfClosingTagsStrategy),
            Box::new(AddXmlDeclarationStrategy),
        ];

        let validator: Box<dyn Validator> = Box::new(XmlValidator);
        let inner = crate::repairer_base::GenericRepairer::new(validator, strategies);

        Self { inner }
    }
}

impl Default for XmlRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for XmlRepairer {
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

        // Calculate confidence based on XML-like patterns
        let mut score: f64 = 0.0;

        // Check for XML declaration
        if content.trim().starts_with("<?xml") {
            score += 0.3;
        }

        // Check for opening tags
        if content.contains('<') && content.contains('>') {
            score += 0.3;
        }

        // Check for proper tag structure
        let open_tags = content.matches('<').count();
        let close_tags = content.matches('>').count();
        if open_tags == close_tags {
            score += 0.2;
        }

        // Check for attributes
        if content.contains('=') {
            score += 0.1;
        }

        // Check for content between tags
        if content.contains("</") {
            score += 0.1;
        }

        score.min(1.0)
    }
}

/// XML validator
pub struct XmlValidator;

impl Validator for XmlValidator {
    fn is_valid(&self, content: &str) -> bool {
        if content.trim().is_empty() {
            return false;
        }

        // Check for missing quotes around attributes
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check for attributes without quotes (e.g., id=1 instead of id="1")
            if trimmed.contains('=') && !trimmed.contains('"') {
                return false;
            }
        }

        // Basic XML validation using quick-xml
        quick_xml::Reader::from_str(content).read_event().is_ok()
    }

    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();

        if content.trim().is_empty() {
            errors.push("Empty XML content".to_string());
            return errors;
        }

        // Try to parse with quick-xml
        let mut reader = quick_xml::Reader::from_str(content);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(_) => continue,
                Err(e) => {
                    errors.push(format!("XML parsing error: {e}"));
                    break;
                }
            }
        }

        errors
    }
}

/// Strategy to fix unclosed tags
struct FixUnclosedTagsStrategy;

impl RepairStrategy for FixUnclosedTagsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_xml_regex_cache();
        let mut result = content.to_string();
        let mut open_tags = Vec::new();

        // Find all opening tags
        for cap in cache.unclosed_tags.captures_iter(&result) {
            let tag_name = &cap[1];
            let attributes = &cap[2];

            // Check if it's a self-closing tag
            if attributes.ends_with('/') {
                continue;
            }

            // Check if it's a closing tag
            if tag_name.starts_with('/') {
                if let Some(expected_tag) = open_tags.pop() {
                    if expected_tag != tag_name[1..] {
                        // Mismatched closing tag
                        open_tags.push(expected_tag);
                    }
                }
            } else {
                open_tags.push(tag_name.to_string());
            }
        }

        // Close any remaining open tags
        for tag in open_tags.iter().rev() {
            result.push_str(&format!("</{tag}>"));
        }

        Ok(result)
    }

    fn priority(&self) -> u8 {
        6
    }

    fn name(&self) -> &str {
        "FixUnclosedTagsStrategy"
    }
}

/// Strategy to fix malformed attributes
struct FixMalformedAttributesStrategy;

impl RepairStrategy for FixMalformedAttributesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_xml_regex_cache();
        let result = cache
            .malformed_attributes
            .replace_all(content, |caps: &regex::Captures| {
                let attr_name = &caps[1];
                let attr_value = &caps[2];
                format!("{attr_name}=\"{attr_value}\"")
            });

        Ok(result.to_string())
    }

    fn priority(&self) -> u8 {
        5
    }

    fn name(&self) -> &str {
        "FixMalformedAttributesStrategy"
    }
}

/// Strategy to fix invalid characters
struct FixInvalidCharactersStrategy;

impl RepairStrategy for FixInvalidCharactersStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();

        // Replace invalid XML characters
        result = result.replace('&', "&amp;");
        result = result.replace('<', "&lt;");
        result = result.replace('>', "&gt;");

        // But restore tags
        result = result.replace("&lt;", "<");
        result = result.replace("&gt;", ">");

        Ok(result)
    }

    fn priority(&self) -> u8 {
        4
    }

    fn name(&self) -> &str {
        "FixInvalidCharactersStrategy"
    }
}

/// Strategy to fix missing quotes around attribute values
struct FixMissingQuotesStrategy;

impl RepairStrategy for FixMissingQuotesStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_xml_regex_cache();
        let result = cache
            .missing_quotes
            .replace_all(content, |caps: &regex::Captures| {
                let attr_name = &caps[1];
                let attr_value = &caps[2];
                format!("{attr_name}=\"{attr_value}\"")
            });

        Ok(result.to_string())
    }

    fn priority(&self) -> u8 {
        3
    }

    fn name(&self) -> &str {
        "FixMissingQuotesStrategy"
    }
}

/// Strategy to fix self-closing tags
struct FixSelfClosingTagsStrategy;

impl RepairStrategy for FixSelfClosingTagsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let cache = get_xml_regex_cache();
        let result = cache
            .self_closing_tags
            .replace_all(content, |caps: &regex::Captures| {
                let tag_name = &caps[1];
                let attributes = &caps[2];
                format!("<{tag_name}{attributes}/>")
            });

        Ok(result.to_string())
    }

    fn priority(&self) -> u8 {
        2
    }

    fn name(&self) -> &str {
        "FixSelfClosingTagsStrategy"
    }
}

/// Strategy to add XML declaration
struct AddXmlDeclarationStrategy;

impl RepairStrategy for AddXmlDeclarationStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let trimmed = content.trim();

        if !trimmed.starts_with("<?xml") {
            Ok(format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{trimmed}"
            ))
        } else {
            Ok(trimmed.to_string())
        }
    }

    fn priority(&self) -> u8 {
        1
    }

    fn name(&self) -> &str {
        "AddXmlDeclarationStrategy"
    }
}
