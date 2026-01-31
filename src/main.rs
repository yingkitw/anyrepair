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

        /// Specify format (skip auto-detection)
        #[arg(short, long)]
        format: Option<String>,
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
    /// Repair XML content
    Xml {
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
    /// Repair TOML content
    Toml {
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
    /// Repair CSV content
    Csv {
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
    /// Repair INI content
    Ini {
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
    /// Repair Diff/Unified Diff content
    Diff {
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
    /// Manage plugins
    Plugins {
        /// Action: list, load, unload
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
        Commands::Json { input, output, confidence } => {
            cli::repair_cmd::handle_json(input.as_deref(), output.as_deref(), confidence, cli.verbose)?;
        }
        Commands::Yaml { input, output, confidence } => {
            cli::repair_cmd::handle_yaml(input.as_deref(), output.as_deref(), confidence, cli.verbose)?;
        }
        Commands::Markdown { input, output, confidence } => {
            cli::repair_cmd::handle_markdown(input.as_deref(), output.as_deref(), confidence, cli.verbose)?;
        }
        Commands::Xml { input, output, confidence } => {
            cli::repair_cmd::handle_xml(input.as_deref(), output.as_deref(), confidence, cli.verbose)?;
        }
        Commands::Toml { input, output, confidence } => {
            cli::repair_cmd::handle_toml(input.as_deref(), output.as_deref(), confidence, cli.verbose)?;
        }
        Commands::Csv { input, output, confidence } => {
            cli::repair_cmd::handle_csv(input.as_deref(), output.as_deref(), confidence, cli.verbose)?;
        }
        Commands::Ini { input, output, confidence } => {
            cli::repair_cmd::handle_ini(input.as_deref(), output.as_deref(), confidence, cli.verbose)?;
        }
        Commands::Diff { input, output, confidence } => {
            cli::repair_cmd::handle_diff(input.as_deref(), output.as_deref(), confidence, cli.verbose)?;
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
        Commands::Plugins { action } => {
            cli::plugins_cmd::handle_plugins(&action, cli.verbose)?;
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

    println!("AnyRepair Statistics:");
    println!("====================");
    println!();
    println!("Supported formats: 8");
    println!("  - JSON");
    println!("  - YAML");
    println!("  - Markdown");
    println!("  - XML");
    println!("  - TOML");
    println!("  - CSV");
    println!("  - INI");
    println!("  - Diff/Unified Diff");
    println!();
    println!("JSON Repair Strategies: 9");
    println!("  - StripTrailingContent (priority: 100)");
    println!("  - StripJsComments (priority: 95)");
    println!("  - AddMissingQuotes (priority: 90)");
    println!("  - FixTrailingCommas (priority: 80)");
    println!("  - AddMissingBraces (priority: 60)");
    println!("  - FixSingleQuotes (priority: 85)");
    println!("  - FixMalformedNumbers (priority: 75)");
    println!("  - FixBooleanNull (priority: 70)");
    println!("  - FixAgenticAiResponse (priority: 50)");
    println!();
    println!("For detailed performance metrics, use AnalyticsTracker from the library API.");
    println!("For batch processing statistics, use the batch command with verbose mode.");

    Ok(())
}
