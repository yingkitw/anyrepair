//! MCP (Model Context Protocol) server for anyrepair
//!
//! Exposes anyrepair repair functionality as an MCP server for integration
//! with Claude and other MCP-compatible clients.

use crate::traits::Repair;
use crate::{json, yaml, markdown, xml, toml, csv, ini};
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
        for format in &["json", "yaml", "markdown", "xml", "toml", "csv", "ini"] {
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
                            "enum": ["json", "yaml", "markdown", "xml", "toml", "csv", "ini"],
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
        match name {
            "repair" => self.handle_repair(input),
            "repair_json" => self.handle_repair_json(input),
            "repair_yaml" => self.handle_repair_yaml(input),
            "repair_markdown" => self.handle_repair_markdown(input),
            "repair_xml" => self.handle_repair_xml(input),
            "repair_toml" => self.handle_repair_toml(input),
            "repair_csv" => self.handle_repair_csv(input),
            "repair_ini" => self.handle_repair_ini(input),
            "validate" => self.handle_validate(input),
            _ => Err(format!("Unknown tool: {}", name)),
        }
    }

    fn handle_repair(&self, input: &Value) -> Result<String, String> {
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' parameter")?;

        let repaired = crate::repair(content)
            .map_err(|e| format!("Repair failed: {}", e))?;

        Ok(json!({
            "repaired": repaired,
            "success": true
        })
        .to_string())
    }

    fn handle_repair_json(&self, input: &Value) -> Result<String, String> {
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' parameter")?;

        let mut repairer = json::JsonRepairer::new();
        let repaired = repairer
            .repair(content)
            .map_err(|e| format!("JSON repair failed: {}", e))?;

        let confidence = repairer.confidence(&repaired);

        Ok(json!({
            "repaired": repaired,
            "confidence": confidence,
            "success": true
        })
        .to_string())
    }

    fn handle_repair_yaml(&self, input: &Value) -> Result<String, String> {
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' parameter")?;

        let mut repairer = yaml::YamlRepairer::new();
        let repaired = repairer
            .repair(content)
            .map_err(|e| format!("YAML repair failed: {}", e))?;

        let confidence = repairer.confidence(&repaired);

        Ok(json!({
            "repaired": repaired,
            "confidence": confidence,
            "success": true
        })
        .to_string())
    }

    fn handle_repair_markdown(&self, input: &Value) -> Result<String, String> {
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' parameter")?;

        let mut repairer = markdown::MarkdownRepairer::new();
        let repaired = repairer
            .repair(content)
            .map_err(|e| format!("Markdown repair failed: {}", e))?;

        let confidence = repairer.confidence(&repaired);

        Ok(json!({
            "repaired": repaired,
            "confidence": confidence,
            "success": true
        })
        .to_string())
    }

    fn handle_repair_xml(&self, input: &Value) -> Result<String, String> {
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' parameter")?;

        let mut repairer = xml::XmlRepairer::new();
        let repaired = repairer
            .repair(content)
            .map_err(|e| format!("XML repair failed: {}", e))?;

        let confidence = repairer.confidence(&repaired);

        Ok(json!({
            "repaired": repaired,
            "confidence": confidence,
            "success": true
        })
        .to_string())
    }

    fn handle_repair_toml(&self, input: &Value) -> Result<String, String> {
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' parameter")?;

        let mut repairer = toml::TomlRepairer::new();
        let repaired = repairer
            .repair(content)
            .map_err(|e| format!("TOML repair failed: {}", e))?;

        let confidence = repairer.confidence(&repaired);

        Ok(json!({
            "repaired": repaired,
            "confidence": confidence,
            "success": true
        })
        .to_string())
    }

    fn handle_repair_csv(&self, input: &Value) -> Result<String, String> {
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' parameter")?;

        let mut repairer = csv::CsvRepairer::new();
        let repaired = repairer
            .repair(content)
            .map_err(|e| format!("CSV repair failed: {}", e))?;

        let confidence = repairer.confidence(&repaired);

        Ok(json!({
            "repaired": repaired,
            "confidence": confidence,
            "success": true
        })
        .to_string())
    }

    fn handle_repair_ini(&self, input: &Value) -> Result<String, String> {
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' parameter")?;

        let mut repairer = ini::IniRepairer::new();
        let repaired = repairer
            .repair(content)
            .map_err(|e| format!("INI repair failed: {}", e))?;

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

        use crate::traits::Validator;

        let is_valid = match format {
            "json" => json::JsonValidator.is_valid(content),
            "yaml" => yaml::YamlValidator.is_valid(content),
            "markdown" => markdown::MarkdownValidator.is_valid(content),
            "xml" => xml::XmlValidator.is_valid(content),
            "toml" => toml::TomlValidator.is_valid(content),
            "csv" => csv::CsvValidator.is_valid(content),
            "ini" => ini::IniValidator.is_valid(content),
            _ => return Err(format!("Unknown format: {}", format)),
        };

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
        // repair_toml, repair_csv, repair_ini, validate = 9 tools
        assert_eq!(tools.len(), 9);
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
}
