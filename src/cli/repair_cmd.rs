//! Repair command handler

use std::io;

/// Unified repair handler for all formats.
/// When format is Some, uses that format directly via the registry.
/// When format is None, uses auto-detection.
pub fn handle_repair(
    input: Option<&str>,
    output: Option<&str>,
    show_confidence: bool,
    verbose: bool,
    format: Option<&str>,
) -> io::Result<()> {
    let content = super::read_input(input)?;

    let (repaired, confidence) = if let Some(fmt) = format {
        if verbose {
            eprintln!("Repairing content as {}...", fmt);
        }
        repair_format(&content, fmt)?
    } else {
        if verbose {
            eprintln!("Repairing content (auto-detect format)...");
        }
        let detected = anyrepair::detect_format(&content);
        if verbose
            && let Some(fmt) = detected {
                eprintln!("Detected format: {}", fmt);
            }
        match detected {
            Some(fmt) => repair_format(&content, fmt)?,
            None => {
                let repaired = anyrepair::repair(&content)
                    .map_err(|e| io::Error::other(e.to_string()))?;
                (repaired, 0.0)
            }
        }
    };

    if verbose {
        eprintln!("Repair completed");
    }

    if show_confidence {
        println!("Confidence: {:.2}%", confidence * 100.0);
    }

    super::write_output(&repaired, output)
}

/// Repair content with a specific format, returning (repaired, confidence)
fn repair_format(content: &str, format: &str) -> io::Result<(String, f64)> {
    let mut repairer = anyrepair::create_repairer(format)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
    let repaired = repairer.repair(content)
        .map_err(|e| io::Error::other(e.to_string()))?;
    let confidence = repairer.confidence(&repaired);
    Ok((repaired, confidence))
}
