//! Batch command handler

use std::fs;
use std::io;
use std::path::Path;

pub fn handle_batch(
    input_dir: &str,
    output_dir: &str,
    pattern: Option<&str>,
    recursive: bool,
    verbose: bool,
) -> io::Result<()> {
    let pattern = pattern.unwrap_or("*");

    if verbose {
        eprintln!("Processing batch files from: {}", input_dir);
        eprintln!("Pattern: {}", pattern);
        eprintln!("Recursive: {}", recursive);
    }

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    let mut count = 0;

    if recursive {
        // Recursive processing
        process_directory_recursive(input_dir, output_dir, pattern, verbose, &mut count)?;
    } else {
        // Single-level processing
        let entries = fs::read_dir(input_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let file_name = path.file_name()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, 
                        format!("Invalid file path: {}", path.display())))?
                    .to_string_lossy();

                // Simple pattern matching
                if pattern == "*" || file_name.contains(pattern) {
                    if verbose {
                        eprintln!("Processing: {}", file_name);
                    }

                    let content = fs::read_to_string(&path)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, 
                            format!("Failed to read {}: {}", path.display(), e)))?;
                    let repaired = anyrepair::repair(&content)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, 
                            format!("Failed to repair {}: {}", path.display(), e)))?;

                    let output_path = Path::new(output_dir).join(&*file_name);
                    fs::write(&output_path, repaired)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, 
                            format!("Failed to write {}: {}", output_path.display(), e)))?;

                    count += 1;
                }
            }
        }
    }

    if verbose {
        eprintln!("Processed {} files", count);
    }

    println!("Processed {} files", count);

    Ok(())
}

fn process_directory_recursive(
    input_dir: &str,
    output_dir: &str,
    pattern: &str,
    verbose: bool,
    count: &mut usize,
) -> io::Result<()> {
    let entries = fs::read_dir(input_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, 
                    format!("Invalid file path: {}", path.display())))?
                .to_string_lossy();

            if pattern == "*" || file_name.contains(pattern) {
                if verbose {
                    eprintln!("Processing: {}", path.display());
                }

                let content = fs::read_to_string(&path)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, 
                        format!("Failed to read {}: {}", path.display(), e)))?;
                let repaired = anyrepair::repair(&content)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, 
                        format!("Failed to repair {}: {}", path.display(), e)))?;

                // Preserve directory structure in output
                let relative_path = path
                    .strip_prefix(input_dir)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, 
                        format!("Failed to compute relative path for {}: {}", path.display(), e)))?;
                let output_path = Path::new(output_dir).join(relative_path);

                // Create parent directories if needed
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, 
                            format!("Failed to create directory {}: {}", parent.display(), e)))?;
                }

                fs::write(&output_path, repaired)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, 
                        format!("Failed to write {}: {}", output_path.display(), e)))?;

                *count += 1;
            }
        } else if path.is_dir() {
            // Recursively process subdirectories
            let dir_name = path.file_name()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, 
                    format!("Invalid directory path: {}", path.display())))?
                .to_string_lossy();
            let new_output_dir = Path::new(output_dir).join(&*dir_name);
            
            let path_str = path.to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, 
                    format!("Non-UTF8 path: {}", path.display())))?;
            let output_str = new_output_dir.to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, 
                    format!("Non-UTF8 output path: {}", new_output_dir.display())))?;
            
            process_directory_recursive(
                path_str,
                output_str,
                pattern,
                verbose,
                count,
            )?;
        }
    }

    Ok(())
}
