# Plugin Development Guide

This guide explains how to create custom plugins for the `anyrepair` crate.

## Overview

The plugin system allows you to extend `anyrepair` with custom repair strategies, validators, and repairers. Plugins can be written in Rust and loaded dynamically.

## Plugin Structure

A plugin must implement the `Plugin` trait and provide one or more of:
- **Repair Strategies**: Custom repair logic for specific issues
- **Validators**: Custom validation logic for content
- **Repairers**: Complete repair implementations for specific formats

## Creating a Plugin

### 1. Basic Plugin Structure

```rust
use anyrepair::{
    plugin::{Plugin, PluginMetadata, PluginConfig},
    traits::{Repair, RepairStrategy, Validator},
    error::Result,
};

pub struct MyPlugin {
    metadata: PluginMetadata,
    enabled: bool,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "my_plugin".to_string(),
                name: "My Custom Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "A custom plugin for specific repair needs".to_string(),
                author: "Your Name".to_string(),
                supported_formats: vec!["json".to_string(), "yaml".to_string()],
                dependencies: vec![],
                config_schema: None,
            },
            enabled: false,
        }
    }
}

impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn initialize(&mut self, config: &PluginConfig) -> Result<()> {
        self.enabled = config.enabled;
        // Initialize your plugin resources here
        Ok(())
    }
    
    fn get_strategies(&self) -> Vec<Box<dyn RepairStrategy>> {
        if !self.enabled {
            return vec![];
        }
        
        vec![
            Box::new(MyStrategy::new()),
        ]
    }
    
    fn get_validator(&self) -> Option<Box<dyn Validator>> {
        if !self.enabled {
            return None;
        }
        
        Some(Box::new(MyValidator::new()))
    }
    
    fn get_repairer(&self) -> Option<Box<dyn Repair>> {
        if !self.enabled {
            return None;
        }
        
        Some(Box::new(MyRepairer::new()))
    }
    
    fn cleanup(&mut self) -> Result<()> {
        // Cleanup your plugin resources here
        Ok(())
    }
}
```

### 2. Creating a Repair Strategy

```rust
use anyrepair::traits::RepairStrategy;
use anyrepair::error::Result;

pub struct MyStrategy {
    name: String,
}

impl MyStrategy {
    pub fn new() -> Self {
        Self {
            name: "My Custom Strategy".to_string(),
        }
    }
}

impl RepairStrategy for MyStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        // Your custom repair logic here
        // For example, fix a specific JSON issue:
        if content.contains("undefined") {
            Ok(content.replace("undefined", "null"))
        } else {
            Ok(content.to_string())
        }
    }
    
    fn priority(&self) -> u8 {
        // Higher numbers = higher priority (0-10)
        5
    }
}
```

### 3. Creating a Validator

```rust
use anyrepair::traits::Validator;

pub struct MyValidator;

impl MyValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Validator for MyValidator {
    fn is_valid(&self, content: &str) -> bool {
        // Your validation logic here
        !content.contains("invalid_pattern")
    }
    
    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        if content.contains("invalid_pattern") {
            errors.push("Content contains invalid pattern".to_string());
        }
        
        errors
    }
}
```

### 4. Creating a Repairer

```rust
use anyrepair::traits::Repair;
use anyrepair::error::Result;

pub struct MyRepairer;

impl MyRepairer {
    pub fn new() -> Self {
        Self
    }
}

impl Repair for MyRepairer {
    fn repair(&self, content: &str) -> Result<String> {
        // Your complete repair logic here
        let mut result = content.to_string();
        
        // Apply multiple fixes
        result = result.replace("undefined", "null");
        result = result.replace("NaN", "null");
        
        Ok(result)
    }
    
    fn confidence(&self, content: &str) -> f64 {
        // Return confidence score (0.0 to 1.0)
        if content.contains("undefined") || content.contains("NaN") {
            0.9 // High confidence for known issues
        } else {
            0.1 // Low confidence for unknown content
        }
    }
}
```

## Plugin Configuration

### Configuration File Format

Plugins are configured in TOML format:

```toml
[global]
max_attempts = 3
parallel_processing = true
min_confidence = 0.5
verbose = false

[plugins.my_plugin]
enabled = true
priority = 5

[plugins.my_plugin.settings]
custom_setting = "value"
another_setting = 42
```

### Plugin Metadata

Each plugin must provide metadata:

- **id**: Unique identifier
- **name**: Human-readable name
- **version**: Plugin version
- **description**: What the plugin does
- **author**: Plugin author
- **supported_formats**: List of supported formats (`*` for all)
- **dependencies**: Required dependencies
- **config_schema**: JSON schema for configuration validation

## Loading Plugins

### 1. Using the CLI

```bash
# List available plugins
anyrepair plugins list

# Show plugin information
anyrepair plugins info my_plugin

# Enable/disable plugins
anyrepair plugins toggle --id my_plugin --enable

# Show plugin statistics
anyrepair plugins stats

# Discover plugins in directories
anyrepair plugins discover --paths ./plugins,./custom-plugins
```

### 2. Programmatically

```rust
use anyrepair::plugin_integration::PluginRegistryManager;
use anyrepair::plugin_config::ExtendedRepairConfig;

let mut manager = PluginRegistryManager::new();

// Load configuration
let config = ExtendedRepairConfig::from_file("config.toml")?;
manager.load_config(config)?;

// Get all strategies
let strategies = manager.plugin_manager().registry().get_all_strategies();

// Get strategies for specific format
let json_strategies = manager.plugin_manager().registry()
    .get_strategies_for_format("json");
```

## Best Practices

### 1. Error Handling

Always handle errors gracefully:

```rust
impl RepairStrategy for MyStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        // Use ? operator for error propagation
        let result = self.process_content(content)?;
        Ok(result)
    }
}
```

### 2. Performance

- Keep strategies focused and efficient
- Use appropriate priority levels
- Avoid expensive operations in validation

### 3. Testing

Write comprehensive tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy() {
        let strategy = MyStrategy::new();
        let result = strategy.apply("test input").unwrap();
        assert!(result.contains("expected output"));
    }
}
```

### 4. Documentation

- Document your plugin's purpose and usage
- Provide examples
- Include configuration options

## Example Plugin

See `examples/example_plugin.rs` for a complete working example.

## Plugin Discovery

The system automatically discovers plugins in configured search paths:

1. **Binary plugins**: `.dylib`, `.so`, `.dll` files
2. **Configuration plugins**: `.toml`, `.json` files with plugin metadata

## Troubleshooting

### Common Issues

1. **Plugin not loading**: Check file permissions and format
2. **Strategy not applying**: Verify priority and enabled status
3. **Configuration errors**: Validate TOML syntax and schema

### Debug Mode

Enable verbose logging to debug plugin issues:

```bash
anyrepair --verbose plugins list
```

## Advanced Features

### 1. Plugin Dependencies

Specify required dependencies:

```rust
PluginMetadata {
    dependencies: vec!["other_plugin".to_string()],
    // ...
}
```

### 2. Configuration Schema

Define configuration validation:

```rust
PluginMetadata {
    config_schema: Some(serde_json::json!({
        "type": "object",
        "properties": {
            "custom_setting": {
                "type": "string",
                "description": "A custom setting"
            }
        }
    })),
    // ...
}
```

### 3. Format-Specific Logic

Handle different formats differently:

```rust
impl RepairStrategy for MyStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        if content.trim().starts_with('{') {
            // JSON-specific logic
            self.repair_json(content)
        } else if content.contains("---") {
            // YAML-specific logic
            self.repair_yaml(content)
        } else {
            // Generic logic
            Ok(content.to_string())
        }
    }
}
```

## Contributing

When contributing plugins:

1. Follow the coding standards
2. Include comprehensive tests
3. Update documentation
4. Test with various input formats
5. Consider performance implications

## Support

For plugin development support:

- Check the examples
- Review the API documentation
- Test with the provided example plugin
- Use the CLI tools for debugging
