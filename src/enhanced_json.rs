use crate::context_parser::{ContextAwareStringParser, ContextStack, ParseContext};
use crate::error::{RepairError, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

/// Enhanced JSON repairer with advanced capabilities inspired by json_repair-main
pub struct EnhancedJsonRepairer {
    skip_json_loads: bool,
    logging: bool,
    stream_stable: bool,
    repair_log: Vec<String>,
}

impl EnhancedJsonRepairer {
    pub fn new() -> Self {
        Self {
            skip_json_loads: false,
            logging: false,
            stream_stable: false,
            repair_log: Vec::new(),
        }
    }

    pub fn with_skip_json_loads(mut self, skip: bool) -> Self {
        self.skip_json_loads = skip;
        self
    }

    pub fn with_logging(mut self, logging: bool) -> Self {
        self.logging = logging;
        self
    }

    pub fn with_stream_stable(mut self, stable: bool) -> Self {
        self.stream_stable = stable;
        self
    }

    /// Repair JSON string with advanced parsing
    pub fn repair_json(&mut self, json_str: &str) -> Result<String> {
        self.repair_log.clear();
        
        if !self.skip_json_loads {
            // Try standard JSON parsing first
            if let Ok(_) = serde_json::from_str::<Value>(json_str) {
                if self.logging {
                    self.repair_log.push("JSON was already valid, no repairs needed".to_string());
                }
                return Ok(json_str.to_string());
            }
        }

        // Use advanced parsing
        self.parse_json_advanced(json_str)
    }

    /// Repair JSON and return parsed object
    pub fn repair_json_objects(&mut self, json_str: &str) -> Result<Value> {
        let repaired = self.repair_json(json_str)?;
        serde_json::from_str(&repaired)
            .map_err(|e| RepairError::json_repair(format!("Failed to parse repaired JSON: {}", e)))
    }

    /// Drop-in replacement for json.loads()
    pub fn loads(&mut self, json_str: &str) -> Result<Value> {
        self.repair_json_objects(json_str)
    }

    /// Drop-in replacement for json.load() with file descriptor
    pub fn load<R: Read>(&mut self, reader: R) -> Result<Value> {
        let mut content = String::new();
        let mut reader = BufReader::new(reader);
        reader.read_to_string(&mut content)
            .map_err(|e| RepairError::generic(format!("Failed to read file: {}", e)))?;
        
        self.loads(&content)
    }

    /// Load JSON from file by filename
    pub fn from_file(&mut self, filename: &str) -> Result<Value> {
        let file = File::open(filename)
            .map_err(|e| RepairError::generic(format!("Failed to open file {}: {}", filename, e)))?;
        self.load(file)
    }

    /// Get repair log
    pub fn get_repair_log(&self) -> &[String] {
        &self.repair_log
    }

    fn parse_json_advanced(&mut self, json_str: &str) -> Result<String> {
        let mut context = ContextStack::new();
        let mut parser = ContextAwareStringParser::new(json_str.to_string(), self.logging);
        
        // Determine initial context
        let trimmed = json_str.trim();
        if trimmed.starts_with('{') {
            context.push(ParseContext::Object);
        } else if trimmed.starts_with('[') {
            context.push(ParseContext::Array);
        } else {
            context.push(ParseContext::Root);
        }

        let result = self.parse_value(&mut parser, &mut context)?;
        
        // Convert back to JSON string
        serde_json::to_string(&result)
            .map_err(|e| RepairError::Generic(format!("Failed to serialize result: {}", e)))
    }

    fn parse_value(&mut self, parser: &mut ContextAwareStringParser, context: &mut ContextStack) -> Result<Value> {
        parser.skip_whitespace();
        
        match parser.get_char_at(0) {
            Some('{') => self.parse_object(parser, context),
            Some('[') => self.parse_array(parser, context),
            Some('"') | Some('\'') => self.parse_string(parser, context),
            Some(ch) if ch.is_ascii_digit() || ch == '-' => self.parse_number(parser),
            Some('t') | Some('f') | Some('n') => self.parse_boolean_or_null(parser),
            Some('#') | Some('/') => self.parse_comment(parser),
            _ => Err(RepairError::Generic("Unexpected character".to_string())),
        }
    }

    fn parse_object(&mut self, parser: &mut ContextAwareStringParser, context: &mut ContextStack) -> Result<Value> {
        context.push(ParseContext::Object);
        parser.advance(1); // Skip opening brace
        
        let mut obj = HashMap::new();
        parser.skip_whitespace();
        
        while let Some(ch) = parser.get_char_at(0) {
            if ch == '}' {
                break;
            }
            
            // Parse key
            context.push(ParseContext::ObjectKey);
            let key = parser.parse_string();
            context.pop();
            
            if key.is_empty() {
                parser.advance(1);
                continue;
            }
            
            // Skip colon
            parser.skip_whitespace();
            if parser.get_char_at(0) == Some(':') {
                parser.advance(1);
            }
            
            // Parse value
            context.push(ParseContext::ObjectValue);
            let value = self.parse_value(parser, context)?;
            context.pop();
            
            obj.insert(key, value);
            
            // Skip comma
            parser.skip_whitespace();
            if parser.get_char_at(0) == Some(',') {
                parser.advance(1);
            }
            parser.skip_whitespace();
        }
        
        parser.advance(1); // Skip closing brace
        context.pop();
        
        Ok(Value::Object(obj.into_iter().collect()))
    }

    fn parse_array(&mut self, parser: &mut ContextAwareStringParser, context: &mut ContextStack) -> Result<Value> {
        context.push(ParseContext::Array);
        parser.advance(1); // Skip opening bracket
        
        let mut arr = Vec::new();
        parser.skip_whitespace();
        
        while let Some(ch) = parser.get_char_at(0) {
            if ch == ']' {
                break;
            }
            
            let value = self.parse_value(parser, context)?;
            arr.push(value);
            
            // Skip comma
            parser.skip_whitespace();
            if parser.get_char_at(0) == Some(',') {
                parser.advance(1);
            }
            parser.skip_whitespace();
        }
        
        parser.advance(1); // Skip closing bracket
        context.pop();
        
        Ok(Value::Array(arr))
    }

    fn parse_string(&mut self, parser: &mut ContextAwareStringParser, context: &mut ContextStack) -> Result<Value> {
        let value = parser.parse_string();
        Ok(Value::String(value))
    }

    fn parse_number(&mut self, parser: &mut ContextAwareStringParser) -> Result<Value> {
        let mut num_str = String::new();
        let mut ch = parser.get_char_at(0);
        
        // Handle negative sign
        if ch == Some('-') {
            num_str.push('-');
            parser.advance(1);
            ch = parser.get_char_at(0);
        }
        
        // Parse integer part
        while let Some(c) = ch {
            if c.is_ascii_digit() {
                num_str.push(c);
                parser.advance(1);
                ch = parser.get_char_at(0);
            } else {
                break;
            }
        }
        
        // Parse decimal part
        if ch == Some('.') {
            num_str.push('.');
            parser.advance(1);
            ch = parser.get_char_at(0);
            
            while let Some(c) = ch {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    parser.advance(1);
                    ch = parser.get_char_at(0);
                } else {
                    break;
                }
            }
        }
        
        // Parse exponent
        if ch == Some('e') || ch == Some('E') {
            num_str.push(ch.unwrap());
            parser.advance(1);
            ch = parser.get_char_at(0);
            
            if ch == Some('+') || ch == Some('-') {
                num_str.push(ch.unwrap());
                parser.advance(1);
                ch = parser.get_char_at(0);
            }
            
            while let Some(c) = ch {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    parser.advance(1);
                    ch = parser.get_char_at(0);
                } else {
                    break;
                }
            }
        }
        
        // Try to parse as number
        if let Ok(int_val) = num_str.parse::<i64>() {
            Ok(Value::Number(int_val.into()))
        } else if let Ok(float_val) = num_str.parse::<f64>() {
            Ok(Value::Number(serde_json::Number::from_f64(float_val).unwrap()))
        } else {
            Err(RepairError::Generic(format!("Invalid number: {}", num_str)))
        }
    }

    fn parse_boolean_or_null(&mut self, parser: &mut ContextAwareStringParser) -> Result<Value> {
        let mut word = String::new();
        let mut ch = parser.get_char_at(0);
        
        while let Some(c) = ch {
            if c.is_alphabetic() {
                word.push(c);
                parser.advance(1);
                ch = parser.get_char_at(0);
            } else {
                break;
            }
        }
        
        match word.to_lowercase().as_str() {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            "null" => Ok(Value::Null),
            _ => Err(RepairError::Generic(format!("Invalid literal: {}", word))),
        }
    }

    fn parse_comment(&mut self, parser: &mut ContextAwareStringParser) -> Result<Value> {
        // Skip comment
        while let Some(ch) = parser.get_char_at(0) {
            if ch == '\n' {
                break;
            }
            parser.advance(1);
        }
        
        // Try to parse the next value
        self.parse_value(parser, &mut ContextStack::new())
    }
}

impl Default for EnhancedJsonRepairer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_json_repair() {
        let mut repairer = EnhancedJsonRepairer::new().with_logging(true);
        
        // Test basic repair with trailing comma (invalid JSON)
        let input = r#"{"name": "John", "age": 30,}"#;
        let result = repairer.repair_json(input).unwrap();
        assert!(result.contains("John"));
        
        // Verify the result is valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["name"], "John");
        assert_eq!(parsed["age"], 30);
    }

    #[test]
    fn test_loads_method() {
        let mut repairer = EnhancedJsonRepairer::new();
        
        let input = r#"{"name": "John", "age": 30}"#;
        let result: Value = repairer.loads(input).unwrap();
        
        assert_eq!(result["name"], "John");
        assert_eq!(result["age"], 30);
    }

    #[test]
    fn test_skip_json_loads() {
        let mut repairer = EnhancedJsonRepairer::new().with_skip_json_loads(true);
        
        let input = r#"{"name": "John", "age": 30}"#;
        let result = repairer.repair_json(input).unwrap();
        assert!(result.contains("John"));
    }

    #[test]
    fn test_stream_stable() {
        let mut repairer = EnhancedJsonRepairer::new().with_stream_stable(true);
        
        // Test with partial JSON
        let input = r#"{"name": "John", "age": 30"#;
        let result = repairer.repair_json(input).unwrap();
        assert!(result.contains("John"));
    }
}
