//! CLI application for anyrepair

mod cli;

use clap::{Parser, Subcommand};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "anyrepair")]
#[command(about = "A tool for repairing LLM responses including JSON, YAML, and Markdown")]
#[command(version)]
pub struct Cli {
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Quiet mode (suppress non-error output)
    #[arg(short, long)]
    quiet: bool,

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

        /// Show a diff of changes without writing output
        #[arg(long)]
        diff: bool,

        /// Show what would be repaired without writing output
        #[arg(long)]
        dry_run: bool,

        /// Output machine-readable JSON result to stdout (for CI)
        #[arg(long)]
        json: bool,

        /// Minimum confidence threshold (0.0–1.0); exit with code 2 if below
        #[arg(long, value_name = "FLOAT")]
        min_confidence: Option<f64>,

        /// Print which repair strategies were applied
        #[arg(long)]
        explain: bool,

        /// Color output: auto, always, never
        #[arg(long, value_name = "WHEN", default_value = "auto")]
        color: String,
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
    /// Generate shell completions
    Completions {
        /// Shell: bash, zsh, fish, elvish, powershell
        #[arg(value_name = "SHELL")]
        shell: String,
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
        Commands::Repair { file, input, output, confidence, format, diff, dry_run, json, min_confidence, explain, color } => {
            let input_path = file.as_deref().or(input.as_deref());
            cli::repair_cmd::handle_repair(input_path, output.as_deref(), confidence, cli.verbose, format.as_deref(), diff, dry_run, json, min_confidence, explain, &color)?;
        }
        Commands::Validate { input, format } => {
            cli::validate_cmd::handle_validate(input.as_deref(), format.as_deref(), cli.verbose)?;
        }
        Commands::Batch { input, output, pattern, recursive } => {
            cli::batch_cmd::handle_batch(&input, &output, pattern.as_deref(), recursive, cli.verbose)?;
        }
        Commands::Stream { input, output, format, buffer_size } => {
            let fmt = format.as_deref().unwrap_or("auto");
            cli::stream_cmd::handle_stream(input.as_deref(), output.as_deref(), fmt, buffer_size, cli.verbose)?;
        }
        Commands::Completions { shell } => {
            cli::completions_cmd::handle_completions(&shell)?;
        }
    }

    if cli.verbose && !cli.quiet {
        let duration = start_time.elapsed();
        eprintln!("Completed in {:?}", duration);
    }

    Ok(())
}
