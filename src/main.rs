//! CLI application for anyrepair

mod cli;

use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let start_time = Instant::now();

    match parse_args(&args) {
        Ok(cli) => {
            let verbose = cli.verbose;
            let quiet = cli.quiet;
            match dispatch(cli) {
                Ok(()) => {
                    if verbose && !quiet {
                        eprintln!("Completed in {:?}", start_time.elapsed());
                    }
                    ExitCode::SUCCESS
                }
                Err(code) => ExitCode::from(code),
            }
        }
        Err(msg) => {
            eprintln!("{msg}");
            print_usage(&args[0]);
            ExitCode::from(2)
        }
    }
}

/// Parsed CLI state.
struct Cli {
    verbose: bool,
    quiet: bool,
    command: Command,
}

enum Command {
    Repair(RepairArgs),
    Validate(ValidateArgs),
    Batch(BatchArgs),
    Stream(StreamArgs),
    RepairBinary(RepairBinaryArgs),
}

struct RepairArgs {
    file: Option<String>,
    input: Option<String>,
    output: Option<String>,
    confidence: bool,
    format: Option<String>,
    diff: bool,
    dry_run: bool,
    json: bool,
    min_confidence: Option<f64>,
    explain: bool,
    color: String,
}

struct ValidateArgs {
    input: Option<String>,
    format: Option<String>,
}

struct BatchArgs {
    input: String,
    output: String,
    pattern: Option<String>,
    recursive: bool,
}

struct StreamArgs {
    input: Option<String>,
    output: Option<String>,
    format: Option<String>,
    buffer_size: Option<usize>,
}

struct RepairBinaryArgs {
    file: Option<String>,
    input: Option<String>,
    output: Option<String>,
    format: Option<String>,
    dry_run: bool,
}

fn dispatch(cli: Cli) -> Result<(), u8> {
    match cli.command {
        Command::Repair(a) => {
            let input_path = a.file.as_deref().or(a.input.as_deref());
            cli::repair_cmd::handle_repair(
                &cli::repair_cmd::RepairOptions {
                    input: input_path,
                    output: a.output.as_deref(),
                    format: a.format.as_deref(),
                    show_confidence: a.confidence,
                    show_diff: a.diff,
                    dry_run: a.dry_run,
                    json_output: a.json,
                    min_confidence: a.min_confidence,
                    explain: a.explain,
                    color: &a.color,
                },
                cli.verbose,
            )
            .map_err(|e| {
                eprintln!("{e}");
                1
            })
        }
        Command::Validate(a) => {
            cli::validate_cmd::handle_validate(a.input.as_deref(), a.format.as_deref(), cli.verbose)
                .map_err(|e| {
                    eprintln!("{e}");
                    1
                })
        }
        Command::Batch(a) => {
            cli::batch_cmd::handle_batch(
                &a.input,
                &a.output,
                a.pattern.as_deref(),
                a.recursive,
                cli.verbose,
            )
            .map_err(|e| {
                eprintln!("{e}");
                1
            })
        }
        Command::Stream(a) => {
            let fmt = a.format.as_deref().unwrap_or("auto");
            cli::stream_cmd::handle_stream(
                a.input.as_deref(),
                a.output.as_deref(),
                fmt,
                a.buffer_size,
                cli.verbose,
            )
            .map_err(|e| {
                eprintln!("{e}");
                1
            })
        }
        Command::RepairBinary(a) => {
            let input_path = a.file.as_deref().or(a.input.as_deref());
            cli::binary_cmd::handle_repair_binary(
                input_path,
                a.output.as_deref(),
                a.format.as_deref(),
                cli.verbose,
                a.dry_run,
            )
            .map_err(|e| {
                eprintln!("{e}");
                1
            })
        }
    }
}

/// Parse command-line arguments into a `Cli`.
fn parse_args(args: &[String]) -> Result<Cli, String> {
    if args.len() < 2 {
        return Err("error: no subcommand provided".to_string());
    }

    // Global flags that may appear before the subcommand.
    let mut verbose = false;
    let mut quiet = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-v" | "--verbose" => {
                verbose = true;
                i += 1;
            }
            "-q" | "--quiet" => {
                quiet = true;
                i += 1;
            }
            "-h" | "--help" => {
                return Err(usage_text(&args[0]));
            }
            "-V" | "--version" => {
                println!("anyrepair {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            other if other.starts_with('-') => {
                return Err(format!("error: unexpected global flag '{other}'"));
            }
            _ => break,
        }
    }

    if i >= args.len() {
        return Err("error: no subcommand provided".to_string());
    }

    let sub = &args[i];
    let sub_args = &args[i + 1..];

    let command = match sub.as_str() {
        "repair" => Command::Repair(parse_repair(sub_args)?),
        "validate" => Command::Validate(parse_validate(sub_args)?),
        "batch" => Command::Batch(parse_batch(sub_args)?),
        "stream" => Command::Stream(parse_stream(sub_args)?),
        "repair-binary" => Command::RepairBinary(parse_repair_binary(sub_args)?),
        "help" => {
            return Err(usage_text(&args[0]).to_string());
        }
        other => {
            return Err(format!("error: unknown subcommand '{other}'"));
        }
    };

    Ok(Cli {
        verbose,
        quiet,
        command,
    })
}

fn parse_repair(args: &[String]) -> Result<RepairArgs, String> {
    let mut a = RepairArgs {
        file: None,
        input: None,
        output: None,
        confidence: false,
        format: None,
        diff: false,
        dry_run: false,
        json: false,
        min_confidence: None,
        explain: false,
        color: "auto".to_string(),
    };
    let mut positional: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-h" | "--help" => return Err(usage_repair().to_string()),
            "--confidence" => a.confidence = true,
            "--diff" => a.diff = true,
            "--dry-run" => a.dry_run = true,
            "--json" => a.json = true,
            "--explain" => a.explain = true,
            "-i" | "--input" => {
                a.input = Some(take_value(args, &mut i, "--input")?);
            }
            "-o" | "--output" => {
                a.output = Some(take_value(args, &mut i, "--output")?);
            }
            "-f" | "--format" => {
                a.format = Some(take_value(args, &mut i, "--format")?);
            }
            "--min-confidence" => {
                let v = take_value(args, &mut i, "--min-confidence")?;
                a.min_confidence = Some(
                    v.parse::<f64>()
                        .map_err(|_| format!("error: --min-confidence expects a float, got '{v}'"))?,
                );
            }
            "--color" => {
                a.color = take_value(args, &mut i, "--color")?;
            }
            other if other.starts_with("--color=") => {
                a.color = other["--color=".len()..].to_string();
            }
            other if other.starts_with("--format=") => {
                a.format = Some(other["--format=".len()..].to_string());
            }
            other if other.starts_with("--input=") => {
                a.input = Some(other["--input=".len()..].to_string());
            }
            other if other.starts_with("--output=") => {
                a.output = Some(other["--output=".len()..].to_string());
            }
            other if other.starts_with("--min-confidence=") => {
                let v = &other["--min-confidence=".len()..];
                a.min_confidence = Some(
                    v.parse::<f64>()
                        .map_err(|_| format!("error: --min-confidence expects a float, got '{v}'"))?,
                );
            }
            other if other.starts_with('-') && other.len() > 1 => {
                return Err(format!("error: unexpected flag '{other}' for 'repair'"));
            }
            _ => positional.push(arg.clone()),
        }
        i += 1;
    }
    if a.file.is_none() && !positional.is_empty() {
        a.file = Some(positional.remove(0));
    }
    Ok(a)
}

fn parse_validate(args: &[String]) -> Result<ValidateArgs, String> {
    let mut a = ValidateArgs {
        input: None,
        format: None,
    };
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-h" | "--help" => return Err(usage_validate().to_string()),
            "-i" | "--input" => {
                a.input = Some(take_value(args, &mut i, "--input")?);
            }
            "-f" | "--format" => {
                a.format = Some(take_value(args, &mut i, "--format")?);
            }
            other if other.starts_with("--input=") => {
                a.input = Some(other["--input=".len()..].to_string());
            }
            other if other.starts_with("--format=") => {
                a.format = Some(other["--format=".len()..].to_string());
            }
            other if other.starts_with('-') && other.len() > 1 => {
                return Err(format!("error: unexpected flag '{other}' for 'validate'"));
            }
            _ => {}
        }
        i += 1;
    }
    Ok(a)
}

fn parse_batch(args: &[String]) -> Result<BatchArgs, String> {
    let mut a = BatchArgs {
        input: String::new(),
        output: String::new(),
        pattern: None,
        recursive: false,
    };
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-h" | "--help" => return Err(usage_batch().to_string()),
            "-r" | "--recursive" => a.recursive = true,
            "-i" | "--input" => {
                a.input = take_value(args, &mut i, "--input")?;
            }
            "-o" | "--output" => {
                a.output = take_value(args, &mut i, "--output")?;
            }
            "-p" | "--pattern" => {
                a.pattern = Some(take_value(args, &mut i, "--pattern")?);
            }
            other if other.starts_with("--input=") => {
                a.input = other["--input=".len()..].to_string();
            }
            other if other.starts_with("--output=") => {
                a.output = other["--output=".len()..].to_string();
            }
            other if other.starts_with("--pattern=") => {
                a.pattern = Some(other["--pattern=".len()..].to_string());
            }
            other if other.starts_with('-') && other.len() > 1 => {
                return Err(format!("error: unexpected flag '{other}' for 'batch'"));
            }
            _ => {}
        }
        i += 1;
    }
    if a.input.is_empty() {
        return Err("error: batch requires --input <DIR>".to_string());
    }
    if a.output.is_empty() {
        return Err("error: batch requires --output <DIR>".to_string());
    }
    Ok(a)
}

fn parse_stream(args: &[String]) -> Result<StreamArgs, String> {
    let mut a = StreamArgs {
        input: None,
        output: None,
        format: None,
        buffer_size: None,
    };
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-h" | "--help" => return Err(usage_stream().to_string()),
            "-i" | "--input" => {
                a.input = Some(take_value(args, &mut i, "--input")?);
            }
            "-o" | "--output" => {
                a.output = Some(take_value(args, &mut i, "--output")?);
            }
            "-f" | "--format" => {
                a.format = Some(take_value(args, &mut i, "--format")?);
            }
            "--buffer-size" => {
                let v = take_value(args, &mut i, "--buffer-size")?;
                a.buffer_size = Some(
                    v.parse::<usize>().map_err(|_| {
                        format!("error: --buffer-size expects an integer, got '{v}'")
                    })?,
                );
            }
            other if other.starts_with("--input=") => {
                a.input = Some(other["--input=".len()..].to_string());
            }
            other if other.starts_with("--output=") => {
                a.output = Some(other["--output=".len()..].to_string());
            }
            other if other.starts_with("--format=") => {
                a.format = Some(other["--format=".len()..].to_string());
            }
            other if other.starts_with("--buffer-size=") => {
                let v = &other["--buffer-size=".len()..];
                a.buffer_size = Some(
                    v.parse::<usize>().map_err(|_| {
                        format!("error: --buffer-size expects an integer, got '{v}'")
                    })?,
                );
            }
            other if other.starts_with('-') && other.len() > 1 => {
                return Err(format!("error: unexpected flag '{other}' for 'stream'"));
            }
            _ => {}
        }
        i += 1;
    }
    Ok(a)
}

fn parse_repair_binary(args: &[String]) -> Result<RepairBinaryArgs, String> {
    let mut a = RepairBinaryArgs {
        file: None,
        input: None,
        output: None,
        format: None,
        dry_run: false,
    };
    let mut positional: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-h" | "--help" => return Err(usage_repair_binary().to_string()),
            "--dry-run" => a.dry_run = true,
            "-i" | "--input" => {
                a.input = Some(take_value(args, &mut i, "--input")?);
            }
            "-o" | "--output" => {
                a.output = Some(take_value(args, &mut i, "--output")?);
            }
            "-f" | "--format" => {
                a.format = Some(take_value(args, &mut i, "--format")?);
            }
            other if other.starts_with("--input=") => {
                a.input = Some(other["--input=".len()..].to_string());
            }
            other if other.starts_with("--output=") => {
                a.output = Some(other["--output=".len()..].to_string());
            }
            other if other.starts_with("--format=") => {
                a.format = Some(other["--format=".len()..].to_string());
            }
            other if other.starts_with('-') && other.len() > 1 => {
                return Err(format!("error: unexpected flag '{other}' for 'repair-binary'"));
            }
            _ => positional.push(arg.clone()),
        }
        i += 1;
    }
    if a.file.is_none() && !positional.is_empty() {
        a.file = Some(positional.remove(0));
    }
    Ok(a)
}

/// Consume the value following a flag at position `i`, advancing `i` past it.
fn take_value(args: &[String], i: &mut usize, flag: &str) -> Result<String, String> {
    *i += 1;
    if *i >= args.len() {
        return Err(format!("error: {flag} requires a value"));
    }
    Ok(args[*i].clone())
}

fn print_usage(prog: &str) {
    eprintln!();
    eprintln!("{}", usage_text(prog));
}

fn usage_text(prog: &str) -> String {
    format!(
        "anyrepair {} — repair structured data and media files\n\
\n\
USAGE:\n    {prog} [OPTIONS] <SUBCOMMAND>\n\
\n\
OPTIONS:\n    -v, --verbose    Verbose output\n    -q, --quiet      Suppress non-error output\n    -h, --help       Print help\n    -V, --version    Print version\n\
\n\
SUBCOMMANDS:\n    repair           Repair content (auto-detects format, or use --format)\n    validate         Validate content without repairing\n    batch            Process batch files in a directory\n    stream           Stream repair for large files\n    repair-binary    Repair binary/media files (PNG, JPEG, WAV, MP3, MP4, MOV)\n    help             Print this help message\n\
\n\
Run `{prog} <SUBCOMMAND> --help` for subcommand-specific options.",
        env!("CARGO_PKG_VERSION")
    )
}

fn usage_repair() -> &'static str {
    "USAGE: anyrepair repair [OPTIONS] [FILE]\n\
\n\
OPTIONS:\n    [FILE]                  Input file path (or --input, stdin if not provided)\n    -i, --input <FILE>      Input file (stdin if not provided)\n    -o, --output <FILE>     Output file (stdout if not provided)\n    -f, --format <FORMAT>   Specify format: json, yaml, markdown, xml, toml, csv, ini, diff\n        --confidence        Show confidence score\n        --diff              Show a diff of changes without writing output\n        --dry-run           Show what would be repaired without writing output\n        --json              Output machine-readable JSON result to stdout (for CI)\n        --min-confidence <FLOAT>  Minimum confidence threshold (0.0–1.0)\n        --explain           Print which repair strategies were applied\n        --color <WHEN>      Color output: auto, always, never (default: auto)"
}

fn usage_validate() -> &'static str {
    "USAGE: anyrepair validate [OPTIONS]\n\
\n\
OPTIONS:\n    -i, --input <FILE>      Input file (stdin if not provided)\n    -f, --format <FORMAT>   Format to validate (auto-detect if not provided)"
}

fn usage_batch() -> &'static str {
    "USAGE: anyrepair batch [OPTIONS]\n\
\n\
OPTIONS:\n    -i, --input <DIR>       Input directory (required)\n    -o, --output <DIR>       Output directory (required)\n    -p, --pattern <PATTERN>  File pattern to match\n    -r, --recursive          Recursive directory processing"
}

fn usage_stream() -> &'static str {
    "USAGE: anyrepair stream [OPTIONS]\n\
\n\
OPTIONS:\n    -i, --input <FILE>      Input file (stdin if not provided)\n    -o, --output <FILE>     Output file (stdout if not provided)\n    -f, --format <FORMAT>   Format (auto-detect if not provided)\n        --buffer-size <BYTES>  Buffer size in bytes"
}

fn usage_repair_binary() -> &'static str {
    "USAGE: anyrepair repair-binary [OPTIONS] [FILE]\n\
\n\
OPTIONS:\n    [FILE]                  Input file path (or --input, stdin if not provided)\n    -i, --input <FILE>      Input file (stdin if not provided)\n    -o, --output <FILE>     Output file (stdout if not provided)\n    -f, --format <FORMAT>   Format: png, jpeg, wav, mp3, mp4, mov (auto-detect if not provided)\n        --dry-run           Show what would be repaired without writing output"
}
