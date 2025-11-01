//! Stream command handler for large files

use anyrepair::StreamingRepair;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};

pub fn handle_stream(
    input: Option<&str>,
    output: Option<&str>,
    format: &str,
    buffer_size: Option<usize>,
    verbose: bool,
) -> io::Result<()> {
    let buffer_size = buffer_size.unwrap_or(8192);
    
    if verbose {
        eprintln!("Streaming repair with buffer size: {} bytes", buffer_size);
        eprintln!("Format: {}", format);
    }
    
    let reader: Box<dyn io::Read> = match input {
        Some(path) => Box::new(File::open(path)?),
        None => Box::new(io::stdin()),
    };
    
    let mut writer: Box<dyn io::Write> = match output {
        Some(path) => Box::new(BufWriter::new(File::create(path)?)),
        None => Box::new(io::stdout()),
    };
    
    let buf_reader = BufReader::new(reader);
    let processor = StreamingRepair::with_buffer_size(buffer_size);
    
    match processor.process(buf_reader, &mut writer, format) {
        Ok(bytes) => {
            if verbose {
                eprintln!("Processed {} bytes", bytes);
            }
            Ok(())
        }
        Err(e) => {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Streaming repair failed: {}", e),
            ))
        }
    }
}
