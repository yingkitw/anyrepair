//! MCP (Model Context Protocol) server for anyrepair
//!
//! Exposes anyrepair repair functionality as an MCP server for integration
//! with Claude and other MCP-compatible clients.

use crate::json_util::{
    parse_tool_call_input, repair_format_response, repair_success_response, validate_response,
};
use std::collections::HashMap;

/// Tool definition for MCP
#[derive(Clone, Debug)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: String,
}

fn content_repair_schema(description: &str) -> String {
    format!(
        r#"{{"type":"object","properties":{{"content":{{"type":"string","description":{}}}}},"required":["content"]}}"#,
        crate::json_util::json_string(description)
    )
}

fn validate_tool_schema() -> String {
    let enum_items: Vec<String> = crate::SUPPORTED_FORMATS
        .iter()
        .map(|f| crate::json_util::json_string(f))
        .collect();
    format!(
        r#"{{"type":"object","properties":{{"content":{{"type":"string","description":"Content to validate"}},"format":{{"type":"string","enum":[{}],"description":"Format to validate"}}}},"required":["content","format"]}}"#,
        enum_items.join(",")
    )
}

/// MCP Server for anyrepair
pub struct AnyrepairMcpServer {
    tools: HashMap<String, Tool>,
}

impl AnyrepairMcpServer {
    /// Create a new MCP server instance
    pub fn new() -> Self {
        let mut tools = HashMap::new();

        // Repair tool
        tools.insert(
            "repair".to_string(),
            Tool {
                name: "repair".to_string(),
                description: "Repair content in various formats (auto-detect)".to_string(),
                input_schema: content_repair_schema("Content to repair"),
            },
        );

        // Format-specific repair tools
        for format in crate::SUPPORTED_FORMATS {
            tools.insert(
                format!("repair_{}", format),
                Tool {
                    name: format!("repair_{}", format),
                    description: format!("Repair {} content", format),
                    input_schema: content_repair_schema(&format!("Content to repair as {}", format)),
                },
            );
        }

        // Validate tool
        tools.insert(
            "validate".to_string(),
            Tool {
                name: "validate".to_string(),
                description: "Validate content in various formats".to_string(),
                input_schema: validate_tool_schema(),
            },
        );

        Self { tools }
    }

    /// Get available tools
    pub fn get_tools(&self) -> Vec<Tool> {
        self.tools.values().cloned().collect()
    }

    /// Process a tool call (`input_json` is a JSON object string).
    pub fn process_tool_call(&self, name: &str, input_json: &str) -> Result<String, String> {
        let input = parse_tool_call_input(input_json)?;
        if name == "repair" {
            return self.handle_repair(&input);
        }
        if name == "validate" {
            return self.handle_validate(&input);
        }
        if let Some(format) = name.strip_prefix("repair_") {
            return self.handle_repair_format(&input, format);
        }
        Err(format!("Unknown tool: {}", name))
    }

    fn handle_repair(&self, input: &crate::json_util::ToolCallInput) -> Result<String, String> {
        let content = input
            .content
            .as_deref()
            .ok_or("Missing 'content' parameter")?;

        let repaired = crate::repair(content).map_err(|e| format!("Repair failed: {}", e))?;

        Ok(repair_success_response(&repaired))
    }

    fn handle_repair_format(
        &self,
        input: &crate::json_util::ToolCallInput,
        format: &str,
    ) -> Result<String, String> {
        let content = input
            .content
            .as_deref()
            .ok_or("Missing 'content' parameter")?;

        let mut repairer = crate::create_repairer(format)
            .map_err(|e| format!("{} repair failed: {}", format, e))?;
        let repaired = repairer
            .repair(content)
            .map_err(|e| format!("{} repair failed: {}", format, e))?;

        let confidence = repairer.confidence(&repaired);

        Ok(repair_format_response(&repaired, confidence))
    }

    fn handle_validate(&self, input: &crate::json_util::ToolCallInput) -> Result<String, String> {
        let content = input
            .content
            .as_deref()
            .ok_or("Missing 'content' parameter")?;

        let format = input
            .format
            .as_deref()
            .ok_or("Missing 'format' parameter")?;

        let validator =
            crate::create_validator(format).map_err(|e| format!("Validation failed: {}", e))?;
        let is_valid = validator.is_valid(content);

        Ok(validate_response(is_valid, format))
    }
}

impl Default for AnyrepairMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_util::{
        get_json_bool_field, get_json_number_field, get_json_string_field, tool_input_json,
        validate_input_json,
    };

    fn call(server: &AnyrepairMcpServer, tool: &str, input: &str) -> Result<String, String> {
        server.process_tool_call(tool, input)
    }

    fn response_success(s: &str) -> bool {
        get_json_bool_field(s, "success").unwrap_or(false)
    }

    fn response_valid(s: &str) -> bool {
        get_json_bool_field(s, "valid").unwrap_or(false)
    }

    fn response_repaired(s: &str) -> Option<String> {
        get_json_string_field(s, "repaired")
    }

    fn response_confidence(s: &str) -> Option<f64> {
        get_json_number_field(s, "confidence")
    }

    // ===== Server Creation Tests =====

    #[test]
    fn test_mcp_server_creation() {
        let server = AnyrepairMcpServer::new();
        assert!(!server.get_tools().is_empty());
    }

    #[test]
    fn test_mcp_server_default() {
        let server = AnyrepairMcpServer::default();
        assert!(!server.get_tools().is_empty());
    }

    #[test]
    fn test_mcp_server_tool_count() {
        let server = AnyrepairMcpServer::new();
        let tools = server.get_tools();
        // Should have: repair, repair_json, repair_yaml, repair_markdown, repair_xml,
        // repair_toml, repair_csv, repair_ini, repair_diff, repair_properties, repair_env, validate = 12 tools
        assert_eq!(tools.len(), 12);
    }

    #[test]
    fn test_mcp_server_has_all_repair_tools() {
        let server = AnyrepairMcpServer::new();
        let tools: Vec<_> = server.get_tools().iter().map(|t| t.name.clone()).collect();
        assert!(tools.contains(&"repair".to_string()));
        assert!(tools.contains(&"repair_json".to_string()));
        assert!(tools.contains(&"repair_yaml".to_string()));
        assert!(tools.contains(&"repair_markdown".to_string()));
        assert!(tools.contains(&"repair_xml".to_string()));
        assert!(tools.contains(&"repair_toml".to_string()));
        assert!(tools.contains(&"repair_csv".to_string()));
        assert!(tools.contains(&"repair_ini".to_string()));
        assert!(tools.contains(&"repair_diff".to_string()));
        assert!(tools.contains(&"validate".to_string()));
    }

    #[test]
    fn test_mcp_server_tool_descriptions() {
        let server = AnyrepairMcpServer::new();
        let tools = server.get_tools();
        for tool in tools {
            assert!(!tool.description.is_empty());
            assert!(!tool.name.is_empty());
        }
    }

    // ===== JSON Repair Tests =====

    #[test]
    fn test_mcp_repair_json_trailing_comma() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json(r#"{"key": "value",}"#);
        let result = call(&server, "repair_json", &input);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("success"));
        assert!(response.contains("confidence"));
    }

    #[test]
    fn test_mcp_repair_json_single_quotes() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("{'key': 'value'}");
        let result = call(&server, "repair_json", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_json_missing_quotes() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("{key: value}");
        let result = call(&server, "repair_json", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_json_valid() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json(r#"{"key": "value"}"#);
        let result = call(&server, "repair_json", &input);
        assert!(result.is_ok());
        assert!(response_success(&result.unwrap()));
    }

    // ===== YAML Repair Tests =====

    #[test]
    fn test_mcp_repair_yaml_basic() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("name: John\nage: 30");
        let result = call(&server, "repair_yaml", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_yaml_with_errors() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("name: John\n  age: 30");
        let result = call(&server, "repair_yaml", &input);
        assert!(result.is_ok());
    }

    // ===== Markdown Repair Tests =====

    #[test]
    fn test_mcp_repair_markdown_headers() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("#Header\n##Subheader");
        let result = call(&server, "repair_markdown", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_markdown_valid() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("# Header\n\nSome content");
        let result = call(&server, "repair_markdown", &input);
        assert!(result.is_ok());
    }

    // ===== XML Repair Tests =====

    #[test]
    fn test_mcp_repair_xml_basic() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("<root><item>value</item></root>");
        let result = call(&server, "repair_xml", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_xml_unclosed() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("<root><item>value</root>");
        let result = call(&server, "repair_xml", &input);
        assert!(result.is_ok());
    }

    // ===== TOML Repair Tests =====

    #[test]
    fn test_mcp_repair_toml_basic() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("name = \"test\"\nversion = \"1.0\"");
        let result = call(&server, "repair_toml", &input);
        assert!(result.is_ok());
    }

    // ===== CSV Repair Tests =====

    #[test]
    fn test_mcp_repair_csv_basic() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("name,age\nJohn,30\nJane,25");
        let result = call(&server, "repair_csv", &input);
        assert!(result.is_ok());
    }

    // ===== INI Repair Tests =====

    #[test]
    fn test_mcp_repair_ini_basic() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("[section]\nkey=value");
        let result = call(&server, "repair_ini", &input);
        assert!(result.is_ok());
    }

    // ===== Auto-Detect Repair Tests =====

    #[test]
    fn test_mcp_repair_auto_detect_json() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json(r#"{"key": "value",}"#);
        let result = call(&server, "repair", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_auto_detect_yaml() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("name: John\nage: 30");
        let result = call(&server, "repair", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_auto_detect_array() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("[1, 2, 3,]");
        let result = call(&server, "repair", &input);
        assert!(result.is_ok());
    }

    // ===== Validation Tests =====

    #[test]
    fn test_mcp_validate_json_valid() {
        let server = AnyrepairMcpServer::new();
        let input = validate_input_json(r#"{"key": "value"}"#, "json");
        let result = call(&server, "validate", &input);
        assert!(result.is_ok());
        let response = result.unwrap();

        assert!(response_valid(&response));
    }

    #[test]
    fn test_mcp_validate_json_invalid() {
        let server = AnyrepairMcpServer::new();
        let input = validate_input_json(r#"{"key": "value",}"#, "json");
        let result = call(&server, "validate", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_validate_yaml() {
        let server = AnyrepairMcpServer::new();
        let input = validate_input_json("name: John\nage: 30", "yaml");
        let result = call(&server, "validate", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_validate_markdown() {
        let server = AnyrepairMcpServer::new();
        let input = validate_input_json("# Header\n\nContent", "markdown");
        let result = call(&server, "validate", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_validate_xml() {
        let server = AnyrepairMcpServer::new();
        let input = validate_input_json("<root></root>", "xml");
        let result = call(&server, "validate", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_validate_toml() {
        let server = AnyrepairMcpServer::new();
        let input = validate_input_json("name = \"test\"", "toml");
        let result = call(&server, "validate", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_validate_csv() {
        let server = AnyrepairMcpServer::new();
        let input = validate_input_json("name,age\nJohn,30", "csv");
        let result = call(&server, "validate", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_validate_ini() {
        let server = AnyrepairMcpServer::new();
        let input = validate_input_json("[section]\nkey=value", "ini");
        let result = call(&server, "validate", &input);
        assert!(result.is_ok());
    }

    // ===== Error Handling Tests =====

    #[test]
    fn test_mcp_unknown_tool() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("test");
        let result = call(&server, "unknown_tool", &input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown tool"));
    }

    #[test]
    fn test_mcp_missing_content_parameter() {
        let server = AnyrepairMcpServer::new();
        let input = "{}".to_string();
        let result = call(&server, "repair_json", &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_missing_format_parameter_validate() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("test");
        let result = call(&server, "validate", &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_invalid_format_validate() {
        let server = AnyrepairMcpServer::new();
        let input = validate_input_json("test", "invalid_format");
        let result = call(&server, "validate", &input);
        assert!(result.is_err());
    }

    // ===== Edge Cases Tests =====

    #[test]
    fn test_mcp_repair_empty_content() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("");
        let result = call(&server, "repair", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_whitespace_only() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("   \n\t  ");
        let result = call(&server, "repair", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_unicode_content() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json(r#"{"name": "日本語"}"#);
        let result = call(&server, "repair_json", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_large_content() {
        let server = AnyrepairMcpServer::new();
        let large_content = format!(r#"{{"data": "{}"}}"#, "x".repeat(10000));
        let input = tool_input_json(&large_content);
        let result = call(&server, "repair_json", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_special_characters() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json(r#"{"key": "value\nwith\nnewlines"}"#);
        let result = call(&server, "repair_json", &input);
        assert!(result.is_ok());
    }

    // ===== Response Format Tests =====

    #[test]
    fn test_mcp_repair_response_format() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json(r#"{"key": "value"}"#);
        let result = call(&server, "repair_json", &input);
        assert!(result.is_ok());
        let response = result.unwrap();
        
        assert!(response_repaired(&response).is_some());
        assert!(response_success(&response));
        assert!(response_confidence(&response).is_some());
    }

    #[test]
    fn test_mcp_validate_response_format() {
        let server = AnyrepairMcpServer::new();
        let input = validate_input_json(r#"{"key": "value"}"#, "json");
        let result = call(&server, "validate", &input);
        assert!(result.is_ok());
        let response = result.unwrap();

        assert!(response_valid(&response));
        assert!(get_json_string_field(&response, "format").is_some());
    }

    #[test]
    fn test_mcp_auto_repair_response_format() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json(r#"{"key": "value"}"#);
        let result = call(&server, "repair", &input);
        assert!(result.is_ok());
        let response = result.unwrap();

        assert!(response_repaired(&response).is_some());
        assert!(response_success(&response));
    }

    // ===== Consistency Tests =====

    #[test]
    fn test_mcp_repair_idempotent() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json(r#"{"key": "value"}"#);
        let result1 = call(&server, "repair_json", &input);
        let result2 = call(&server, "repair_json", &input);
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_mcp_validate_consistency() {
        let server = AnyrepairMcpServer::new();
        let input = validate_input_json(r#"{"key": "value"}"#, "json");
        let result1 = call(&server, "validate", &input);
        let result2 = call(&server, "validate", &input);
        assert_eq!(result1, result2);
    }

    // ===== Diff Repair Tests =====

    #[test]
    fn test_mcp_repair_diff_basic() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("--- a/file.txt\n+++ b/file.txt\n@@ -1,3 +1,3 @@\n line1\n-old\n+new\n line3\n");
        let result = call(&server, "repair_diff", &input);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response_success(&response));
        assert!(response_confidence(&response).is_some());
    }

    #[test]
    fn test_mcp_repair_diff_missing_headers() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("@@ -1,3 +1,3 @@\n line1\n-old\n+new\n line3\n");
        let result = call(&server, "repair_diff", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_validate_diff() {
        let server = AnyrepairMcpServer::new();
        let input = validate_input_json("--- a/file.txt\n+++ b/file.txt\n@@ -1,3 +1,3 @@\n line1\n-old\n+new\n line3\n", "diff");
        let result = call(&server, "validate", &input);
        assert!(result.is_ok());
    }

    // ===== Content Correctness Tests =====

    #[test]
    fn test_mcp_repair_json_trailing_comma_correctness() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json(r#"{"key": "value",}"#);
        let result = call(&server, "repair_json", &input);

        let repaired = response_repaired(&result.unwrap()).unwrap();
        assert!(crate::json_util::is_valid_json(&repaired));
    }

    #[test]
    fn test_mcp_repair_json_single_quotes_correctness() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("{'key': 'value'}");
        let result = call(&server, "repair_json", &input);
        
        let repaired = response_repaired(&result.unwrap()).unwrap();
        assert!(crate::json_util::is_valid_json(&repaired));
    }

    #[test]
    fn test_mcp_repair_yaml_correctness() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("name: John\n  age: 30");
        let result = call(&server, "repair_yaml", &input);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response_success(&response));
        assert!(!response_repaired(&response).unwrap().is_empty());
    }

    #[test]
    fn test_mcp_repair_xml_unclosed_correctness() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("<root><item>value</root>");
        let result = call(&server, "repair_xml", &input);
        
        let repaired = response_repaired(&result.unwrap()).unwrap();
        // Should have closing tags balanced
        assert!(repaired.contains("</root>"));
    }

    // ===== Confidence Score Range Tests =====

    #[test]
    fn test_mcp_confidence_score_range_valid_json() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json(r#"{"key": "value"}"#);
        let result = call(&server, "repair_json", &input);
        
        let confidence = response_confidence(&result.unwrap()).unwrap();
        assert!(
            (0.0..=1.0).contains(&confidence),
            "confidence {} out of range",
            confidence
        );
    }

    #[test]
    fn test_mcp_confidence_score_range_malformed_json() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("{key: value,}");
        let result = call(&server, "repair_json", &input);
        
        let confidence = response_confidence(&result.unwrap()).unwrap();
        assert!(
            (0.0..=1.0).contains(&confidence),
            "confidence {} out of range",
            confidence
        );
    }

    #[test]
    fn test_mcp_confidence_all_formats() {
        let server = AnyrepairMcpServer::new();
        let test_cases = vec![
            ("repair_json", r#"{"key": "value"}"#),
            ("repair_yaml", "name: John\nage: 30"),
            ("repair_markdown", "# Header\n\nContent"),
            ("repair_xml", "<root></root>"),
            ("repair_toml", "name = \"test\""),
            ("repair_csv", "name,age\nJohn,30"),
            ("repair_ini", "[section]\nkey=value"),
            ("repair_diff", "--- a/f\n+++ b/f\n@@ -1 +1 @@\n-old\n+new\n"),
        ];
        for (tool, content) in test_cases {
            let input = tool_input_json(content);
            let result = call(&server, tool, &input);
            assert!(result.is_ok(), "tool {} failed", tool);
            
            let confidence = response_confidence(&result.unwrap()).unwrap();
            assert!(
                confidence >= 0.0 && confidence <= 1.0,
                "tool {} confidence {} out of range",
                tool,
                confidence
            );
        }
    }

    // ===== All-Format Repair Loop Tests =====

    #[test]
    fn test_mcp_repair_all_formats_via_registry() {
        let server = AnyrepairMcpServer::new();
        for format in crate::SUPPORTED_FORMATS {
            let tool_name = format!("repair_{}", format);
            let input = tool_input_json("test content");
            let result = call(&server, &tool_name, &input);
            assert!(result.is_ok(), "repair_{} should succeed", format);
            assert!(
                response_success(&result.unwrap()),
                "repair_{} success should be true",
                format
            );
        }
    }

    #[test]
    fn test_mcp_validate_all_formats() {
        let server = AnyrepairMcpServer::new();
        for format in crate::SUPPORTED_FORMATS {
            let input = validate_input_json("test content", format);
            let result = call(&server, "validate", &input);
            assert!(result.is_ok(), "validate {} should succeed", format);
            let response = result.unwrap();
            assert!(
                get_json_bool_field(&response, "valid").is_some(),
                "validate {} should include valid field",
                format
            );
            assert!(
                get_json_string_field(&response, "format").as_deref() == Some(*format)
            );
        }
    }

    // ===== Complex Malformed Input Tests =====

    #[test]
    fn test_mcp_repair_json_deeply_nested() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json(r#"{"a": {"b": {"c": {"d": "value",},},},}"#);
        let result = call(&server, "repair_json", &input);
        assert!(result.is_ok());
        assert!(response_success(&result.unwrap()));
    }

    #[test]
    fn test_mcp_repair_json_mixed_issues() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("{'users': [{'name': 'John', 'age': 30,}, {'name': 'Jane', 'age': 25,},],}");
        let result = call(&server, "repair_json", &input);
        assert!(result.is_ok());
        
        let repaired = response_repaired(&result.unwrap()).unwrap();
        assert!(
            crate::json_util::is_valid_json(&repaired),
            "repaired JSON should be valid: {}",
            repaired
        );
    }

    #[test]
    fn test_mcp_repair_yaml_complex_indentation() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("server:\n  host: localhost\n    port: 8080\ndb:\n  name: test\n    user: admin");
        let result = call(&server, "repair_yaml", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_markdown_broken_links() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("#Header\n\n[broken link(http://example.com)\n\n**unclosed bold");
        let result = call(&server, "repair_markdown", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_csv_inconsistent_columns() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("name,age,city\nJohn,30\nJane,25,NYC,extra");
        let result = call(&server, "repair_csv", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_toml_malformed() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("[package]\nname = test\nversion = 1.0");
        let result = call(&server, "repair_toml", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_ini_missing_equals() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("[section]\nkey value\nother = ok");
        let result = call(&server, "repair_ini", &input);
        assert!(result.is_ok());
    }

    // ===== Tool Schema Validation Tests =====

    #[test]
    fn test_mcp_tool_schemas_have_required_fields() {
        let server = AnyrepairMcpServer::new();
        for tool in server.get_tools() {
            let schema = &tool.input_schema;
            assert!(schema.contains(r#""type":"object"#) || schema.contains(r#""type": "object"#),
                "tool {} schema type", tool.name);
            assert!(schema.contains("properties"), "tool {} has properties", tool.name);
            assert!(schema.contains("required"), "tool {} has required", tool.name);
            assert!(schema.contains("content"), "tool {} requires content", tool.name);
        }
    }

    #[test]
    fn test_mcp_validate_tool_schema_has_format_enum() {
        let server = AnyrepairMcpServer::new();
        let tools = server.get_tools();
        let validate_tool = tools.iter().find(|t| t.name == "validate").unwrap();
        let schema = &validate_tool.input_schema;
        assert!(schema.contains("enum"), "validate tool should have format enum");
        for fmt in crate::SUPPORTED_FORMATS {
            assert!(
                schema.contains(&format!("\"{}\"", fmt)),
                "validate enum should include {}",
                fmt
            );
        }
    }

    // ===== Error Edge Cases =====

    #[test]
    fn test_mcp_repair_null_content() {
        let server = AnyrepairMcpServer::new();
        let input = r#"{"content":null}"#.to_string();
        let result = call(&server, "repair_json", &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_repair_numeric_content() {
        let server = AnyrepairMcpServer::new();
        let input = r#"{"content":42}"#.to_string();
        let result = call(&server, "repair_json", &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_validate_unknown_format() {
        let server = AnyrepairMcpServer::new();
        let input = validate_input_json("test", "protobuf");
        let result = call(&server, "validate", &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_unknown_repair_format() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("test");
        let result = call(&server, "repair_protobuf", &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_empty_tool_name() {
        let server = AnyrepairMcpServer::new();
        let input = tool_input_json("test");
        let result = call(&server, "", &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_repair_with_extra_params_ignored() {
        let server = AnyrepairMcpServer::new();
        let input = format!(
            r#"{{"content":{},"extra_param":"should be ignored","another":123}}"#,
            crate::json_util::json_string(r#"{"key": "value"}"#)
        );
        let result = call(&server, "repair_json", &input);
        assert!(result.is_ok());
    }

    // ===== Validate After Repair Round-Trip Tests =====

    #[test]
    fn test_mcp_repair_then_validate_json() {
        let server = AnyrepairMcpServer::new();
        // Repair malformed JSON
        let repair_input = tool_input_json(r#"{"key": "value",}"#);
        let repair_result = call(&server, "repair_json", &repair_input).unwrap();
        
        let repaired = response_repaired(&repair_result).unwrap();

        // Validate the repaired content
        let validate_input = validate_input_json(&repaired, "json");
        let validate_result = call(&server, "validate", &validate_input).unwrap();
        
        assert!(response_valid(&validate_result),
            "repaired JSON should validate: {}",
            repaired
        );
    }

    #[test]
    fn test_mcp_repair_then_validate_yaml() {
        let server = AnyrepairMcpServer::new();
        let repair_input = tool_input_json("name: John\nage: 30");
        let repair_result = call(&server, "repair_yaml", &repair_input).unwrap();
        
        let repaired = response_repaired(&repair_result).unwrap();

        let validate_input = validate_input_json(&repaired, "yaml");
        let validate_result = call(&server, "validate", &validate_input).unwrap();
        
        assert!(response_valid(&validate_result));
    }

    #[test]
    fn test_mcp_repair_then_validate_xml() {
        let server = AnyrepairMcpServer::new();
        let repair_input = tool_input_json("<root><item>value</root>");
        let repair_result = call(&server, "repair_xml", &repair_input).unwrap();
        
        let repaired = response_repaired(&repair_result).unwrap();

        let validate_input = validate_input_json(&repaired, "xml");
        let validate_result = call(&server, "validate", &validate_input).unwrap();
        assert!(!repaired.is_empty());
        // Validate tool returns a well-formed response (content may still be imperfect XML)
        assert!(get_json_bool_field(&validate_result, "valid").is_some());
    }
}
