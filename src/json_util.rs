//! Minimal JSON helpers (validation, escaping, MCP payloads) without serde.

/// Escape a string for use inside a JSON string literal.
pub fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Return true if `content` is valid JSON.
pub fn is_valid_json(content: &str) -> bool {
    parse_json_value(content.trim()).is_ok()
}

/// Validation errors for invalid JSON (empty if valid).
pub fn validate_json_errors(content: &str) -> Vec<String> {
    match parse_json_value(content.trim()) {
        Ok(()) => vec![],
        Err(e) => vec![e],
    }
}

/// Parsed fields from an MCP tool input object.
pub struct ToolCallInput {
    pub content: Option<String>,
    pub format: Option<String>,
}

/// Read a string field from a JSON object (e.g. `"repaired"` from an MCP response).
pub fn get_json_string_field(json: &str, key: &str) -> Option<String> {
    extract_object_string_field(json, key).ok().flatten()
}

/// Read a boolean field from a JSON object.
pub fn get_json_bool_field(json: &str, key: &str) -> Option<bool> {
    let raw = extract_object_value_field(json, key).ok().flatten()?;
    match raw.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Read a number field from a JSON object.
pub fn get_json_number_field(json: &str, key: &str) -> Option<f64> {
    let raw = extract_object_value_field(json, key).ok().flatten()?;
    raw.trim().parse().ok()
}

/// Build a `{"content":"..."}` JSON string for MCP tool input.
pub fn tool_input_json(content: &str) -> String {
    format!(r#"{{"content":{}}}"#, json_string(content))
}

/// Build a `{"content":"...","format":"..."}` JSON string for MCP validate input.
pub fn validate_input_json(content: &str, format: &str) -> String {
    format!(
        r#"{{"content":{},"format":{}}}"#,
        json_string(content),
        json_string(format)
    )
}

/// Parse an MCP tool call input JSON object into `ToolCallInput`.
pub fn parse_tool_call_input(json: &str) -> Result<ToolCallInput, String> {
    let trimmed = json.trim();
    if !trimmed.starts_with('{') {
        return Err("expected JSON object".to_string());
    }
    Ok(ToolCallInput {
        content: extract_object_string_field(trimmed, "content")?,
        format: extract_object_string_field(trimmed, "format")?,
    })
}

/// Parse `{"tool":"...","input":{...}}` for the MCP binary.
pub fn parse_mcp_request_line(line: &str) -> Result<(String, String), String> {
    let trimmed = line.trim();
    let tool = extract_object_string_field(trimmed, "tool")?
        .ok_or_else(|| "missing 'tool' field".to_string())?;
    let input = extract_object_value_field(trimmed, "input")?
        .ok_or_else(|| "missing 'input' field".to_string())?;
    Ok((tool, input))
}

/// Build a `{"repaired":"...","success":true}` MCP success response.
pub fn repair_success_response(repaired: &str) -> String {
    format!(
        r#"{{"repaired":{},"success":true}}"#,
        json_string(repaired)
    )
}

/// Build a `{"repaired":"...","confidence":N,"success":true}` MCP response.
pub fn repair_format_response(repaired: &str, confidence: f64) -> String {
    format!(
        r#"{{"repaired":{},"confidence":{},"success":true}}"#,
        json_string(repaired),
        confidence
    )
}

/// Build a `{"valid":bool,"format":"..."}` MCP validate response.
pub fn validate_response(valid: bool, format: &str) -> String {
    format!(
        r#"{{"valid":{},"format":{}}}"#,
        if valid { "true" } else { "false" },
        json_string(format)
    )
}

fn extract_object_string_field(json: &str, key: &str) -> Result<Option<String>, String> {
    let bytes = json.trim().as_bytes();
    if bytes.first() != Some(&b'{') {
        return Err("expected JSON object".to_string());
    }
    let mut i = 1;
    i = skip_whitespace(bytes, i);
    if i < bytes.len() && bytes[i] == b'}' {
        return Ok(None);
    }
    loop {
        i = skip_whitespace(bytes, i);
        if i >= bytes.len() || bytes[i] != b'"' {
            return Err(format!("missing field '{}'", key));
        }
        let key_start = i;
        let key_end = parse_string(bytes, i)?;
        let field_key = parse_json_string(
            std::str::from_utf8(&bytes[key_start..key_end])
                .map_err(|_| "invalid UTF-8 in JSON key".to_string())?,
        )?;

        i = key_end;
        i = skip_whitespace(bytes, i);
        if i >= bytes.len() || bytes[i] != b':' {
            return Err(format!("invalid field '{}'", key));
        }
        i += 1;
        let value_start = skip_whitespace(bytes, i);

        if field_key == key {
            let value_end = parse_value(bytes, value_start)?;
            let raw = std::str::from_utf8(&bytes[value_start..value_end])
                .map_err(|_| "invalid UTF-8 in JSON value".to_string())?;
            let raw = raw.trim();
            if raw == "null" {
                return Ok(None);
            }
            if !raw.starts_with('"') {
                return Err(format!("field '{}' must be a JSON string", key));
            }
            return parse_json_string(raw).map(Some);
        }

        i = parse_value(bytes, value_start)?;
        i = skip_whitespace(bytes, i);
        if i >= bytes.len() {
            return Err(format!("missing field '{}'", key));
        }
        match bytes[i] {
            b'}' => return Ok(None),
            b',' => i += 1,
            _ => return Err(format!("invalid object after field '{}'", field_key)),
        }
    }
}

fn extract_object_value_field(json: &str, key: &str) -> Result<Option<String>, String> {
    let bytes = json.trim().as_bytes();
    if bytes.first() != Some(&b'{') {
        return Err("expected JSON object".to_string());
    }
    let mut i = 1;
    i = skip_whitespace(bytes, i);
    if i < bytes.len() && bytes[i] == b'}' {
        return Ok(None);
    }
    loop {
        i = skip_whitespace(bytes, i);
        if i >= bytes.len() || bytes[i] != b'"' {
            return Err(format!("missing field '{}'", key));
        }
        let key_start = i;
        let key_end = parse_string(bytes, i)?;
        let field_key = parse_json_string(
            std::str::from_utf8(&bytes[key_start..key_end])
                .map_err(|_| "invalid UTF-8 in JSON key".to_string())?,
        )?;

        i = key_end;

        i = skip_whitespace(bytes, i);
        if i >= bytes.len() || bytes[i] != b':' {
            return Err(format!("invalid field '{}'", key));
        }
        i += 1;
        let value_start = skip_whitespace(bytes, i);

        if field_key == key {
            let value_end = parse_value(bytes, value_start)?;
            return Ok(Some(
                std::str::from_utf8(&bytes[value_start..value_end])
                    .map_err(|_| "invalid UTF-8 in JSON value".to_string())?
                    .to_string(),
            ));
        }

        i = parse_value(bytes, value_start)?;
        i = skip_whitespace(bytes, i);
        if i >= bytes.len() {
            return Err(format!("missing field '{}'", key));
        }
        match bytes[i] {
            b'}' => return Ok(None),
            b',' => i += 1,
            _ => return Err(format!("invalid object after field '{}'", field_key)),
        }
    }
}

fn parse_json_string(s: &str) -> Result<String, String> {
    let s = s.trim();
    if !s.starts_with('"') {
        return Err("expected JSON string".to_string());
    }
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    chars.next();
    loop {
        match chars.next() {
            None => return Err("unterminated JSON string".to_string()),
            Some('"') => break,
            Some('\\') => match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('/') => out.push('/'),
                Some('b') => out.push('\u{0008}'),
                Some('f') => out.push('\u{000C}'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('u') => {
                    let hex: String = chars.by_ref().take(4).collect();
                    if hex.len() != 4 {
                        return Err("invalid unicode escape".to_string());
                    }
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|_| "invalid unicode escape".to_string())?;
                    let ch = char::from_u32(code)
                        .ok_or_else(|| "invalid unicode value".to_string())?;
                    out.push(ch);
                }
                Some(c) => return Err(format!("invalid escape \\{}", c)),
                None => return Err("unterminated escape".to_string()),
            },
            Some(c) => out.push(c),
        }
    }
    Ok(out)
}

fn skip_whitespace(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

fn parse_value(bytes: &[u8], mut i: usize) -> Result<usize, String> {
    i = skip_whitespace(bytes, i);
    if i >= bytes.len() {
        return Err("unexpected end of JSON".to_string());
    }
    match bytes[i] {
        b'"' => parse_string(bytes, i),
        b'{' => parse_object(bytes, i),
        b'[' => parse_array(bytes, i),
        b't' if bytes[i..].starts_with(b"true") => Ok(i + 4),
        b'f' if bytes[i..].starts_with(b"false") => Ok(i + 5),
        b'n' if bytes[i..].starts_with(b"null") => Ok(i + 4),
        b'-' | b'0'..=b'9' => parse_number(bytes, i),
        _ => Err("invalid JSON token".to_string()),
    }
}

fn parse_string(bytes: &[u8], mut i: usize) -> Result<usize, String> {
    i += 1;
    let mut escape = false;
    while i < bytes.len() {
        let b = bytes[i];
        if escape {
            escape = false;
            i += 1;
            continue;
        }
        if b == b'\\' {
            escape = true;
            i += 1;
            continue;
        }
        if b == b'"' {
            return Ok(i + 1);
        }
        i += 1;
    }
    Err("unterminated string".to_string())
}

fn parse_number(bytes: &[u8], mut i: usize) -> Result<usize, String> {
    if bytes[i] == b'-' {
        i += 1;
    }
    if i >= bytes.len() {
        return Err("invalid number".to_string());
    }
    if bytes[i] == b'0' {
        i += 1;
    } else if bytes[i].is_ascii_digit() {
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    } else {
        return Err("invalid number".to_string());
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        if i >= bytes.len() || !bytes[i].is_ascii_digit() {
            return Err("invalid number".to_string());
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
            i += 1;
        }
        if i >= bytes.len() || !bytes[i].is_ascii_digit() {
            return Err("invalid number".to_string());
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    Ok(i)
}

fn parse_array(bytes: &[u8], mut i: usize) -> Result<usize, String> {
    i += 1;
    i = skip_whitespace(bytes, i);
    if i < bytes.len() && bytes[i] == b']' {
        return Ok(i + 1);
    }
    loop {
        i = parse_value(bytes, i)?;
        i = skip_whitespace(bytes, i);
        if i >= bytes.len() {
            return Err("unterminated array".to_string());
        }
        match bytes[i] {
            b']' => return Ok(i + 1),
            b',' => {
                i += 1;
                i = skip_whitespace(bytes, i);
                if i < bytes.len() && bytes[i] == b']' {
                    return Err("trailing comma in array".to_string());
                }
            }
            _ => return Err("expected ',' or ']' in array".to_string()),
        }
    }
}

fn parse_object(bytes: &[u8], mut i: usize) -> Result<usize, String> {
    i += 1;
    i = skip_whitespace(bytes, i);
    if i < bytes.len() && bytes[i] == b'}' {
        return Ok(i + 1);
    }
    loop {
        i = skip_whitespace(bytes, i);
        if i >= bytes.len() || bytes[i] != b'"' {
            return Err("expected string key in object".to_string());
        }
        i = parse_string(bytes, i)?;
        i = skip_whitespace(bytes, i);
        if i >= bytes.len() || bytes[i] != b':' {
            return Err("expected ':' after key".to_string());
        }
        i += 1;
        i = parse_value(bytes, i)?;
        i = skip_whitespace(bytes, i);
        if i >= bytes.len() {
            return Err("unterminated object".to_string());
        }
        match bytes[i] {
            b'}' => return Ok(i + 1),
            b',' => {
                i += 1;
                i = skip_whitespace(bytes, i);
                if i < bytes.len() && bytes[i] == b'}' {
                    return Err("trailing comma in object".to_string());
                }
            }
            _ => return Err("expected ',' or '}' in object".to_string()),
        }
    }
}

fn parse_json_value(s: &str) -> Result<(), String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty JSON".to_string());
    }
    let end = parse_value(s.as_bytes(), 0)?;
    if skip_whitespace(s.as_bytes(), end) != s.len() {
        return Err("trailing characters".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_json_object() {
        assert!(is_valid_json(r#"{"a":1}"#));
    }

    #[test]
    fn invalid_trailing_comma() {
        assert!(!is_valid_json(r#"{"a":1,}"#));
    }

    #[test]
    fn parse_tool_input_content() {
        let input = parse_tool_call_input(r#"{"content":"hello"}"#).unwrap();
        assert_eq!(input.content.as_deref(), Some("hello"));
    }
}
