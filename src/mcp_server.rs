//! MCP (Model Context Protocol) server for anyrepair
//!
//! Exposes anyrepair repair functionality as an MCP server for integration
//! with Claude and other MCP-compatible clients.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Tool definition for MCP
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
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
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Content to repair"
                        }
                    },
                    "required": ["content"]
                }),
            },
        );

        // Format-specific repair tools
        for format in crate::SUPPORTED_FORMATS {
            tools.insert(
                format!("repair_{}", format),
                Tool {
                    name: format!("repair_{}", format),
                    description: format!("Repair {} content", format),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string",
                                "description": format!("Content to repair as {}", format)
                            }
                        },
                        "required": ["content"]
                    }),
                },
            );
        }

        // Validate tool
        tools.insert(
            "validate".to_string(),
            Tool {
                name: "validate".to_string(),
                description: "Validate content in various formats".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Content to validate"
                        },
                        "format": {
                            "type": "string",
                            "enum": crate::SUPPORTED_FORMATS,
                            "description": "Format to validate"
                        }
                    },
                    "required": ["content", "format"]
                }),
            },
        );

        Self { tools }
    }

    /// Get available tools
    pub fn get_tools(&self) -> Vec<Tool> {
        self.tools.values().cloned().collect()
    }

    /// Process a tool call
    pub fn process_tool_call(&self, name: &str, input: &Value) -> Result<String, String> {
        if name == "repair" {
            return self.handle_repair(input);
        }
        if name == "validate" {
            return self.handle_validate(input);
        }
        if let Some(format) = name.strip_prefix("repair_") {
            return self.handle_repair_format(input, format);
        }
        Err(format!("Unknown tool: {}", name))
    }

    fn handle_repair(&self, input: &Value) -> Result<String, String> {
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' parameter")?;

        let repaired = crate::repair(content).map_err(|e| format!("Repair failed: {}", e))?;

        Ok(json!({
            "repaired": repaired,
            "success": true
        })
        .to_string())
    }

    fn handle_repair_format(&self, input: &Value, format: &str) -> Result<String, String> {
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' parameter")?;

        let mut repairer = crate::create_repairer(format)
            .map_err(|e| format!("{} repair failed: {}", format, e))?;
        let repaired = repairer
            .repair(content)
            .map_err(|e| format!("{} repair failed: {}", format, e))?;

        let confidence = repairer.confidence(&repaired);

        Ok(json!({
            "repaired": repaired,
            "confidence": confidence,
            "success": true
        })
        .to_string())
    }

    fn handle_validate(&self, input: &Value) -> Result<String, String> {
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' parameter")?;

        let format = input
            .get("format")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'format' parameter")?;

        let validator =
            crate::create_validator(format).map_err(|e| format!("Validation failed: {}", e))?;
        let is_valid = validator.is_valid(content);

        Ok(json!({
            "valid": is_valid,
            "format": format
        })
        .to_string())
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
        let input = json!({
            "content": r#"{"key": "value",}"#
        });
        let result = server.process_tool_call("repair_json", &input);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("success"));
        assert!(response.contains("confidence"));
    }

    #[test]
    fn test_mcp_repair_json_single_quotes() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "{'key': 'value'}"
        });
        let result = server.process_tool_call("repair_json", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_json_missing_quotes() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "{key: value}"
        });
        let result = server.process_tool_call("repair_json", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_json_valid() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"key": "value"}"#
        });
        let result = server.process_tool_call("repair_json", &input);
        assert!(result.is_ok());
        let response = result.unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["success"], true);
    }

    // ===== YAML Repair Tests =====

    #[test]
    fn test_mcp_repair_yaml_basic() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "name: John\nage: 30"
        });
        let result = server.process_tool_call("repair_yaml", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_yaml_with_errors() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "name: John\n  age: 30"
        });
        let result = server.process_tool_call("repair_yaml", &input);
        assert!(result.is_ok());
    }

    // ===== Markdown Repair Tests =====

    #[test]
    fn test_mcp_repair_markdown_headers() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "#Header\n##Subheader"
        });
        let result = server.process_tool_call("repair_markdown", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_markdown_valid() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "# Header\n\nSome content"
        });
        let result = server.process_tool_call("repair_markdown", &input);
        assert!(result.is_ok());
    }

    // ===== XML Repair Tests =====

    #[test]
    fn test_mcp_repair_xml_basic() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "<root><item>value</item></root>"
        });
        let result = server.process_tool_call("repair_xml", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_xml_unclosed() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "<root><item>value</root>"
        });
        let result = server.process_tool_call("repair_xml", &input);
        assert!(result.is_ok());
    }

    // ===== TOML Repair Tests =====

    #[test]
    fn test_mcp_repair_toml_basic() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "name = \"test\"\nversion = \"1.0\""
        });
        let result = server.process_tool_call("repair_toml", &input);
        assert!(result.is_ok());
    }

    // ===== CSV Repair Tests =====

    #[test]
    fn test_mcp_repair_csv_basic() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "name,age\nJohn,30\nJane,25"
        });
        let result = server.process_tool_call("repair_csv", &input);
        assert!(result.is_ok());
    }

    // ===== INI Repair Tests =====

    #[test]
    fn test_mcp_repair_ini_basic() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "[section]\nkey=value"
        });
        let result = server.process_tool_call("repair_ini", &input);
        assert!(result.is_ok());
    }

    // ===== Auto-Detect Repair Tests =====

    #[test]
    fn test_mcp_repair_auto_detect_json() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"key": "value",}"#
        });
        let result = server.process_tool_call("repair", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_auto_detect_yaml() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "name: John\nage: 30"
        });
        let result = server.process_tool_call("repair", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_auto_detect_array() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "[1, 2, 3,]"
        });
        let result = server.process_tool_call("repair", &input);
        assert!(result.is_ok());
    }

    // ===== Validation Tests =====

    #[test]
    fn test_mcp_validate_json_valid() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"key": "value"}"#,
            "format": "json"
        });
        let result = server.process_tool_call("validate", &input);
        assert!(result.is_ok());
        let response = result.unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["valid"], true);
    }

    #[test]
    fn test_mcp_validate_json_invalid() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"key": "value",}"#,
            "format": "json"
        });
        let result = server.process_tool_call("validate", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_validate_yaml() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "name: John\nage: 30",
            "format": "yaml"
        });
        let result = server.process_tool_call("validate", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_validate_markdown() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "# Header\n\nContent",
            "format": "markdown"
        });
        let result = server.process_tool_call("validate", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_validate_xml() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "<root></root>",
            "format": "xml"
        });
        let result = server.process_tool_call("validate", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_validate_toml() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "name = \"test\"",
            "format": "toml"
        });
        let result = server.process_tool_call("validate", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_validate_csv() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "name,age\nJohn,30",
            "format": "csv"
        });
        let result = server.process_tool_call("validate", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_validate_ini() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "[section]\nkey=value",
            "format": "ini"
        });
        let result = server.process_tool_call("validate", &input);
        assert!(result.is_ok());
    }

    // ===== Error Handling Tests =====

    #[test]
    fn test_mcp_unknown_tool() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "test"
        });
        let result = server.process_tool_call("unknown_tool", &input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown tool"));
    }

    #[test]
    fn test_mcp_missing_content_parameter() {
        let server = AnyrepairMcpServer::new();
        let input = json!({});
        let result = server.process_tool_call("repair_json", &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_missing_format_parameter_validate() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "test"
        });
        let result = server.process_tool_call("validate", &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_invalid_format_validate() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "test",
            "format": "invalid_format"
        });
        let result = server.process_tool_call("validate", &input);
        assert!(result.is_err());
    }

    // ===== Edge Cases Tests =====

    #[test]
    fn test_mcp_repair_empty_content() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": ""
        });
        let result = server.process_tool_call("repair", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_whitespace_only() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "   \n\t  "
        });
        let result = server.process_tool_call("repair", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_unicode_content() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"name": "日本語"}"#
        });
        let result = server.process_tool_call("repair_json", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_large_content() {
        let server = AnyrepairMcpServer::new();
        let large_content = format!(r#"{{"data": "{}"}}"#, "x".repeat(10000));
        let input = json!({
            "content": large_content
        });
        let result = server.process_tool_call("repair_json", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_special_characters() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"key": "value\nwith\nnewlines"}"#
        });
        let result = server.process_tool_call("repair_json", &input);
        assert!(result.is_ok());
    }

    // ===== Response Format Tests =====

    #[test]
    fn test_mcp_repair_response_format() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"key": "value"}"#
        });
        let result = server.process_tool_call("repair_json", &input);
        assert!(result.is_ok());
        let response = result.unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert!(parsed.get("repaired").is_some());
        assert!(parsed.get("success").is_some());
        assert!(parsed.get("confidence").is_some());
    }

    #[test]
    fn test_mcp_validate_response_format() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"key": "value"}"#,
            "format": "json"
        });
        let result = server.process_tool_call("validate", &input);
        assert!(result.is_ok());
        let response = result.unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert!(parsed.get("valid").is_some());
        assert!(parsed.get("format").is_some());
    }

    #[test]
    fn test_mcp_auto_repair_response_format() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"key": "value"}"#
        });
        let result = server.process_tool_call("repair", &input);
        assert!(result.is_ok());
        let response = result.unwrap();
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert!(parsed.get("repaired").is_some());
        assert!(parsed.get("success").is_some());
    }

    // ===== Consistency Tests =====

    #[test]
    fn test_mcp_repair_idempotent() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"key": "value"}"#
        });
        let result1 = server.process_tool_call("repair_json", &input);
        let result2 = server.process_tool_call("repair_json", &input);
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_mcp_validate_consistency() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"key": "value"}"#,
            "format": "json"
        });
        let result1 = server.process_tool_call("validate", &input);
        let result2 = server.process_tool_call("validate", &input);
        assert_eq!(result1, result2);
    }

    // ===== Diff Repair Tests =====

    #[test]
    fn test_mcp_repair_diff_basic() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "--- a/file.txt\n+++ b/file.txt\n@@ -1,3 +1,3 @@\n line1\n-old\n+new\n line3\n"
        });
        let result = server.process_tool_call("repair_diff", &input);
        assert!(result.is_ok());
        let parsed: Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(parsed["success"], true);
        assert!(parsed["confidence"].as_f64().is_some());
    }

    #[test]
    fn test_mcp_repair_diff_missing_headers() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "@@ -1,3 +1,3 @@\n line1\n-old\n+new\n line3\n"
        });
        let result = server.process_tool_call("repair_diff", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_validate_diff() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "--- a/file.txt\n+++ b/file.txt\n@@ -1,3 +1,3 @@\n line1\n-old\n+new\n line3\n",
            "format": "diff"
        });
        let result = server.process_tool_call("validate", &input);
        assert!(result.is_ok());
    }

    // ===== Content Correctness Tests =====

    #[test]
    fn test_mcp_repair_json_trailing_comma_correctness() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"key": "value",}"#
        });
        let result = server.process_tool_call("repair_json", &input);
        let parsed: Value = serde_json::from_str(&result.unwrap()).unwrap();
        let repaired = parsed["repaired"].as_str().unwrap();
        // Repaired JSON should be valid
        assert!(serde_json::from_str::<Value>(repaired).is_ok());
    }

    #[test]
    fn test_mcp_repair_json_single_quotes_correctness() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "{'key': 'value'}"
        });
        let result = server.process_tool_call("repair_json", &input);
        let parsed: Value = serde_json::from_str(&result.unwrap()).unwrap();
        let repaired = parsed["repaired"].as_str().unwrap();
        assert!(serde_json::from_str::<Value>(repaired).is_ok());
    }

    #[test]
    fn test_mcp_repair_yaml_correctness() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "name: John\n  age: 30"
        });
        let result = server.process_tool_call("repair_yaml", &input);
        let parsed: Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(parsed["success"], true);
        assert!(!parsed["repaired"].as_str().unwrap().is_empty());
    }

    #[test]
    fn test_mcp_repair_xml_unclosed_correctness() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "<root><item>value</root>"
        });
        let result = server.process_tool_call("repair_xml", &input);
        let parsed: Value = serde_json::from_str(&result.unwrap()).unwrap();
        let repaired = parsed["repaired"].as_str().unwrap();
        // Should have closing tags balanced
        assert!(repaired.contains("</root>"));
    }

    // ===== Confidence Score Range Tests =====

    #[test]
    fn test_mcp_confidence_score_range_valid_json() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"key": "value"}"#
        });
        let result = server.process_tool_call("repair_json", &input);
        let parsed: Value = serde_json::from_str(&result.unwrap()).unwrap();
        let confidence = parsed["confidence"].as_f64().unwrap();
        assert!(
            confidence >= 0.0 && confidence <= 1.0,
            "confidence {} out of range",
            confidence
        );
    }

    #[test]
    fn test_mcp_confidence_score_range_malformed_json() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "{key: value,}"
        });
        let result = server.process_tool_call("repair_json", &input);
        let parsed: Value = serde_json::from_str(&result.unwrap()).unwrap();
        let confidence = parsed["confidence"].as_f64().unwrap();
        assert!(
            confidence >= 0.0 && confidence <= 1.0,
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
            let input = json!({ "content": content });
            let result = server.process_tool_call(tool, &input);
            assert!(result.is_ok(), "tool {} failed", tool);
            let parsed: Value = serde_json::from_str(&result.unwrap()).unwrap();
            let confidence = parsed["confidence"].as_f64().unwrap();
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
            let input = json!({ "content": "test content" });
            let result = server.process_tool_call(&tool_name, &input);
            assert!(result.is_ok(), "repair_{} should succeed", format);
            let parsed: Value = serde_json::from_str(&result.unwrap()).unwrap();
            assert_eq!(
                parsed["success"], true,
                "repair_{} success should be true",
                format
            );
        }
    }

    #[test]
    fn test_mcp_validate_all_formats() {
        let server = AnyrepairMcpServer::new();
        for format in crate::SUPPORTED_FORMATS {
            let input = json!({
                "content": "test content",
                "format": format
            });
            let result = server.process_tool_call("validate", &input);
            assert!(result.is_ok(), "validate {} should succeed", format);
            let parsed: Value = serde_json::from_str(&result.unwrap()).unwrap();
            assert!(
                parsed.get("valid").is_some(),
                "validate {} should have valid field",
                format
            );
            assert_eq!(parsed["format"], *format);
        }
    }

    // ===== Complex Malformed Input Tests =====

    #[test]
    fn test_mcp_repair_json_deeply_nested() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"a": {"b": {"c": {"d": "value",},},},}"#
        });
        let result = server.process_tool_call("repair_json", &input);
        assert!(result.is_ok());
        let parsed: Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(parsed["success"], true);
    }

    #[test]
    fn test_mcp_repair_json_mixed_issues() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "{'users': [{'name': 'John', 'age': 30,}, {'name': 'Jane', 'age': 25,},],}"
        });
        let result = server.process_tool_call("repair_json", &input);
        assert!(result.is_ok());
        let parsed: Value = serde_json::from_str(&result.unwrap()).unwrap();
        let repaired = parsed["repaired"].as_str().unwrap();
        assert!(
            serde_json::from_str::<Value>(repaired).is_ok(),
            "repaired JSON should be valid: {}",
            repaired
        );
    }

    #[test]
    fn test_mcp_repair_yaml_complex_indentation() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "server:\n  host: localhost\n    port: 8080\ndb:\n  name: test\n    user: admin"
        });
        let result = server.process_tool_call("repair_yaml", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_markdown_broken_links() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "#Header\n\n[broken link(http://example.com)\n\n**unclosed bold"
        });
        let result = server.process_tool_call("repair_markdown", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_csv_inconsistent_columns() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "name,age,city\nJohn,30\nJane,25,NYC,extra"
        });
        let result = server.process_tool_call("repair_csv", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_toml_malformed() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "[package]\nname = test\nversion = 1.0"
        });
        let result = server.process_tool_call("repair_toml", &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_repair_ini_missing_equals() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "[section]\nkey value\nother = ok"
        });
        let result = server.process_tool_call("repair_ini", &input);
        assert!(result.is_ok());
    }

    // ===== Tool Schema Validation Tests =====

    #[test]
    fn test_mcp_tool_schemas_have_required_fields() {
        let server = AnyrepairMcpServer::new();
        for tool in server.get_tools() {
            let schema = &tool.input_schema;
            assert_eq!(schema["type"], "object", "tool {} schema type", tool.name);
            assert!(
                schema.get("properties").is_some(),
                "tool {} has properties",
                tool.name
            );
            assert!(
                schema.get("required").is_some(),
                "tool {} has required",
                tool.name
            );
            // All tools require "content"
            let required = schema["required"].as_array().unwrap();
            assert!(
                required.iter().any(|v| v == "content"),
                "tool {} requires content",
                tool.name
            );
        }
    }

    #[test]
    fn test_mcp_validate_tool_schema_has_format_enum() {
        let server = AnyrepairMcpServer::new();
        let tools = server.get_tools();
        let validate_tool = tools.iter().find(|t| t.name == "validate").unwrap();
        let format_prop = &validate_tool.input_schema["properties"]["format"];
        assert!(
            format_prop.get("enum").is_some(),
            "validate tool should have format enum"
        );
        let enum_values = format_prop["enum"].as_array().unwrap();
        // Should include all supported formats
        for fmt in crate::SUPPORTED_FORMATS {
            assert!(
                enum_values.iter().any(|v| v.as_str() == Some(fmt)),
                "validate enum should include {}",
                fmt
            );
        }
    }

    // ===== Error Edge Cases =====

    #[test]
    fn test_mcp_repair_null_content() {
        let server = AnyrepairMcpServer::new();
        let input = json!({ "content": null });
        let result = server.process_tool_call("repair_json", &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_repair_numeric_content() {
        let server = AnyrepairMcpServer::new();
        let input = json!({ "content": 42 });
        let result = server.process_tool_call("repair_json", &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_validate_unknown_format() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": "test",
            "format": "protobuf"
        });
        let result = server.process_tool_call("validate", &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_unknown_repair_format() {
        let server = AnyrepairMcpServer::new();
        let input = json!({ "content": "test" });
        let result = server.process_tool_call("repair_protobuf", &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_empty_tool_name() {
        let server = AnyrepairMcpServer::new();
        let input = json!({ "content": "test" });
        let result = server.process_tool_call("", &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_repair_with_extra_params_ignored() {
        let server = AnyrepairMcpServer::new();
        let input = json!({
            "content": r#"{"key": "value"}"#,
            "extra_param": "should be ignored",
            "another": 123
        });
        let result = server.process_tool_call("repair_json", &input);
        assert!(result.is_ok());
    }

    // ===== Validate After Repair Round-Trip Tests =====

    #[test]
    fn test_mcp_repair_then_validate_json() {
        let server = AnyrepairMcpServer::new();
        // Repair malformed JSON
        let repair_input = json!({ "content": r#"{"key": "value",}"# });
        let repair_result = server
            .process_tool_call("repair_json", &repair_input)
            .unwrap();
        let repair_parsed: Value = serde_json::from_str(&repair_result).unwrap();
        let repaired = repair_parsed["repaired"].as_str().unwrap();

        // Validate the repaired content
        let validate_input = json!({
            "content": repaired,
            "format": "json"
        });
        let validate_result = server
            .process_tool_call("validate", &validate_input)
            .unwrap();
        let validate_parsed: Value = serde_json::from_str(&validate_result).unwrap();
        assert_eq!(
            validate_parsed["valid"], true,
            "repaired JSON should validate: {}",
            repaired
        );
    }

    #[test]
    fn test_mcp_repair_then_validate_yaml() {
        let server = AnyrepairMcpServer::new();
        let repair_input = json!({ "content": "name: John\nage: 30" });
        let repair_result = server
            .process_tool_call("repair_yaml", &repair_input)
            .unwrap();
        let repair_parsed: Value = serde_json::from_str(&repair_result).unwrap();
        let repaired = repair_parsed["repaired"].as_str().unwrap();

        let validate_input = json!({ "content": repaired, "format": "yaml" });
        let validate_result = server
            .process_tool_call("validate", &validate_input)
            .unwrap();
        let validate_parsed: Value = serde_json::from_str(&validate_result).unwrap();
        assert_eq!(validate_parsed["valid"], true);
    }

    #[test]
    fn test_mcp_repair_then_validate_xml() {
        let server = AnyrepairMcpServer::new();
        let repair_input = json!({ "content": "<root><item>value</root>" });
        let repair_result = server
            .process_tool_call("repair_xml", &repair_input)
            .unwrap();
        let repair_parsed: Value = serde_json::from_str(&repair_result).unwrap();
        let repaired = repair_parsed["repaired"].as_str().unwrap();

        let validate_input = json!({ "content": repaired, "format": "xml" });
        let validate_result = server
            .process_tool_call("validate", &validate_input)
            .unwrap();
        let validate_parsed: Value = serde_json::from_str(&validate_result).unwrap();
        assert_eq!(
            validate_parsed["valid"], true,
            "repaired XML should validate: {}",
            repaired
        );
    }
}
