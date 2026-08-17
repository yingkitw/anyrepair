//! XML repair module

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};

/// Find all opening tags matching `<(\w+)([^>]*)>` in order.
///
/// Returns `(tag_name, attributes)` for each non-overlapping leftmost match.
/// Closing tags (`</...>`), declarations (`<?...?>`), and CDATA (`<![...]>`)
/// do not start with a word character and are therefore skipped.
fn find_open_tags(content: &str) -> Vec<(String, String)> {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        if chars[i] == '<' && i + 1 < n && is_word_char(chars[i + 1]) {
            // \w+ (tag name)
            let name_start = i + 1;
            let mut j = name_start;
            while j < n && is_word_char(chars[j]) {
                j += 1;
            }
            // [^>]* (attributes)
            let attrs_start = j;
            while j < n && chars[j] != '>' {
                j += 1;
            }
            if j < n && chars[j] == '>' {
                let name: String = chars[name_start..attrs_start].iter().collect();
                let attrs: String = chars[attrs_start..j].iter().collect();
                out.push((name, attrs));
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

#[inline]
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Quote unquoted XML attribute values.
///
/// Mirrors the regex `(\w+)=([^"'\s>]+)` → `name="value"`. A word run
/// followed by `=` and a run of chars that are not quotes, whitespace, or
/// `>` gets wrapped in double quotes.
fn quote_attributes(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;
    while i < n {
        if is_word_char(chars[i]) && (i == 0 || !is_word_char(chars[i - 1]))
            && let Some((name, value, end)) = match_attr(&chars, i) {
                out.push_str(&name);
                out.push_str("=\"");
                out.push_str(&value);
                out.push('"');
                i = end;
                continue;
            }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Match `(\w+)=([^"'\s>]+)` starting at a word-run start `i`.
fn match_attr(chars: &[char], i: usize) -> Option<(String, String, usize)> {
    let n = chars.len();
    let mut j = i;
    while j < n && is_word_char(chars[j]) {
        j += 1;
    }
    let name: String = chars[i..j].iter().collect();
    if j >= n || chars[j] != '=' {
        return None;
    }
    let vstart = j + 1;
    if vstart >= n {
        return None;
    }
    // [^"'\s>]+
    let vchar = |c: char| c != '"' && c != '\'' && !c.is_whitespace() && c != '>';
    if !vchar(chars[vstart]) {
        return None;
    }
    let mut k = vstart;
    while k < n && vchar(chars[k]) {
        k += 1;
    }
    let value: String = chars[vstart..k].iter().collect();
    Some((name, value, k))
}

/// XML repairer that can fix common XML issues
///
/// Uses trait-based composition with GenericRepairer for better modularity
pub struct XmlRepairer {
    pub inner: crate::repairer_base::GenericRepairer,
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
        xml_structure_valid(content)
    }

    fn validate(&self, content: &str) -> Vec<String> {
        if content.trim().is_empty() {
            return vec!["Empty XML content".to_string()];
        }
        if xml_structure_valid(content) {
            vec![]
        } else {
            vec!["XML structure validation failed".to_string()]
        }
    }
}

fn xml_structure_valid(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return false;
    }

    let mut stack: Vec<String> = Vec::new();
    let mut rest = trimmed;
    while let Some(start) = rest.find('<') {
        let Some(rel_end) = rest[start..].find('>') else {
            return false;
        };
        let tag_inner = rest[start + 1..start + rel_end].trim();
        rest = &rest[start + rel_end + 1..];

        if tag_inner.is_empty() || tag_inner.starts_with('?') || tag_inner.starts_with('!') {
            continue;
        }

        // Reject unquoted attribute values inside tags (e.g., <tag attr=value>)
        if tag_inner.contains('=')
            && !tag_inner.contains('"')
            && !tag_inner.contains('\'')
        {
            return false;
        }

        let self_closing = tag_inner.ends_with('/');
        let inner = tag_inner.trim_end_matches('/').trim();
        if let Some(stripped) = inner.strip_prefix('/') {
            let name = stripped.split_whitespace().next().unwrap_or("");
            match stack.pop() {
                Some(open) if open == name => {}
                _ => return false,
            }
        } else {
            let name = inner.split_whitespace().next().unwrap_or("").to_string();
            if name.is_empty() {
                return false;
            }
            if !self_closing {
                stack.push(name);
            }
        }
    }

    stack.is_empty()
}

/// Strategy to fix unclosed tags
struct FixUnclosedTagsStrategy;

impl RepairStrategy for FixUnclosedTagsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();
        let mut open_tags = Vec::new();

        // Find all opening tags
        for (tag_name, attributes) in find_open_tags(&result) {
            // Check if it's a self-closing tag
            if attributes.ends_with('/') {
                continue;
            }

            // Check if it's a closing tag
            if let Some(stripped) = tag_name.strip_prefix('/') {
                if let Some(expected_tag) = open_tags.pop()
                    && expected_tag != stripped {
                        // Mismatched closing tag
                        open_tags.push(expected_tag);
                    }
            } else {
                open_tags.push(tag_name);
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
        Ok(quote_attributes(content))
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
        let mut result = String::with_capacity(content.len() * 2);
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '&' {
                let mut entity = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '#' {
                        entity.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if chars.peek() == Some(&';') && !entity.is_empty() {
                    // Existing entity reference — preserve it
                    result.push('&');
                    result.push_str(&entity);
                    result.push(';');
                    chars.next(); // consume ';'
                } else {
                    // Bare ampersand — escape it
                    result.push_str("&amp;");
                    result.push_str(&entity);
                }
            } else {
                result.push(ch);
            }
        }

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
        Ok(quote_attributes(content))
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
        // The original regex `<(\w+)([^>]*)/>` reconstructed each self-closing
        // tag verbatim from its captures, making this a no-op. Preserve that
        // behavior by returning the content unchanged.
        Ok(content.to_string())
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
