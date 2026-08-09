//! Binary (media) repair command handler.

use std::fs;
use std::io::{self, Read, Write};

use anyrepair::binary_detection::detect_binary_format;

pub fn handle_repair_binary(
    input: Option<&str>,
    output: Option<&str>,
    format: Option<&str>,
    verbose: bool,
    dry_run: bool,
) -> io::Result<()> {
    let data = read_binary_input(input)?;

    if data.is_empty() {
        eprintln!("Error: empty input");
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty input"));
    }

    let (repaired, detected_format, detection_confidence) = if let Some(fmt) = format {
        if verbose {
            eprintln!("Repairing binary as {}...", fmt);
        }
        let repaired = anyrepair::repair_binary_with_format(&data, fmt)
            .map_err(|e| io::Error::other(e.to_string()))?;
        (repaired, fmt.to_string(), 1.0)
    } else {
        match detect_binary_format(&data) {
            Some(detected) => {
                if verbose {
                    eprintln!(
                        "Detected format: {} (confidence: {:.0}%)",
                        detected.format,
                        detected.confidence * 100.0
                    );
                }
                let repaired = anyrepair::repair_binary_with_format(&data, detected.format)
                    .map_err(|e| io::Error::other(e.to_string()))?;
                (repaired, detected.format.to_string(), detected.confidence)
            }
            None => {
                eprintln!("Could not detect binary format");
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "unknown binary format",
                ));
            }
        }
    };

    let had_changes = repaired != data;

    if verbose {
        eprintln!("Repair completed");
        if had_changes {
            eprintln!("Changes detected ({} -> {} bytes)", data.len(), repaired.len());
        } else {
            eprintln!("No changes needed");
        }
    }

    if dry_run {
        eprintln!("Dry run — no output written.");
    } else {
        write_binary_output(&repaired, output)?;
    }

    // JSON summary to stderr for scripting
    eprintln!(
        r#"{{"format":"{}","detection_confidence":{},"repaired":{},"original_length":{},"repaired_length":{}}}"#,
        detected_format,
        detection_confidence,
        had_changes,
        data.len(),
        repaired.len(),
    );

    Ok(())
}

fn read_binary_input(file_path: Option<&str>) -> io::Result<Vec<u8>> {
    match file_path {
        Some(path) => fs::read(path),
        None => {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            Ok(buffer)
        }
    }
}

fn write_binary_output(data: &[u8], file_path: Option<&str>) -> io::Result<()> {
    match file_path {
        Some(path) => fs::write(path, data),
        None => {
            io::stdout().write_all(data)?;
            Ok(())
        }
    }
}
