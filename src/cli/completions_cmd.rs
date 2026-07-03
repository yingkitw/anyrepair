//! Shell completions command handler

use std::io;

use clap::CommandFactory;
use clap_complete::{generate, Shell};

/// Generate shell completions for the given shell and print to stdout.
pub fn handle_completions(shell: &str) -> io::Result<()> {
    let shell_enum = match shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "elvish" => Shell::Elvish,
        "powershell" | "pwsh" => Shell::PowerShell,
        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Unsupported shell '{}'. Supported: bash, zsh, fish, elvish, powershell",
                    other
                ),
            ));
        }
    };

    let mut cmd = crate::Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(shell_enum, &mut cmd, bin_name, &mut io::stdout());
    Ok(())
}
