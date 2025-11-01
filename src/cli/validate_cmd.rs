//! Validate command handler

use anyrepair::traits::Validator;
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
    
    let format_to_use = format.unwrap_or("auto");
    
    let is_valid = match format_to_use {
        "json" => {
            let validator = anyrepair::json::JsonValidator;
            validator.is_valid(&content)
        }
        "yaml" => {
            let validator = anyrepair::yaml::YamlValidator;
            validator.is_valid(&content)
        }
        "markdown" => {
            let validator = anyrepair::markdown::MarkdownValidator;
            validator.is_valid(&content)
        }
        "auto" => {
            // Try to detect format and validate
            let json_validator = anyrepair::json::JsonValidator;
            if json_validator.is_valid(&content) {
                true
            } else {
                let yaml_validator = anyrepair::yaml::YamlValidator;
                if yaml_validator.is_valid(&content) {
                    true
                } else {
                    let md_validator = anyrepair::markdown::MarkdownValidator;
                    md_validator.is_valid(&content)
                }
            }
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unknown format: {}", format_to_use),
            ));
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
