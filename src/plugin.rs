use crate::error::{RepairError, Result};
use crate::traits::{Repair, RepairStrategy, Validator};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin metadata for identification and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin identifier
    pub id: String,
    /// Human-readable plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Supported formats
    pub supported_formats: Vec<String>,
    /// Plugin dependencies
    pub dependencies: Vec<String>,
    /// Plugin configuration schema (JSON schema)
    pub config_schema: Option<serde_json::Value>,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Plugin-specific configuration
    pub settings: HashMap<String, serde_json::Value>,
    /// Whether the plugin is enabled
    pub enabled: bool,
    /// Plugin priority (higher = more priority)
    pub priority: u8,
}

/// Plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin with configuration
    fn initialize(&mut self, config: &PluginConfig) -> Result<()>;
    
    /// Get the plugin's repair strategies
    fn get_strategies(&self) -> Vec<Box<dyn RepairStrategy>>;
    
    /// Get the plugin's validator (if any)
    fn get_validator(&self) -> Option<Box<dyn Validator>>;
    
    /// Get the plugin's repairer (if any)
    fn get_repairer(&self) -> Option<Box<dyn Repair>>;
    
    /// Cleanup resources when plugin is unloaded
    fn cleanup(&mut self) -> Result<()>;
}

/// Plugin registry for managing loaded plugins
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
    configs: HashMap<String, PluginConfig>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            configs: HashMap::new(),
        }
    }
    
    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>, config: PluginConfig) -> Result<()> {
        let id = plugin.metadata().id.clone();
        
        // Initialize the plugin
        let mut plugin = plugin;
        plugin.initialize(&config)?;
        
        // Store the plugin and config
        self.plugins.insert(id.clone(), plugin);
        self.configs.insert(id, config);
        
        Ok(())
    }
    
    /// Unregister a plugin
    pub fn unregister_plugin(&mut self, plugin_id: &str) -> Result<()> {
        if let Some(mut plugin) = self.plugins.remove(plugin_id) {
            plugin.cleanup()?;
        }
        self.configs.remove(plugin_id);
        Ok(())
    }
    
    /// Get a plugin by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&dyn Plugin> {
        self.plugins.get(plugin_id).map(|p| p.as_ref())
    }
    
    /// Get all enabled plugins
    pub fn get_enabled_plugins(&self) -> Vec<&dyn Plugin> {
        self.plugins
            .iter()
            .filter(|(id, _)| {
                self.configs.get(*id)
                    .map(|config| config.enabled)
                    .unwrap_or(false)
            })
            .map(|(_, plugin)| plugin.as_ref())
            .collect()
    }
    
    /// Get plugins for a specific format
    pub fn get_plugins_for_format(&self, format: &str) -> Vec<&dyn Plugin> {
        self.get_enabled_plugins()
            .into_iter()
            .filter(|plugin| {
                plugin.metadata().supported_formats.contains(&format.to_string()) ||
                plugin.metadata().supported_formats.contains(&"*".to_string())
            })
            .collect()
    }
    
    /// Get all repair strategies from enabled plugins
    pub fn get_all_strategies(&self) -> Vec<Box<dyn RepairStrategy>> {
        let mut strategies = Vec::new();
        
        for plugin in self.get_enabled_plugins() {
            strategies.extend(plugin.get_strategies());
        }
        
        // Sort by priority (higher priority first)
        strategies.sort_by_key(|s| std::cmp::Reverse(s.priority()));
        strategies
    }
    
    /// Get repair strategies for a specific format
    pub fn get_strategies_for_format(&self, format: &str) -> Vec<Box<dyn RepairStrategy>> {
        let mut strategies = Vec::new();
        
        for plugin in self.get_plugins_for_format(format) {
            strategies.extend(plugin.get_strategies());
        }
        
        // Sort by priority (higher priority first)
        strategies.sort_by_key(|s| std::cmp::Reverse(s.priority()));
        strategies
    }
    
    /// Get all validators from enabled plugins
    pub fn get_all_validators(&self) -> Vec<Box<dyn Validator>> {
        let mut validators = Vec::new();
        
        for plugin in self.get_enabled_plugins() {
            if let Some(validator) = plugin.get_validator() {
                validators.push(validator);
            }
        }
        
        validators
    }
    
    /// Get validators for a specific format
    pub fn get_validators_for_format(&self, format: &str) -> Vec<Box<dyn Validator>> {
        let mut validators = Vec::new();
        
        for plugin in self.get_plugins_for_format(format) {
            if let Some(validator) = plugin.get_validator() {
                validators.push(validator);
            }
        }
        
        validators
    }
    
    /// Get all repairers from enabled plugins
    pub fn get_all_repairers(&self) -> Vec<Box<dyn Repair>> {
        let mut repairers = Vec::new();
        
        for plugin in self.get_enabled_plugins() {
            if let Some(repairer) = plugin.get_repairer() {
                repairers.push(repairer);
            }
        }
        
        repairers
    }
    
    /// Get repairers for a specific format
    pub fn get_repairers_for_format(&self, format: &str) -> Vec<Box<dyn Repair>> {
        let mut repairers = Vec::new();
        
        for plugin in self.get_plugins_for_format(format) {
            if let Some(repairer) = plugin.get_repairer() {
                repairers.push(repairer);
            }
        }
        
        repairers
    }
    
    /// List all registered plugins
    pub fn list_plugins(&self) -> Vec<&PluginMetadata> {
        self.plugins.values().map(|p| p.metadata()).collect()
    }
    
    /// Get plugin statistics
    pub fn get_statistics(&self) -> PluginStatistics {
        let total_plugins = self.plugins.len();
        let enabled_plugins = self.get_enabled_plugins().len();
        let total_strategies = self.get_all_strategies().len();
        let total_validators = self.get_all_validators().len();
        let total_repairers = self.get_all_repairers().len();
        
        PluginStatistics {
            total_plugins,
            enabled_plugins,
            total_strategies,
            total_validators,
            total_repairers,
        }
    }
    
    /// Update plugin configuration
    pub fn update_plugin_config(&mut self, plugin_id: &str, config: PluginConfig) -> Result<()> {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.initialize(&config)?;
        }
        self.configs.insert(plugin_id.to_string(), config);
        Ok(())
    }
    
    /// Enable/disable a plugin
    pub fn toggle_plugin(&mut self, plugin_id: &str, enabled: bool) -> Result<()> {
        if let Some(config) = self.configs.get_mut(plugin_id) {
            config.enabled = enabled;
            Ok(())
        } else {
            Err(RepairError::generic(format!("Plugin '{}' not found", plugin_id)))
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin statistics
#[derive(Debug, Clone)]
pub struct PluginStatistics {
    pub total_plugins: usize,
    pub enabled_plugins: usize,
    pub total_strategies: usize,
    pub total_validators: usize,
    pub total_repairers: usize,
}

/// Plugin loader trait for dynamic loading
pub trait PluginLoader: Send + Sync {
    /// Load a plugin from a path
    fn load_plugin(&self, path: &str) -> Result<Box<dyn Plugin>>;
    
    /// Get supported file extensions
    fn supported_extensions(&self) -> Vec<String>;
}

/// Built-in plugin loader for Rust plugins
pub struct RustPluginLoader;

impl PluginLoader for RustPluginLoader {
    fn load_plugin(&self, _path: &str) -> Result<Box<dyn Plugin>> {
        // This would typically use dynamic loading libraries like libloading
        // For now, we'll return an error as this is a foundation
        Err(RepairError::generic("Dynamic plugin loading not yet implemented"))
    }
    
    fn supported_extensions(&self) -> Vec<String> {
        vec!["dylib".to_string(), "so".to_string(), "dll".to_string()]
    }
}

/// Plugin manager that handles loading and managing plugins
pub struct PluginManager {
    registry: PluginRegistry,
    loaders: Vec<Box<dyn PluginLoader>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        let mut manager = Self {
            registry: PluginRegistry::new(),
            loaders: Vec::new(),
        };
        
        // Add built-in loaders
        manager.add_loader(Box::new(RustPluginLoader));
        
        manager
    }
    
    /// Add a plugin loader
    pub fn add_loader(&mut self, loader: Box<dyn PluginLoader>) {
        self.loaders.push(loader);
    }
    
    /// Load a plugin from a file
    pub fn load_plugin_from_file(&mut self, path: &str, config: PluginConfig) -> Result<()> {
        for loader in &self.loaders {
            if let Ok(plugin) = loader.load_plugin(path) {
                return self.registry.register_plugin(plugin, config);
            }
        }
        
        Err(RepairError::generic(format!("No loader found for plugin: {}", path)))
    }
    
    /// Get the plugin registry
    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }
    
    /// Get mutable access to the plugin registry
    pub fn registry_mut(&mut self) -> &mut PluginRegistry {
        &mut self.registry
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Example built-in plugin for demonstration
pub struct ExamplePlugin {
    metadata: PluginMetadata,
}

impl ExamplePlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "example".to_string(),
                name: "Example Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "An example plugin for demonstration".to_string(),
                author: "AnyRepair Team".to_string(),
                supported_formats: vec!["*".to_string()],
                dependencies: vec![],
                config_schema: None,
            },
        }
    }
}

impl Plugin for ExamplePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn initialize(&mut self, _config: &PluginConfig) -> Result<()> {
        // Initialize plugin resources
        Ok(())
    }
    
    fn get_strategies(&self) -> Vec<Box<dyn RepairStrategy>> {
        // Return example strategies
        vec![]
    }
    
    fn get_validator(&self) -> Option<Box<dyn Validator>> {
        None
    }
    
    fn get_repairer(&self) -> Option<Box<dyn Repair>> {
        None
    }
    
    fn cleanup(&mut self) -> Result<()> {
        // Cleanup plugin resources
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginConfig;

    #[test]
    fn test_plugin_registry() {
        let mut registry = PluginRegistry::new();
        
        let plugin = Box::new(ExamplePlugin::new());
        let config = PluginConfig {
            metadata: plugin.metadata().clone(),
            settings: HashMap::new(),
            enabled: true,
            priority: 5,
        };
        
        registry.register_plugin(plugin, config).unwrap();
        
        assert_eq!(registry.list_plugins().len(), 1);
        assert_eq!(registry.get_enabled_plugins().len(), 1);
    }
    
    #[test]
    fn test_plugin_manager() {
        let manager = PluginManager::new();
        assert_eq!(manager.loaders.len(), 1);
    }
    
    #[test]
    fn test_plugin_statistics() {
        let mut registry = PluginRegistry::new();
        
        let plugin = Box::new(ExamplePlugin::new());
        let config = PluginConfig {
            metadata: plugin.metadata().clone(),
            settings: HashMap::new(),
            enabled: true,
            priority: 5,
        };
        
        registry.register_plugin(plugin, config).unwrap();
        
        let stats = registry.get_statistics();
        assert_eq!(stats.total_plugins, 1);
        assert_eq!(stats.enabled_plugins, 1);
    }
}
