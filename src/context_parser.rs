use std::collections::HashMap;

/// Context for parsing JSON strings
#[derive(Debug, Clone, PartialEq)]
pub enum ParseContext {
    Object,
    ObjectKey,
    ObjectValue,
    Array,
    Root,
}

/// Context stack for nested parsing
#[derive(Debug, Clone)]
pub struct ContextStack {
    stack: Vec<ParseContext>,
}

impl ContextStack {
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    pub fn push(&mut self, context: ParseContext) {
        self.stack.push(context);
    }

    pub fn pop(&mut self) -> Option<ParseContext> {
        self.stack.pop()
    }

    pub fn current(&self) -> Option<&ParseContext> {
        self.stack.last()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn contains(&self, context: &ParseContext) -> bool {
        self.stack.contains(context)
    }

    pub fn reset(&mut self) {
        self.stack.clear();
    }
}

/// Advanced string parser with context awareness
pub struct ContextAwareStringParser {
    content: String,
    index: usize,
    context: ContextStack,
    logging: bool,
    log: Vec<String>,
}

impl ContextAwareStringParser {
    pub fn new(content: String, logging: bool) -> Self {
        Self {
            content,
            index: 0,
            context: ContextStack::new(),
            logging,
            log: Vec::new(),
        }
    }

    pub fn with_context(mut self, context: ParseContext) -> Self {
        self.context.push(context);
        self
    }

    fn log(&mut self, message: &str) {
        if self.logging {
            let context = self.get_context_window(10);
            self.log.push(format!("{} | Context: {}", message, context));
        }
    }

    fn get_context_window(&self, window: usize) -> String {
        let start = self.index.saturating_sub(window);
        let end = (self.index + window).min(self.content.len());
        self.content[start..end].to_string()
    }

    pub fn get_char_at(&self, offset: usize) -> Option<char> {
        self.content.chars().nth(self.index + offset)
    }

    pub fn advance(&mut self, count: usize) {
        self.index += count;
    }

    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.get_char_at(0) {
            if ch.is_whitespace() {
                self.advance(1);
            } else {
                break;
            }
        }
    }

    fn skip_to_character(&mut self, characters: &[char]) -> Option<char> {
        while let Some(ch) = self.get_char_at(0) {
            if characters.contains(&ch) {
                return Some(ch);
            }
            self.advance(1);
        }
        None
    }

    /// Parse a string with advanced context awareness
    pub fn parse_string(&mut self) -> String {
        let mut result = String::new();
        let mut missing_quotes = false;
        let mut lstring_delimiter = '"';
        let mut rstring_delimiter = '"';

        // Skip leading whitespace
        self.skip_whitespace();

        let mut char = self.get_char_at(0);
        
        // Handle comments
        if let Some(ch) = char {
            if ch == '#' || ch == '/' {
                return self.parse_comment();
            }
        }

        // Skip non-alphanumeric characters at start
        while let Some(ch) = char {
            if !ch.is_alphanumeric() && ch != '"' && ch != '\'' {
                self.advance(1);
                char = self.get_char_at(0);
            } else {
                break;
            }
        }

        char = self.get_char_at(0);
        if char.is_none() {
            return String::new();
        }

        // Determine string delimiters
        if let Some(ch) = char {
            match ch {
                '\'' => {
                    lstring_delimiter = '\'';
                    rstring_delimiter = '\'';
                }
                '"' => {
                    lstring_delimiter = '"';
                    rstring_delimiter = '"';
                }
                '"' => {
                    lstring_delimiter = '"';
                    rstring_delimiter = '"';
                }
                _ if ch.is_alphanumeric() => {
                    // This could be a boolean/null or unquoted string
                    if let Some(context) = self.context.current() {
                        if matches!(context, ParseContext::ObjectKey) && ch.is_alphabetic() {
                            self.log("Found unquoted string in object key context");
                            missing_quotes = true;
                        } else if ch == 't' || ch == 'f' || ch == 'n' {
                            // Try parsing as boolean/null
                            if let Some(parsed) = self.try_parse_boolean_or_null() {
                                return parsed;
                            }
                            missing_quotes = true;
                        } else {
                            missing_quotes = true;
                        }
                    } else {
                        missing_quotes = true;
                    }
                }
                _ => {
                    missing_quotes = true;
                }
            }
        }

        if !missing_quotes {
            self.advance(1);
        }

        // Parse the string content
        let mut char = self.get_char_at(0);

        while let Some(ch) = char {
            if ch == rstring_delimiter {
                // Found potential end of string
                if self.is_valid_string_end() {
                    break;
                } else {
                    // This quote might be part of the content
                    result.push(ch);
                    self.advance(1);
                    char = self.get_char_at(0);
                    continue;
                }
            }

            // Handle escape sequences
            if ch == '\\' {
                if let Some(escaped) = self.handle_escape_sequence() {
                    result.push(escaped);
                    // handle_escape_sequence already advanced past the escape sequence
                    char = self.get_char_at(0);
                    continue;
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }

            self.advance(1);
            char = self.get_char_at(0);
        }

        // Handle missing closing quote
        if char.is_none() && missing_quotes {
            self.log("String missing closing quote, trimming whitespace");
            result = result.trim_end().to_string();
        } else if char.is_some() {
            self.advance(1); // Skip the closing quote
        }

        result
    }

    fn try_parse_boolean_or_null(&mut self) -> Option<String> {
        let start_index = self.index;
        let mut word = String::new();
        
        while let Some(ch) = self.get_char_at(0) {
            if ch.is_alphanumeric() {
                word.push(ch);
                self.advance(1);
            } else {
                break;
            }
        }

        match word.to_lowercase().as_str() {
            "true" | "false" | "null" => Some(word),
            _ => {
                // Rollback
                self.index = start_index;
                None
            }
        }
    }

    fn parse_comment(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.get_char_at(0) {
            if ch == '\n' {
                break;
            }
            result.push(ch);
            self.advance(1);
        }
        result
    }

    fn handle_escape_sequence(&mut self) -> Option<char> {
        self.advance(1); // Skip the backslash
        let ch = self.get_char_at(0)?;
        self.advance(1);

        match ch {
            't' => Some('\t'),
            'n' => Some('\n'),
            'r' => Some('\r'),
            'b' => Some('\u{0008}'),
            'f' => Some('\u{000C}'),
            '\\' => Some('\\'),
            '"' => Some('"'),
            '\'' => Some('\''),
            'u' => self.parse_unicode_escape(),
            'x' => self.parse_hex_escape(),
            _ => Some(ch), // Unknown escape, keep as-is
        }
    }

    fn parse_unicode_escape(&mut self) -> Option<char> {
        let mut hex = String::new();
        for _ in 0..4 {
            if let Some(ch) = self.get_char_at(0) {
                if ch.is_ascii_hexdigit() {
                    hex.push(ch);
                    self.advance(1);
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        
        if let Ok(code) = u32::from_str_radix(&hex, 16) {
            char::from_u32(code)
        } else {
            None
        }
    }

    fn parse_hex_escape(&mut self) -> Option<char> {
        let mut hex = String::new();
        for _ in 0..2 {
            if let Some(ch) = self.get_char_at(0) {
                if ch.is_ascii_hexdigit() {
                    hex.push(ch);
                    self.advance(1);
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        
        if let Ok(code) = u32::from_str_radix(&hex, 16) {
            char::from_u32(code)
        } else {
            None
        }
    }

    fn is_valid_string_end(&self) -> bool {
        // Check if this is a valid end of string based on context
        if let Some(context) = self.context.current() {
            match context {
                ParseContext::ObjectKey => {
                    // Look for colon after this quote
                    let mut i = 1;
                    while let Some(ch) = self.get_char_at(i) {
                        if ch.is_whitespace() {
                            i += 1;
                        } else if ch == ':' {
                            return true;
                        } else {
                            return false;
                        }
                    }
                    false
                }
                ParseContext::ObjectValue => {
                    // Look for comma or closing brace
                    let mut i = 1;
                    while let Some(ch) = self.get_char_at(i) {
                        if ch.is_whitespace() {
                            i += 1;
                        } else if ch == ',' || ch == '}' {
                            return true;
                        } else {
                            return false;
                        }
                    }
                    // End of input is valid
                    true
                }
                ParseContext::Array => {
                    // Look for comma or closing bracket
                    let mut i = 1;
                    while let Some(ch) = self.get_char_at(i) {
                        if ch.is_whitespace() {
                            i += 1;
                        } else if ch == ',' || ch == ']' {
                            return true;
                        } else {
                            return false;
                        }
                    }
                    false
                }
                _ => true,
            }
        } else {
            true
        }
    }

    pub fn get_log(&self) -> Vec<String> {
        self.log.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_aware_string_parsing() {
        let content = r#"{"key": "value with \"quotes\" inside"}"#.to_string();
        let mut parser = ContextAwareStringParser::new(content, true)
            .with_context(ParseContext::ObjectKey);
        
        let result = parser.parse_string();
        assert_eq!(result, "key");
    }

    #[test]
    fn test_escape_sequence_handling() {
        // Test escape sequence parsing with simple content
        let content = r#""test\nline""#.to_string();
        let mut parser = ContextAwareStringParser::new(content, false)
            .with_context(ParseContext::ObjectValue);
        
        let result = parser.parse_string();
        // The parser should convert \n to newline
        assert_eq!(result, "test\nline");
    }

    #[test]
    fn test_unicode_escape() {
        let content = "\"\\u263a\"".to_string();
        let mut parser = ContextAwareStringParser::new(content, false)
            .with_context(ParseContext::ObjectValue);
        
        let result = parser.parse_string();
        assert_eq!(result, "☺");
    }
}