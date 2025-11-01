//! CLI module for anyrepair
//!
//! Provides command handlers for the CLI interface

pub mod repair_cmd;
pub mod validate_cmd;
pub mod batch_cmd;
pub mod rules_cmd;
pub mod stream_cmd;

pub use repair_cmd::handle_repair;
pub use validate_cmd::handle_validate;
pub use batch_cmd::handle_batch;
pub use rules_cmd::handle_rules;
pub use stream_cmd::handle_stream;

use std::fs;
use std::io::{self, Read};

/// Read content from file or stdin
pub fn read_input(file_path: Option<&str>) -> io::Result<String> {
    match file_path {
        Some(path) => fs::read_to_string(path),
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

/// Write content to file or stdout
pub fn write_output(content: &str, file_path: Option<&str>) -> io::Result<()> {
    match file_path {
        Some(path) => fs::write(path, content),
        None => {
            print!("{}", content);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_input_from_string() {
        // This would require a temp file in real tests
        // Placeholder for now
    }
}
