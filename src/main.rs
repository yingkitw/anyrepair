//! CLI application for anyrepair

use anyrepair::{repair, json, yaml, markdown, traits::{Repair, Validator}, config::{RepairConfig, CustomRule}, custom_rules::{CustomRuleEngine, RuleTemplates}, plugin_config::ExtendedRepairConfig, plugin_integration::PluginRegistryManager};
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Read};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "anyrepair")]
#[command(about = "A tool for repairing LLM responses including JSON, YAML, and Markdown")]
#[command(version)]
struct Cli {
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Quiet mode (suppress non-error output)
    #[arg(short, long)]
    quiet: bool,
    
    /// Configuration file
    #[arg(short, long)]
    config: Option<String>,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Repair content automatically (detects format)
    Repair {
        /// Input file (stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,
        
        /// Output file (stdout if not provided)
        #[arg(short, long)]
        output: Option<String>,
        
        /// Show confidence score
        #[arg(long)]
        confidence: bool,
    },
    /// Repair JSON content
    Json {
        /// Input file (stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,
        
        /// Output file (stdout if not provided)
        #[arg(short, long)]
        output: Option<String>,
        
        /// Show confidence score
        #[arg(long)]
        confidence: bool,
    },
    /// Repair YAML content
    Yaml {
        /// Input file (stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,
        
        /// Output file (stdout if not provided)
        #[arg(short, long)]
        output: Option<String>,
        
        /// Show confidence score
        #[arg(long)]
        confidence: bool,
    },
    /// Repair Markdown content
    Markdown {
        /// Input file (stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,
        
        /// Output file (stdout if not provided)
        #[arg(short, long)]
        output: Option<String>,
        
        /// Show confidence score
        #[arg(long)]
        confidence: bool,
    },
    /// Validate content without repairing
    Validate {
        /// Input file (stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,
        
        /// Format to validate (auto-detect if not provided)
        #[arg(short, long)]
        format: Option<String>,
    },
    /// Batch process multiple files
    Batch {
        /// Input directory
        #[arg(short, long)]
        input_dir: String,
        
        /// Output directory
        #[arg(short, long)]
        output_dir: String,
        
        /// File pattern (e.g., "*.json")
        #[arg(short, long, default_value = "*")]
        pattern: String,
        
        /// Show progress
        #[arg(long)]
        progress: bool,
    },
    /// Show statistics about repair quality
    Stats {
        /// Input file (stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,
    },
    /// Manage custom repair rules
    Rules {
        #[command(subcommand)]
        command: RuleCommands,
    },
    /// Manage plugins
    Plugins {
        #[command(subcommand)]
        command: PluginCommands,
    },
}

#[derive(Subcommand)]
enum RuleCommands {
    /// List all custom rules
    List {
        /// Show rules for specific format only
        #[arg(short, long)]
        format: Option<String>,
    },
    /// Add a new custom rule
    Add {
        /// Rule ID
        #[arg(short, long)]
        id: String,
        
        /// Rule name
        #[arg(short, long)]
        name: String,
        
        /// Target format (json, yaml, markdown, etc.)
        #[arg(short, long)]
        format: String,
        
        /// Pattern to match
        #[arg(short, long)]
        pattern: String,
        
        /// Replacement pattern
        #[arg(short, long)]
        replacement: String,
        
        /// Priority (0-10, higher = more priority)
        #[arg(long, default_value = "5")]
        priority: u8,
        
        /// Configuration file to update
        #[arg(short, long, default_value = "anyrepair.toml")]
        config: String,
    },
    /// Remove a custom rule
    Remove {
        /// Rule ID to remove
        #[arg(short, long)]
        id: String,
        
        /// Configuration file to update
        #[arg(short, long, default_value = "anyrepair.toml")]
        config: String,
    },
    /// Enable/disable a custom rule
    Toggle {
        /// Rule ID to toggle
        #[arg(short, long)]
        id: String,
        
        /// Configuration file to update
        #[arg(short, long, default_value = "anyrepair.toml")]
        config: String,
    },
    /// Initialize configuration file with templates
    Init {
        /// Output configuration file
        #[arg(short, long, default_value = "anyrepair.toml")]
        output: String,
    },
    /// Test a custom rule
    Test {
        /// Rule ID to test
        #[arg(short, long)]
        id: String,
        
        /// Input content to test
        #[arg(long)]
        input: String,
        
        /// Configuration file
        #[arg(short, long, default_value = "anyrepair.toml")]
        config: String,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// List all available plugins
    List {
        /// Show only enabled plugins
        #[arg(long)]
        enabled_only: bool,
        
        /// Show plugins for specific format
        #[arg(short, long)]
        format: Option<String>,
    },
    /// Enable/disable a plugin
    Toggle {
        /// Plugin ID to toggle
        #[arg(short, long)]
        id: String,
        
        /// Enable (true) or disable (false)
        #[arg(long)]
        enable: bool,
        
        /// Configuration file
        #[arg(short, long, default_value = "anyrepair.toml")]
        config: String,
    },
    /// Show plugin information
    Info {
        /// Plugin ID
        #[arg(short, long)]
        id: String,
    },
    /// Show plugin statistics
    Stats,
    /// Initialize plugin configuration
    Init {
        /// Output configuration file
        #[arg(short, long, default_value = "anyrepair.toml")]
        output: String,
    },
    /// Discover plugins in search paths
    Discover {
        /// Plugin search paths (comma-separated)
        #[arg(short, long)]
        paths: Option<String>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Load configuration if provided
    if let Some(config_path) = &cli.config {
        if !cli.quiet {
            eprintln!("Loading configuration from: {}", config_path);
        }
        // TODO: Implement configuration loading
    }
    
    match cli.command {
        Commands::Repair { input, output, confidence } => {
            let start_time = Instant::now();
            let content = read_input(input)?;
            
            if cli.verbose && !cli.quiet {
                eprintln!("Detecting format and repairing content...");
            }
            
            let repaired = repair(&content)?;
            write_output(output, &repaired)?;
            
            if confidence && !cli.quiet {
                let repairer = json::JsonRepairer::new();
                let conf = repairer.confidence(&content);
                eprintln!("Confidence: {:.2}", conf);
            }
            
            if cli.verbose && !cli.quiet {
                let duration = start_time.elapsed();
                eprintln!("Repair completed in {:?}", duration);
            }
        }
        Commands::Json { input, output, confidence } => {
            let content = read_input(input)?;
            let repairer = json::JsonRepairer::new();
            let repaired = repairer.repair(&content)?;
            write_output(output, &repaired)?;
            
            if confidence {
                let conf = repairer.confidence(&content);
                eprintln!("Confidence: {:.2}", conf);
            }
        }
        Commands::Yaml { input, output, confidence } => {
            let content = read_input(input)?;
            let repairer = yaml::YamlRepairer::new();
            let repaired = repairer.repair(&content)?;
            write_output(output, &repaired)?;
            
            if confidence {
                let conf = repairer.confidence(&content);
                eprintln!("Confidence: {:.2}", conf);
            }
        }
        Commands::Markdown { input, output, confidence } => {
            let content = read_input(input)?;
            let repairer = markdown::MarkdownRepairer::new();
            let repaired = repairer.repair(&content)?;
            write_output(output, &repaired)?;
            
            if confidence {
                let conf = repairer.confidence(&content);
                eprintln!("Confidence: {:.2}", conf);
            }
        }
        Commands::Validate { input, format } => {
            let content = read_input(input)?;
            
            match format.as_deref() {
                Some("json") => {
                    let validator = json::JsonValidator;
                    if validator.is_valid(&content) {
                        println!("Valid JSON");
                    } else {
                        let errors = validator.validate(&content);
                        eprintln!("Invalid JSON:");
                        for error in errors {
                            eprintln!("  - {}", error);
                        }
                        std::process::exit(1);
                    }
                }
                Some("yaml") => {
                    let validator = yaml::YamlValidator;
                    if validator.is_valid(&content) {
                        println!("Valid YAML");
                    } else {
                        let errors = validator.validate(&content);
                        eprintln!("Invalid YAML:");
                        for error in errors {
                            eprintln!("  - {}", error);
                        }
                        std::process::exit(1);
                    }
                }
                Some("markdown") => {
                    let validator = markdown::MarkdownValidator;
                    if validator.is_valid(&content) {
                        println!("Valid Markdown");
                    } else {
                        let errors = validator.validate(&content);
                        eprintln!("Invalid Markdown:");
                        for error in errors {
                            eprintln!("  - {}", error);
                        }
                        std::process::exit(1);
                    }
                }
                _ => {
                    // Auto-detect format
                    if json::JsonRepairer::new().confidence(&content) > 0.5 {
                        let validator = json::JsonValidator;
                        if validator.is_valid(&content) {
                            println!("Valid JSON");
                        } else {
                            eprintln!("Invalid JSON");
                            std::process::exit(1);
                        }
                    } else if yaml::YamlRepairer::new().confidence(&content) > 0.5 {
                        let validator = yaml::YamlValidator;
                        if validator.is_valid(&content) {
                            println!("Valid YAML");
                        } else {
                            eprintln!("Invalid YAML");
                            std::process::exit(1);
                        }
                    } else {
                        let validator = markdown::MarkdownValidator;
                        if validator.is_valid(&content) {
                            println!("Valid Markdown");
                        } else {
                            eprintln!("Invalid Markdown");
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Batch { input_dir, output_dir, pattern, progress } => {
            if !cli.quiet {
                eprintln!("Batch processing files from '{}' to '{}'", input_dir, output_dir);
                eprintln!("Pattern: {}", pattern);
            }
            
            // Create output directory if it doesn't exist
            fs::create_dir_all(&output_dir)?;
            
            // Get list of files to process
            let entries = fs::read_dir(&input_dir)?;
            let files: Vec<_> = entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    if let Some(name) = entry.file_name().to_str() {
                        if pattern == "*" {
                            true
                        } else {
                            name.ends_with(&pattern[1..]) // Remove the '*' prefix
                        }
                    } else {
                        false
                    }
                })
                .collect();
            
            if files.is_empty() {
                eprintln!("No files found matching pattern '{}'", pattern);
                return Ok(());
            }
            
            let total_files = files.len();
            let mut processed = 0;
            let mut successful = 0;
            let mut failed = 0;
            
            for entry in files {
                let input_path = entry.path();
                let file_name = input_path.file_name().unwrap().to_string_lossy();
                let output_path = std::path::Path::new(&output_dir).join(&*file_name);
                
                if progress && !cli.quiet {
                    eprintln!("Processing {}/{}: {}", processed + 1, total_files, file_name);
                }
                
                match process_file(&input_path, &output_path) {
                    Ok(_) => {
                        successful += 1;
                        if cli.verbose && !cli.quiet {
                            eprintln!("✓ Successfully processed: {}", file_name);
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        eprintln!("✗ Failed to process {}: {}", file_name, e);
                    }
                }
                
                processed += 1;
            }
            
            if !cli.quiet {
                eprintln!("Batch processing completed:");
                eprintln!("  Total files: {}", total_files);
                eprintln!("  Successful: {}", successful);
                eprintln!("  Failed: {}", failed);
            }
        }
        Commands::Stats { input } => {
            let content = read_input(input)?;
            let start_time = Instant::now();
            
            // Test all repairers
            let json_repairer = json::JsonRepairer::new();
            let yaml_repairer = yaml::YamlRepairer::new();
            let markdown_repairer = markdown::MarkdownRepairer::new();
            
            let json_conf = json_repairer.confidence(&content);
            let yaml_conf = yaml_repairer.confidence(&content);
            let markdown_conf = markdown_repairer.confidence(&content);
            
            let detection_time = start_time.elapsed();
            
            println!("Format Detection Statistics:");
            println!("  JSON confidence: {:.3}", json_conf);
            println!("  YAML confidence: {:.3}", yaml_conf);
            println!("  Markdown confidence: {:.3}", markdown_conf);
            println!("  Detection time: {:?}", detection_time);
            
            // Determine best format
            let best_format = if json_conf >= yaml_conf && json_conf >= markdown_conf {
                "JSON"
            } else if yaml_conf >= markdown_conf {
                "YAML"
            } else {
                "Markdown"
            };
            
            println!("  Recommended format: {}", best_format);
            
            // Test repair if needed
            if json_conf > 0.5 || yaml_conf > 0.5 || markdown_conf > 0.5 {
                let repair_start = Instant::now();
                let repaired = repair(&content)?;
                let repair_time = repair_start.elapsed();
                
                println!("Repair Statistics:");
                println!("  Original size: {} bytes", content.len());
                println!("  Repaired size: {} bytes", repaired.len());
                println!("  Size change: {} bytes", repaired.len() as i32 - content.len() as i32);
                println!("  Repair time: {:?}", repair_time);
            }
        }
        Commands::Rules { command } => {
            match command {
                RuleCommands::List { format } => {
                    let config = load_or_create_config(&cli.config)?;
                    let engine = create_rule_engine(&config)?;
                    
                    if let Some(target_format) = format {
                        let rules = engine.get_rules_for_format(&target_format);
                        if rules.is_empty() {
                            println!("No custom rules found for format: {}", target_format);
                        } else {
                            println!("Custom rules for {}:", target_format);
                            for rule in rules {
                                println!("  {} ({}): {}", rule.id, rule.priority, rule.name);
                                println!("    Pattern: {}", rule.pattern);
                                println!("    Replacement: {}", rule.replacement);
                                println!("    Enabled: {}", rule.enabled);
                                println!();
                            }
                        }
                    } else {
                        let stats = engine.get_statistics();
                        println!("Custom Rules Summary:");
                        println!("  Total rules: {}", stats.total_rules);
                        println!("  Enabled rules: {}", stats.enabled_rules);
                        println!("  Formats: {}", stats.format_count);
                        println!();
                        
                        for (format, rules) in &engine.rules {
                            if !rules.is_empty() {
                                println!("{} ({} rules):", format, rules.len());
                                for rule in rules {
                                    println!("  {} ({}): {} - {}", rule.id, rule.priority, rule.name, if rule.enabled { "enabled" } else { "disabled" });
                                }
                                println!();
                            }
                        }
                    }
                }
                RuleCommands::Add { id, name, format, pattern, replacement, priority, config } => {
                    let mut config_data = load_or_create_config(&Some(config.clone()))?;
                    
                    let rule = CustomRule {
                        id: id.clone(),
                        name: name.clone(),
                        description: format!("Custom rule: {}", name),
                        target_format: format.clone(),
                        priority,
                        enabled: true,
                        pattern: pattern.clone(),
                        replacement: replacement.clone(),
                        conditions: vec![],
                    };
                    
                    // Remove existing rule with same ID
                    config_data.custom_rules.retain(|r| r.id != id);
                    config_data.add_custom_rule(rule);
                    
                    config_data.to_file(&config)?;
                    println!("Added custom rule '{}' for format '{}'", name, format);
                }
                RuleCommands::Remove { id, config } => {
                    let mut config_data = load_or_create_config(&Some(config.clone()))?;
                    let original_count = config_data.custom_rules.len();
                    config_data.custom_rules.retain(|r| r.id != id);
                    
                    if config_data.custom_rules.len() < original_count {
                        config_data.to_file(&config)?;
                        println!("Removed custom rule '{}'", id);
                    } else {
                        eprintln!("Rule '{}' not found", id);
                        std::process::exit(1);
                    }
                }
                RuleCommands::Toggle { id, config } => {
                    let mut config_data = load_or_create_config(&Some(config.clone()))?;
                    
                    if let Some(rule) = config_data.custom_rules.iter_mut().find(|r| r.id == id) {
                        let _was_enabled = rule.enabled;
                        rule.enabled = !rule.enabled;
                        let new_state = rule.enabled;
                        config_data.to_file(&config)?;
                        println!("{} custom rule '{}'", if new_state { "Enabled" } else { "Disabled" }, id);
                    } else {
                        eprintln!("Rule '{}' not found", id);
                        std::process::exit(1);
                    }
                }
                RuleCommands::Init { output } => {
                    let mut config = RepairConfig::new();
                    
                    // Add template rules
                    let templates = RuleTemplates::get_all_templates();
                    for template in templates {
                        config.add_custom_rule(template);
                    }
                    
                    config.to_file(&output)?;
                    println!("Initialized configuration file: {}", output);
                    println!("Added {} template rules", config.custom_rules.len());
                }
                RuleCommands::Test { id, input, config } => {
                    let config_data = load_or_create_config(&Some(config))?;
                    let engine = create_rule_engine(&config_data)?;
                    
                    if let Some(rule) = config_data.custom_rules.iter().find(|r| r.id == id) {
                        println!("Testing rule: {} ({})", rule.name, rule.id);
                        println!("Pattern: {}", rule.pattern);
                        println!("Replacement: {}", rule.replacement);
                        println!("Input: {}", input);
                        
                        let result = engine.apply_rules(&input, &rule.target_format)?;
                        println!("Output: {}", result);
                        
                        if result != input {
                            println!("✓ Rule applied successfully");
                        } else {
                            println!("⚠ Rule did not modify input");
                        }
                    } else {
                        eprintln!("Rule '{}' not found", id);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Plugins { command } => {
            let mut plugin_manager = PluginRegistryManager::new();
            
            // Load configuration if available
            if let Some(config_path) = &cli.config {
                if std::path::Path::new(config_path).exists() {
                    let config = ExtendedRepairConfig::from_file(config_path)?;
                    plugin_manager.load_config(config)?;
                }
            }
            
            match command {
                PluginCommands::List { enabled_only, format } => {
                    let plugins = plugin_manager.list_plugins();
                    let filtered_plugins: Vec<_> = plugins.into_iter()
                        .filter(|p| {
                            if enabled_only && !p.enabled {
                                return false;
                            }
                            if let Some(ref fmt) = format {
                                return p.supported_formats.contains(fmt) || p.supported_formats.contains(&"*".to_string());
                            }
                            true
                        })
                        .collect();
                    
                    if filtered_plugins.is_empty() {
                        println!("No plugins found");
                    } else {
                        println!("Available Plugins:");
                        for plugin in filtered_plugins {
                            println!("  {} ({}): {}", plugin.id, plugin.version, plugin.name);
                            println!("    Description: {}", plugin.description);
                            println!("    Author: {}", plugin.author);
                            println!("    Enabled: {}", plugin.enabled);
                            println!("    Loaded: {}", plugin.loaded);
                            println!("    Supported formats: {}", plugin.supported_formats.join(", "));
                            println!();
                        }
                    }
                }
                PluginCommands::Toggle { id, enable, config: _ } => {
                    plugin_manager.toggle_plugin(&id, enable)?;
                    println!("{} plugin '{}'", if enable { "Enabled" } else { "Disabled" }, id);
                }
                PluginCommands::Info { id } => {
                    let plugins = plugin_manager.list_plugins();
                    if let Some(plugin) = plugins.iter().find(|p| p.id == id) {
                        println!("Plugin Information:");
                        println!("  ID: {}", plugin.id);
                        println!("  Name: {}", plugin.name);
                        println!("  Version: {}", plugin.version);
                        println!("  Description: {}", plugin.description);
                        println!("  Author: {}", plugin.author);
                        println!("  Enabled: {}", plugin.enabled);
                        println!("  Loaded: {}", plugin.loaded);
                        println!("  Supported formats: {}", plugin.supported_formats.join(", "));
                    } else {
                        eprintln!("Plugin '{}' not found", id);
                        std::process::exit(1);
                    }
                }
                PluginCommands::Stats => {
                    let stats = plugin_manager.get_statistics();
                    println!("Plugin Statistics:");
                    println!("  Total plugins: {}", stats.total_plugins);
                    println!("  Enabled plugins: {}", stats.enabled_plugins);
                    println!("  Loaded plugins: {}", stats.loaded_plugins);
                    println!("  Total strategies: {}", stats.total_strategies);
                    println!("  Total validators: {}", stats.total_validators);
                    println!("  Total repairers: {}", stats.total_repairers);
                }
                PluginCommands::Init { output } => {
                    let config = ExtendedRepairConfig::new();
                    config.to_file(&output)?;
                    println!("Initialized plugin configuration file: {}", output);
                }
                PluginCommands::Discover { paths } => {
                    let search_paths = if let Some(paths_str) = paths {
                        paths_str.split(',').map(|s| s.trim().to_string()).collect()
                    } else {
                        vec!["./plugins".to_string()]
                    };
                    
                    let discovery = anyrepair::plugin_config::PluginDiscovery::new(search_paths);
                    match discovery.discover_plugins() {
                        Ok(discovered) => {
                            if discovered.is_empty() {
                                println!("No plugins discovered in search paths");
                            } else {
                                println!("Discovered Plugins:");
                                for plugin in discovered {
                                    println!("  {} ({}): {}", plugin.metadata.id, plugin.metadata.version, plugin.metadata.name);
                                    println!("    Path: {}", plugin.path);
                                    println!("    Description: {}", plugin.metadata.description);
                                    println!("    Author: {}", plugin.metadata.author);
                                    println!("    Supported formats: {}", plugin.metadata.supported_formats.join(", "));
                                    println!();
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error discovering plugins: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn read_input(input: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    match input {
        Some(path) => Ok(fs::read_to_string(path)?),
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

fn write_output(output: Option<String>, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    match output {
        Some(path) => {
            fs::write(path, content)?;
            Ok(())
        }
        None => {
            print!("{}", content);
            Ok(())
        }
    }
}

fn process_file(input_path: &std::path::Path, output_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(input_path)?;
    let repaired = repair(&content)?;
    fs::write(output_path, repaired)?;
    Ok(())
}

fn load_or_create_config(config_path: &Option<String>) -> Result<RepairConfig, Box<dyn std::error::Error>> {
    match config_path {
        Some(path) => {
            if std::path::Path::new(path).exists() {
                RepairConfig::from_file(path)
            } else {
                let config = RepairConfig::new();
                config.to_file(path)?;
                Ok(config)
            }
        }
        None => Ok(RepairConfig::new()),
    }
}

fn create_rule_engine(config: &RepairConfig) -> Result<CustomRuleEngine, Box<dyn std::error::Error>> {
    let mut engine = CustomRuleEngine::new();
    engine.load_from_config(config)?;
    Ok(engine)
}
