//! CLI application for anyrepair

use anyrepair::{repair, json, yaml, markdown, traits::{Repair, Validator}};
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
