use crate::config::RepairConfig;
use crate::plugin::{PluginConfig, PluginMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Extended configuration that includes plugin settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedRepairConfig {
    /// Base repair configuration
    #[serde(flatten)]
    pub base: RepairConfig,
    /// Plugin configurations
    pub plugins: HashMap<String, PluginConfig>,
    /// Plugin search paths
    pub plugin_paths: Vec<String>,
    /// Auto-load plugins from paths
    pub auto_load_plugins: bool,
}

impl Default for ExtendedRepairConfig {
    fn default() -> Self {
        Self {
            base: RepairConfig::default(),
            plugins: HashMap::new(),
            plugin_paths: vec!["./plugins".to_string()],
            auto_load_plugins: false,
        }
    }
}

impl ExtendedRepairConfig {
    /// Create a new extended configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Load configuration from a file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: ExtendedRepairConfig = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to a file
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Add a plugin configuration
    pub fn add_plugin(&mut self, plugin_config: PluginConfig) {
        let id = plugin_config.metadata.id.clone();
        self.plugins.insert(id, plugin_config);
    }
    
    /// Remove a plugin configuration
    pub fn remove_plugin(&mut self, plugin_id: &str) {
        self.plugins.remove(plugin_id);
    }
    
    /// Get plugin configuration by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&PluginConfig> {
        self.plugins.get(plugin_id)
    }
    
    /// Get all enabled plugins
    pub fn get_enabled_plugins(&self) -> Vec<&PluginConfig> {
        self.plugins.values().filter(|p| p.enabled).collect()
    }
    
    /// Enable/disable a plugin
    pub fn toggle_plugin(&mut self, plugin_id: &str, enabled: bool) -> bool {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.enabled = enabled;
            true
        } else {
            false
        }
    }
    
    /// Add a plugin search path
    pub fn add_plugin_path(&mut self, path: String) {
        if !self.plugin_paths.contains(&path) {
            self.plugin_paths.push(path);
        }
    }
    
    /// Remove a plugin search path
    pub fn remove_plugin_path(&mut self, path: &str) {
        self.plugin_paths.retain(|p| p != path);
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Validate base configuration
        if let Err(base_errors) = self.base.validate() {
            errors.extend(base_errors);
        }
        
        // Validate plugin configurations
        for (plugin_id, plugin_config) in &self.plugins {
            if plugin_config.metadata.id != *plugin_id {
                errors.push(format!("Plugin ID mismatch: {} != {}", plugin_id, plugin_config.metadata.id));
            }
            
            if plugin_config.metadata.id.is_empty() {
                errors.push(format!("Plugin '{}' has empty ID", plugin_id));
            }
            
            if plugin_config.metadata.name.is_empty() {
                errors.push(format!("Plugin '{}' has empty name", plugin_id));
            }
            
            if plugin_config.priority > 10 {
                errors.push(format!("Plugin '{}' priority must be between 0 and 10", plugin_id));
            }
        }
        
        // Validate plugin paths
        for path in &self.plugin_paths {
            if !std::path::Path::new(path).exists() {
                errors.push(format!("Plugin path does not exist: {}", path));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Plugin discovery and loading utilities
pub struct PluginDiscovery {
    search_paths: Vec<String>,
}

impl PluginDiscovery {
    /// Create a new plugin discovery instance
    pub fn new(search_paths: Vec<String>) -> Self {
        Self { search_paths }
    }
    
    /// Discover plugins in the search paths
    pub fn discover_plugins(&self) -> Result<Vec<DiscoveredPlugin>, Box<dyn std::error::Error>> {
        let mut discovered = Vec::new();
        
        for path in &self.search_paths {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries {
                    let entry = entry?;
                    let path = entry.path();
                    
                    if path.is_file() {
                        if let Some(extension) = path.extension() {
                            if self.is_plugin_file(extension) {
                                if let Ok(metadata) = self.extract_plugin_metadata(&path) {
                                    discovered.push(DiscoveredPlugin {
                                        path: path.to_string_lossy().to_string(),
                                        metadata,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(discovered)
    }
    
    /// Check if a file is a plugin file
    fn is_plugin_file(&self, extension: &std::ffi::OsStr) -> bool {
        let ext = extension.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "dylib" | "so" | "dll" | "toml" | "json")
    }
    
    /// Extract plugin metadata from a file
    fn extract_plugin_metadata(&self, path: &std::path::Path) -> Result<PluginMetadata, Box<dyn std::error::Error>> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "toml" | "json" => {
                // Load metadata from configuration file
                let content = std::fs::read_to_string(path)?;
                let config: PluginConfig = if extension == "toml" {
                    toml::from_str(&content)?
                } else {
                    serde_json::from_str(&content)?
                };
                Ok(config.metadata)
            }
            _ => {
                // For binary plugins, we would need to load the metadata dynamically
                // For now, return a placeholder
                Err("Binary plugin metadata extraction not yet implemented".into())
            }
        }
    }
}

/// A discovered plugin
#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    pub path: String,
    pub metadata: PluginMetadata,
}

/// Plugin configuration builder
pub struct PluginConfigBuilder {
    metadata: PluginMetadata,
    settings: HashMap<String, serde_json::Value>,
    enabled: bool,
    priority: u8,
}

impl PluginConfigBuilder {
    /// Create a new plugin configuration builder
    pub fn new(id: String, name: String) -> Self {
        Self {
            metadata: PluginMetadata {
                id,
                name,
                version: "1.0.0".to_string(),
                description: String::new(),
                author: String::new(),
                supported_formats: vec!["*".to_string()],
                dependencies: vec![],
                config_schema: None,
            },
            settings: HashMap::new(),
            enabled: true,
            priority: 5,
        }
    }
    
    /// Set plugin version
    pub fn version(mut self, version: String) -> Self {
        self.metadata.version = version;
        self
    }
    
    /// Set plugin description
    pub fn description(mut self, description: String) -> Self {
        self.metadata.description = description;
        self
    }
    
    /// Set plugin author
    pub fn author(mut self, author: String) -> Self {
        self.metadata.author = author;
        self
    }
    
    /// Set supported formats
    pub fn supported_formats(mut self, formats: Vec<String>) -> Self {
        self.metadata.supported_formats = formats;
        self
    }
    
    /// Set plugin dependencies
    pub fn dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.metadata.dependencies = dependencies;
        self
    }
    
    /// Set configuration schema
    pub fn config_schema(mut self, schema: serde_json::Value) -> Self {
        self.metadata.config_schema = Some(schema);
        self
    }
    
    /// Add a setting
    pub fn setting(mut self, key: String, value: serde_json::Value) -> Self {
        self.settings.insert(key, value);
        self
    }
    
    /// Set enabled status
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    /// Set priority
    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
    
    /// Build the plugin configuration
    pub fn build(self) -> PluginConfig {
        PluginConfig {
            metadata: self.metadata,
            settings: self.settings,
            enabled: self.enabled,
            priority: self.priority,
        }
    }
}

impl Default for PluginConfigBuilder {
    fn default() -> Self {
        Self::new("default".to_string(), "Default Plugin".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extended_config() {
        let config = ExtendedRepairConfig::new();
        assert_eq!(config.plugins.len(), 0);
        assert_eq!(config.plugin_paths.len(), 1);
        assert!(!config.auto_load_plugins);
    }
    
    #[test]
    fn test_plugin_config_builder() {
        let config = PluginConfigBuilder::new("test".to_string(), "Test Plugin".to_string())
            .version("1.0.0".to_string())
            .description("A test plugin".to_string())
            .author("Test Author".to_string())
            .supported_formats(vec!["json".to_string(), "yaml".to_string()])
            .enabled(true)
            .priority(7)
            .build();
        
        assert_eq!(config.metadata.id, "test");
        assert_eq!(config.metadata.name, "Test Plugin");
        assert_eq!(config.metadata.version, "1.0.0");
        assert_eq!(config.enabled, true);
        assert_eq!(config.priority, 7);
    }
    
    #[test]
    fn test_plugin_discovery() {
        let discovery = PluginDiscovery::new(vec!["./plugins".to_string()]);
        // This will fail if the plugins directory doesn't exist, which is expected
        let _ = discovery.discover_plugins();
    }
}
