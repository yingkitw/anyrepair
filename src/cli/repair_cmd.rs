//! Repair command handler

use anyrepair::{repair, traits::Repair};
use std::io;

pub fn handle_repair(
    input: Option<&str>,
    output: Option<&str>,
    show_confidence: bool,
    verbose: bool,
    format: Option<&str>,
) -> io::Result<()> {
    let content = super::read_input(input)?;

    if let Some(fmt) = format {
        if verbose {
            eprintln!("Repairing content as {}...", fmt);
        }

        let (repaired, confidence) = match fmt.to_lowercase().as_str() {
            "json" => {
                let mut repairer = anyrepair::json::JsonRepairer::new();
                let repaired = repairer.repair(&content)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let confidence = repairer.confidence(&repaired);
                (repaired, confidence)
            }
            "yaml" | "yml" => {
                let mut repairer = anyrepair::yaml::YamlRepairer::new();
                let repaired = repairer.repair(&content)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let confidence = repairer.confidence(&repaired);
                (repaired, confidence)
            }
            "markdown" | "md" => {
                let mut repairer = anyrepair::markdown::MarkdownRepairer::new();
                let repaired = repairer.repair(&content)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let confidence = repairer.confidence(&repaired);
                (repaired, confidence)
            }
            "xml" => {
                let mut repairer = anyrepair::xml::XmlRepairer::new();
                let repaired = repairer.repair(&content)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let confidence = repairer.confidence(&repaired);
                (repaired, confidence)
            }
            "toml" => {
                let mut repairer = anyrepair::toml::TomlRepairer::new();
                let repaired = repairer.repair(&content)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let confidence = repairer.confidence(&repaired);
                (repaired, confidence)
            }
            "csv" => {
                let mut repairer = anyrepair::csv::CsvRepairer::new();
                let repaired = repairer.repair(&content)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let confidence = repairer.confidence(&repaired);
                (repaired, confidence)
            }
            "ini" => {
                let mut repairer = anyrepair::ini::IniRepairer::new();
                let repaired = repairer.repair(&content)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let confidence = repairer.confidence(&repaired);
                (repaired, confidence)
            }
            "diff" => {
                let mut repairer = anyrepair::diff::DiffRepairer::new();
                let repaired = repairer.repair(&content)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let confidence = repairer.confidence(&repaired);
                (repaired, confidence)
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unknown format: {}", fmt),
                ))
            }
        };

        if show_confidence {
            println!("Confidence: {:.2}%", confidence * 100.0);
        }

        super::write_output(&repaired, output)?;
    } else {
        if verbose {
            eprintln!("Repairing content (auto-detect format)...");
        }

        let repaired = anyrepair::repair(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        if verbose {
            eprintln!("Repair completed");
        }

        if show_confidence {
            let json_repairer = anyrepair::json::JsonRepairer::new();
            let confidence = json_repairer.confidence(&repaired);
            println!("Confidence: {:.2}%", confidence * 100.0);
        }

        super::write_output(&repaired, output)?;
    }

    Ok(())
}

pub fn handle_json(
    input: Option<&str>,
    output: Option<&str>,
    show_confidence: bool,
    verbose: bool,
) -> io::Result<()> {
    let content = super::read_input(input)?;
    
    if verbose {
        eprintln!("Repairing JSON content...");
    }
    
    let mut repairer = anyrepair::json::JsonRepairer::new();
    let repaired = repairer.repair(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    
    let confidence = repairer.confidence(&repaired);
    
    if verbose {
        eprintln!("Confidence: {:.2}%", confidence * 100.0);
    }
    
    if show_confidence {
        println!("Confidence: {:.2}%", confidence * 100.0);
    }
    
    super::write_output(&repaired, output)?;
    
    Ok(())
}

pub fn handle_yaml(
    input: Option<&str>,
    output: Option<&str>,
    show_confidence: bool,
    verbose: bool,
) -> io::Result<()> {
    let content = super::read_input(input)?;
    
    if verbose {
        eprintln!("Repairing YAML content...");
    }
    
    let mut repairer = anyrepair::yaml::YamlRepairer::new();
    let repaired = repairer.repair(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    
    let confidence = repairer.confidence(&repaired);
    
    if verbose {
        eprintln!("Confidence: {:.2}%", confidence * 100.0);
    }
    
    if show_confidence {
        println!("Confidence: {:.2}%", confidence * 100.0);
    }
    
    super::write_output(&repaired, output)?;
    
    Ok(())
}

pub fn handle_markdown(
    input: Option<&str>,
    output: Option<&str>,
    show_confidence: bool,
    verbose: bool,
) -> io::Result<()> {
    let content = super::read_input(input)?;

    if verbose {
        eprintln!("Repairing Markdown content...");
    }

    let mut repairer = anyrepair::markdown::MarkdownRepairer::new();
    let repaired = repairer
        .repair(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let confidence = repairer.confidence(&repaired);

    if verbose {
        eprintln!("Confidence: {:.2}%", confidence * 100.0);
    }

    if show_confidence {
        println!("Confidence: {:.2}%", confidence * 100.0);
    }

    super::write_output(&repaired, output)?;

    Ok(())
}

pub fn handle_xml(
    input: Option<&str>,
    output: Option<&str>,
    show_confidence: bool,
    verbose: bool,
) -> io::Result<()> {
    let content = super::read_input(input)?;

    if verbose {
        eprintln!("Repairing XML content...");
    }

    let mut repairer = anyrepair::xml::XmlRepairer::new();
    let repaired = repairer
        .repair(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let confidence = repairer.confidence(&repaired);

    if verbose {
        eprintln!("Confidence: {:.2}%", confidence * 100.0);
    }

    if show_confidence {
        println!("Confidence: {:.2}%", confidence * 100.0);
    }

    super::write_output(&repaired, output)?;

    Ok(())
}

pub fn handle_toml(
    input: Option<&str>,
    output: Option<&str>,
    show_confidence: bool,
    verbose: bool,
) -> io::Result<()> {
    let content = super::read_input(input)?;

    if verbose {
        eprintln!("Repairing TOML content...");
    }

    let mut repairer = anyrepair::toml::TomlRepairer::new();
    let repaired = repairer
        .repair(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let confidence = repairer.confidence(&repaired);

    if verbose {
        eprintln!("Confidence: {:.2}%", confidence * 100.0);
    }

    if show_confidence {
        println!("Confidence: {:.2}%", confidence * 100.0);
    }

    super::write_output(&repaired, output)?;

    Ok(())
}

pub fn handle_csv(
    input: Option<&str>,
    output: Option<&str>,
    show_confidence: bool,
    verbose: bool,
) -> io::Result<()> {
    let content = super::read_input(input)?;

    if verbose {
        eprintln!("Repairing CSV content...");
    }

    let mut repairer = anyrepair::csv::CsvRepairer::new();
    let repaired = repairer
        .repair(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let confidence = repairer.confidence(&repaired);

    if verbose {
        eprintln!("Confidence: {:.2}%", confidence * 100.0);
    }

    if show_confidence {
        println!("Confidence: {:.2}%", confidence * 100.0);
    }

    super::write_output(&repaired, output)?;

    Ok(())
}

pub fn handle_ini(
    input: Option<&str>,
    output: Option<&str>,
    show_confidence: bool,
    verbose: bool,
) -> io::Result<()> {
    let content = super::read_input(input)?;

    if verbose {
        eprintln!("Repairing INI content...");
    }

    let mut repairer = anyrepair::ini::IniRepairer::new();
    let repaired = repairer
        .repair(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let confidence = repairer.confidence(&repaired);

    if verbose {
        eprintln!("Confidence: {:.2}%", confidence * 100.0);
    }

    if show_confidence {
        println!("Confidence: {:.2}%", confidence * 100.0);
    }

    super::write_output(&repaired, output)?;

    Ok(())
}

pub fn handle_diff(
    input: Option<&str>,
    output: Option<&str>,
    show_confidence: bool,
    verbose: bool,
) -> io::Result<()> {
    let content = super::read_input(input)?;

    if verbose {
        eprintln!("Repairing Diff content...");
    }

    let mut repairer = anyrepair::diff::DiffRepairer::new();
    let repaired = repairer
        .repair(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let confidence = repairer.confidence(&repaired);

    if verbose {
        eprintln!("Confidence: {:.2}%", confidence * 100.0);
    }

    if show_confidence {
        println!("Confidence: {:.2}%", confidence * 100.0);
    }

    super::write_output(&repaired, output)?;

    Ok(())
}
