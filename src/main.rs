//! CLI application for anyrepair

mod cli;

use clap::{Parser, Subcommand};
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
        /// Input file path (or use --input flag, stdin if not provided)
        #[arg(value_name = "FILE")]
        file: Option<String>,
        
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
    /// Process batch files
    Batch {
        /// Input directory
        #[arg(short, long)]
        input: String,
        
        /// Output directory
        #[arg(short, long)]
        output: String,
        
        /// File pattern to match
        #[arg(short, long)]
        pattern: Option<String>,
    },
    /// Manage repair rules
    Rules {
        /// Action: list or show
        #[arg(value_name = "ACTION")]
        action: String,
    },
    /// Stream repair for large files
    Stream {
        /// Input file (stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,
        
        /// Output file (stdout if not provided)
        #[arg(short, long)]
        output: Option<String>,
        
        /// Format (auto-detect if not provided)
        #[arg(short, long)]
        format: Option<String>,
        
        /// Buffer size in bytes
        #[arg(long)]
        buffer_size: Option<usize>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let start_time = Instant::now();
    
    match cli.command {
        Commands::Repair { file, input, output, confidence } => {
            let input_path = file.as_deref().or(input.as_deref());
            cli::repair_cmd::handle_repair(input_path, output.as_deref(), confidence, cli.verbose)?;
        }
        Commands::Json { input, output, confidence } => {
            cli::repair_cmd::handle_json(input.as_deref(), output.as_deref(), confidence, cli.verbose)?;
        }
        Commands::Yaml { input, output, confidence } => {
            cli::repair_cmd::handle_yaml(input.as_deref(), output.as_deref(), confidence, cli.verbose)?;
        }
        Commands::Markdown { input, output, confidence } => {
            cli::repair_cmd::handle_markdown(input.as_deref(), output.as_deref(), confidence, cli.verbose)?;
        }
        Commands::Validate { input, format } => {
            cli::validate_cmd::handle_validate(input.as_deref(), format.as_deref(), cli.verbose)?;
        }
        Commands::Batch { input, output, pattern } => {
            cli::batch_cmd::handle_batch(&input, &output, pattern.as_deref(), cli.verbose)?;
        }
        Commands::Rules { action } => {
            cli::rules_cmd::handle_rules(&action, cli.verbose)?;
        }
        Commands::Stream { input, output, format, buffer_size } => {
            let fmt = format.as_deref().unwrap_or("auto");
            cli::stream_cmd::handle_stream(input.as_deref(), output.as_deref(), fmt, buffer_size, cli.verbose)?;
        }
    }
    
    if cli.verbose && !cli.quiet {
        let duration = start_time.elapsed();
        eprintln!("Completed in {:?}", duration);
    }
    
    Ok(())
}
