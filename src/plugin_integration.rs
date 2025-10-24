use crate::error::{RepairError, Result};
use crate::plugin::PluginManager;
use crate::plugin_config::ExtendedRepairConfig;
use crate::traits::{Repair, RepairStrategy, Validator};

/// Plugin-integrated repairer that combines built-in and plugin strategies
pub struct PluginIntegratedRepairer {
    plugin_manager: PluginManager,
    format: String,
}

impl PluginIntegratedRepairer {
    /// Create a new plugin-integrated repairer
    pub fn new(format: String) -> Self {
        Self {
            plugin_manager: PluginManager::new(),
            format,
        }
    }
    
    /// Load configuration and plugins
    pub fn load_config(&mut self, config: &ExtendedRepairConfig) -> Result<()> {
        // Load plugins from configuration
        for (plugin_id, plugin_config) in &config.plugins {
            if plugin_config.enabled {
                // For now, we'll create example plugins
                // In a real implementation, this would load from files
                self.load_example_plugin(plugin_id, plugin_config)?;
            }
        }
        
        Ok(())
    }
    
    /// Load an example plugin (for demonstration)
    fn load_example_plugin(&mut self, plugin_id: &str, _config: &crate::plugin::PluginConfig) -> Result<()> {
        // This is a placeholder for loading actual plugins
        // In a real implementation, this would use the plugin manager
        match plugin_id {
            "example" => {
                // Load example plugin
                Ok(())
            }
            _ => Err(RepairError::generic(format!("Unknown plugin: {}", plugin_id)))
        }
    }
    
    /// Get all repair strategies (built-in + plugin)
    pub fn get_all_strategies(&self) -> Vec<Box<dyn RepairStrategy>> {
        let mut strategies = Vec::new();
        
        // Add plugin strategies
        strategies.extend(self.plugin_manager.registry().get_strategies_for_format(&self.format));
        
        // Sort by priority
        strategies.sort_by_key(|s| std::cmp::Reverse(s.priority()));
        strategies
    }
    
    /// Get all validators (built-in + plugin)
    pub fn get_all_validators(&self) -> Vec<Box<dyn Validator>> {
        let mut validators = Vec::new();
        
        // Add plugin validators
        validators.extend(self.plugin_manager.registry().get_validators_for_format(&self.format));
        
        validators
    }
    
    /// Get all repairers (built-in + plugin)
    pub fn get_all_repairers(&self) -> Vec<Box<dyn Repair>> {
        let mut repairers = Vec::new();
        
        // Add plugin repairers
        repairers.extend(self.plugin_manager.registry().get_repairers_for_format(&self.format));
        
        repairers
    }
}

/// Plugin-aware format detector
pub struct PluginAwareFormatDetector;

impl PluginAwareFormatDetector {
    /// Create a new plugin-aware format detector
    pub fn new() -> Self {
        Self
    }
    
    /// Detect format with plugin assistance
    pub fn detect_format(&self, content: &str) -> String {
        // First try built-in detection
        let built_in_format = self.detect_built_in_format(content);
        
        // Then try plugin-based detection
        let plugin_format = self.detect_with_plugins(content);
        
        // Prefer plugin detection if available, otherwise use built-in
        plugin_format.unwrap_or(built_in_format)
    }
    
    /// Built-in format detection
    fn detect_built_in_format(&self, content: &str) -> String {
        let trimmed = content.trim();
        
        if (trimmed.starts_with('{') && (trimmed.ends_with('}') || trimmed.contains(':'))) ||
           (trimmed.starts_with('[') && (trimmed.ends_with(']') || trimmed.contains(','))) {
            "json".to_string()
        } else if trimmed.contains("---") || 
                  (trimmed.contains(":") && !trimmed.starts_with('{') && !trimmed.starts_with('[')) ||
                  trimmed.lines().any(|line| line.contains(":") && !line.trim().starts_with('"') && !line.trim().starts_with('{')) {
            "yaml".to_string()
        } else if trimmed.starts_with("<?xml") ||
                  (trimmed.starts_with('<') && trimmed.contains('>') && !trimmed.starts_with('#')) ||
                  (trimmed.contains('<') && trimmed.contains('>') && trimmed.contains("</")) {
            "xml".to_string()
        } else if trimmed.starts_with('[') ||
                  (trimmed.contains('=') && !trimmed.starts_with('{') && !trimmed.starts_with('<') && !trimmed.starts_with('#')) ||
                  trimmed.lines().any(|line| line.trim().starts_with('[') && line.trim().ends_with(']')) {
            "toml".to_string()
        } else if trimmed.contains(',') &&
                  !trimmed.starts_with('{') &&
                  !trimmed.starts_with('[') &&
                  !trimmed.starts_with('<') &&
                  !trimmed.starts_with('#') &&
                  !trimmed.starts_with("<?xml") &&
                  trimmed.lines().count() > 1 {
            "csv".to_string()
        } else if (trimmed.starts_with('[') && trimmed.contains(']')) ||
                  (trimmed.contains('=') && !trimmed.starts_with('{') && !trimmed.starts_with('<') && 
                   !trimmed.starts_with('#') && !trimmed.starts_with("<?xml") && 
                   !trimmed.contains(',') && !trimmed.contains(':')) ||
                  trimmed.lines().any(|line| {
                      let line = line.trim();
                      line.starts_with('[') && line.contains(']') && !line.contains(',')
                  }) {
            "ini".to_string()
        } else if trimmed.starts_with('#') ||
                  trimmed.contains("```") ||
                  trimmed.contains("**") ||
                  trimmed.contains("*") ||
                  trimmed.contains("`") {
            "markdown".to_string()
        } else {
            "unknown".to_string()
        }
    }
    
    /// Plugin-based format detection
    fn detect_with_plugins(&self, _content: &str) -> Option<String> {
        // This would use plugin-based format detection
        // For now, return None to use built-in detection
        None
    }
}

impl Default for PluginAwareFormatDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin registry manager for CLI operations
pub struct PluginRegistryManager {
    plugin_manager: PluginManager,
    config: Option<ExtendedRepairConfig>,
}

impl PluginRegistryManager {
    /// Create a new plugin registry manager
    pub fn new() -> Self {
        Self {
            plugin_manager: PluginManager::new(),
            config: None,
        }
    }
    
    /// Load configuration
    pub fn load_config(&mut self, config: ExtendedRepairConfig) -> Result<()> {
        self.config = Some(config);
        Ok(())
    }
    
    /// List all available plugins
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        let mut plugins = Vec::new();
        
        // Add built-in plugins
        plugins.push(PluginInfo {
            id: "built-in".to_string(),
            name: "Built-in Repairers".to_string(),
            version: "1.0.0".to_string(),
            description: "Built-in repair strategies for all supported formats".to_string(),
            author: "AnyRepair Team".to_string(),
            enabled: true,
            loaded: true,
            supported_formats: vec!["json".to_string(), "yaml".to_string(), "markdown".to_string(), 
                                   "xml".to_string(), "toml".to_string(), "csv".to_string(), "ini".to_string()],
        });
        
        // Add plugin-registered plugins
        for plugin in self.plugin_manager.registry().list_plugins() {
            let enabled = self.config.as_ref()
                .and_then(|c| c.get_plugin(&plugin.id))
                .map(|p| p.enabled)
                .unwrap_or(false);
            
            plugins.push(PluginInfo {
                id: plugin.id.clone(),
                name: plugin.name.clone(),
                version: plugin.version.clone(),
                description: plugin.description.clone(),
                author: plugin.author.clone(),
                enabled,
                loaded: true,
                supported_formats: plugin.supported_formats.clone(),
            });
        }
        
        plugins
    }
    
    /// Enable/disable a plugin
    pub fn toggle_plugin(&mut self, plugin_id: &str, enabled: bool) -> Result<()> {
        if plugin_id == "built-in" {
            return Err(RepairError::generic("Cannot disable built-in plugins"));
        }
        
        if let Some(config) = &mut self.config {
            if config.toggle_plugin(plugin_id, enabled) {
                Ok(())
            } else {
                Err(RepairError::generic(format!("Plugin '{}' not found", plugin_id)))
            }
        } else {
            Err(RepairError::generic("No configuration loaded"))
        }
    }
    
    /// Get plugin statistics
    pub fn get_statistics(&self) -> PluginManagerStatistics {
        let registry_stats = self.plugin_manager.registry().get_statistics();
        
        PluginManagerStatistics {
            total_plugins: registry_stats.total_plugins + 1, // +1 for built-in
            enabled_plugins: registry_stats.enabled_plugins + 1, // +1 for built-in
            loaded_plugins: registry_stats.total_plugins,
            total_strategies: registry_stats.total_strategies,
            total_validators: registry_stats.total_validators,
            total_repairers: registry_stats.total_repairers,
        }
    }
    
    /// Get the plugin manager
    pub fn plugin_manager(&self) -> &PluginManager {
        &self.plugin_manager
    }
    
    /// Get mutable access to the plugin manager
    pub fn plugin_manager_mut(&mut self) -> &mut PluginManager {
        &mut self.plugin_manager
    }
}

impl Default for PluginRegistryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin information for display
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub enabled: bool,
    pub loaded: bool,
    pub supported_formats: Vec<String>,
}

/// Plugin manager statistics
#[derive(Debug, Clone)]
pub struct PluginManagerStatistics {
    pub total_plugins: usize,
    pub enabled_plugins: usize,
    pub loaded_plugins: usize,
    pub total_strategies: usize,
    pub total_validators: usize,
    pub total_repairers: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_integrated_repairer() {
        let repairer = PluginIntegratedRepairer::new("json".to_string());
        assert_eq!(repairer.format, "json");
    }
    
    #[test]
    fn test_plugin_aware_format_detector() {
        let detector = PluginAwareFormatDetector::new();
        let format = detector.detect_format(r#"{"key": "value"}"#);
        assert_eq!(format, "json");
    }
    
    #[test]
    fn test_plugin_registry_manager() {
        let manager = PluginRegistryManager::new();
        let plugins = manager.list_plugins();
        assert!(!plugins.is_empty());
        assert!(plugins.iter().any(|p| p.id == "built-in"));
    }
}
