//! Repair command handler

use anyrepair::{repair, traits::Repair};
use std::io;

pub fn handle_repair(
    input: Option<&str>,
    output: Option<&str>,
    show_confidence: bool,
    verbose: bool,
) -> io::Result<()> {
    let content = super::read_input(input)?;
    
    if verbose {
        eprintln!("Repairing content (auto-detect format)...");
    }
    
    let repaired = anyrepair::repair(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    
    if verbose {
        eprintln!("Repair completed");
    }
    
    if show_confidence {
        let mut json_repairer = anyrepair::json::JsonRepairer::new();
        let confidence = json_repairer.confidence(&repaired);
        println!("Confidence: {:.2}%", confidence * 100.0);
    }
    
    super::write_output(&repaired, output)?;
    
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
