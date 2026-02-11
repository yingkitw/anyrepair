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
    /// Repair content (auto-detects format, or use --format to specify)
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

        /// Specify format: json, yaml, markdown, xml, toml, csv, ini, diff
        #[arg(short, long)]
        format: Option<String>,
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

        /// Recursive directory processing
        #[arg(short, long)]
        recursive: bool,
    },
    /// Manage repair rules
    Rules {
        /// Action: list, show, add, remove, enable, disable
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
    /// Show repair statistics
    Stats,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let start_time = Instant::now();

    match cli.command {
        Commands::Repair { file, input, output, confidence, format } => {
            let input_path = file.as_deref().or(input.as_deref());
            cli::repair_cmd::handle_repair(input_path, output.as_deref(), confidence, cli.verbose, format.as_deref())?;
        }
        Commands::Validate { input, format } => {
            cli::validate_cmd::handle_validate(input.as_deref(), format.as_deref(), cli.verbose)?;
        }
        Commands::Batch { input, output, pattern, recursive } => {
            cli::batch_cmd::handle_batch(&input, &output, pattern.as_deref(), recursive, cli.verbose)?;
        }
        Commands::Rules { action } => {
            cli::rules_cmd::handle_rules(&action, cli.verbose)?;
        }
        Commands::Stream { input, output, format, buffer_size } => {
            let fmt = format.as_deref().unwrap_or("auto");
            cli::stream_cmd::handle_stream(input.as_deref(), output.as_deref(), fmt, buffer_size, cli.verbose)?;
        }
        Commands::Stats => {
            handle_stats(cli.verbose)?;
        }
    }

    if cli.verbose && !cli.quiet {
        let duration = start_time.elapsed();
        eprintln!("Completed in {:?}", duration);
    }

    Ok(())
}

/// Handle stats command
fn handle_stats(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        eprintln!("Showing repair statistics...");
    }

    let formats = anyrepair::SUPPORTED_FORMATS;
    println!("AnyRepair Statistics:");
    println!("====================");
    println!();
    println!("Supported formats: {}", formats.len());
    for fmt in formats {
        println!("  - {}", fmt);
    }
    println!();
    println!("Use `anyrepair repair --format <FORMAT>` to repair a specific format.");

    Ok(())
}
