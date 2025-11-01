//! Batch command handler

use std::fs;
use std::io;
use std::path::Path;

pub fn handle_batch(
    input_dir: &str,
    output_dir: &str,
    pattern: Option<&str>,
    verbose: bool,
) -> io::Result<()> {
    let pattern = pattern.unwrap_or("*");
    
    if verbose {
        eprintln!("Processing batch files from: {}", input_dir);
        eprintln!("Pattern: {}", pattern);
    }
    
    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;
    
    let entries = fs::read_dir(input_dir)?;
    let mut count = 0;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy();
            
            // Simple pattern matching
            if pattern == "*" || file_name.contains(pattern) {
                if verbose {
                    eprintln!("Processing: {}", file_name);
                }
                
                let content = fs::read_to_string(&path)?;
                let repaired = anyrepair::repair(&content)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                
                let output_path = Path::new(output_dir).join(&*file_name);
                fs::write(output_path, repaired)?;
                
                count += 1;
            }
        }
    }
    
    if verbose {
        eprintln!("Processed {} files", count);
    }
    
    println!("Processed {} files", count);
    
    Ok(())
}
