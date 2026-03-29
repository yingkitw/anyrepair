//! Validate command handler

use std::io;

pub fn handle_validate(
    input: Option<&str>,
    format: Option<&str>,
    verbose: bool,
) -> io::Result<()> {
    let content = super::read_input(input)?;
    
    if verbose {
        eprintln!("Validating content...");
    }
    
    let format_to_use = match format {
        Some(fmt) => Some(fmt),
        None => {
            let detected = anyrepair::detect_format(&content);
            if verbose
                && let Some(fmt) = detected {
                    eprintln!("Detected format: {}", fmt);
                }
            detected
        }
    };
    
    let is_valid = match format_to_use {
        Some(fmt) => {
            let validator = anyrepair::create_validator(fmt)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
            validator.is_valid(&content)
        }
        None => {
            // No format detected, try all validators
            anyrepair::SUPPORTED_FORMATS.iter().any(|fmt| {
                anyrepair::create_validator(fmt)
                    .map(|v| v.is_valid(&content))
                    .unwrap_or(false)
            })
        }
    };
    
    if is_valid {
        println!("✓ Content is valid");
        Ok(())
    } else {
        println!("✗ Content is invalid");
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Content validation failed",
        ))
    }
}
