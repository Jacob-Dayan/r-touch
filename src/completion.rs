// R-touch CLI application
// Copyright (C) 2026 Jacob Dayan
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Shell completion generation and automated installation routines for `rtouch`.
//!
//! This module provides utilities to automatically detect active user shells
//! (such as Bash, Zsh, Fish, PowerShell, and Elvish), generate their corresponding
//! completion scripts using `clap_complete`, and install them into standard user
//! configuration directories without requiring root / superuser privileges.

use clap::Command;
pub use clap_complete::Shell;
use std::{
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
};

/// Binary name used when generating shell completion definitions.
pub const BIN_NAME: &str = "rtouch";

/// Inspects the environment to determine the user's active shell.
///
/// Looks at the `$SHELL` environment variable on Unix-like operating systems
/// (extracting the binary name such as `bash`, `zsh`, `fish`, or `elvish`).
/// On Windows, checks for standard PowerShell environment indicators.
///
/// # Returns
///
/// * `Some(Shell)` if a supported shell is identified.
/// * `None` if the active shell cannot be determined.
///
/// # Examples
///
/// ```rust,ignore
/// if let Some(shell) = detect_shell() {
///     println!("Detected active shell: {shell}");
/// }
/// ```
#[must_use]
pub fn detect_shell() -> Option<Shell> {
    if let Ok(shell_path) = std::env::var("SHELL") {
        let path = Path::new(&shell_path);
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let name_lower = name.to_lowercase();
            match name_lower.as_str() {
                "bash" => return Some(Shell::Bash),
                "zsh" => return Some(Shell::Zsh),
                "fish" => return Some(Shell::Fish),
                "elvish" => return Some(Shell::Elvish),
                _ => {}
            }
        }
    }

    #[cfg(target_family = "windows")]
    {
        if std::env::var_os("PSModulePath").is_some() {
            return Some(Shell::PowerShell);
        }
    }

    None
}

/// Resolves the user-level target file path for a given shell's completion script.
///
/// Creates any required parent directories under the user's home directory.
///
/// # Supported Shell Locations:
/// - **Bash**: `~/.local/share/bash-completion/completions/rtouch`
/// - **Fish**: `~/.config/fish/completions/rtouch.fish`
/// - **Zsh**: `~/.zsh/completions/_rtouch` & `~/.local/share/zsh/site-functions/_rtouch`
/// - **PowerShell**: Windows PowerShell documents or `~/.config/powershell/`
/// - **Elvish**: `~/.elvish/lib/rtouch.elv`
///
/// # Errors
///
/// Returns an [`io::Error`] if the home directory cannot be located or directory
/// creation fails.
fn resolve_target_paths(home: &Path, shell: Shell) -> io::Result<Vec<PathBuf>> {
    match shell {
        Shell::Bash => {
            let dir = home.join(".local/share/bash-completion/completions");
            fs_err::create_dir_all(&dir)?;
            Ok(vec![dir.join(BIN_NAME)])
        }
        Shell::Fish => {
            let dir = home.join(".config/fish/completions");
            fs_err::create_dir_all(&dir)?;
            Ok(vec![dir.join(format!("{BIN_NAME}.fish"))])
        }
        Shell::Zsh => {
            let mut paths = Vec::with_capacity(2);
            let dir1 = home.join(".zsh/completions");
            if fs_err::create_dir_all(&dir1).is_ok() {
                paths.push(dir1.join(format!("_{BIN_NAME}")));
            }

            let dir2 = home.join(".local/share/zsh/site-functions");
            if fs_err::create_dir_all(&dir2).is_ok() {
                paths.push(dir2.join(format!("_{BIN_NAME}")));
            }

            if paths.is_empty() {
                let fallback = home.join(format!("_{BIN_NAME}"));
                paths.push(fallback);
            }
            Ok(paths)
        }
        Shell::PowerShell => {
            #[cfg(target_family = "windows")]
            let dir = home.join("Documents/WindowsPowerShell");
            #[cfg(not(target_family = "windows"))]
            let dir = home.join(".config/powershell");
            fs_err::create_dir_all(&dir)?;
            Ok(vec![dir.join(format!("{BIN_NAME}_completion.ps1"))])
        }
        Shell::Elvish => {
            let dir = home.join(".elvish/lib");
            fs_err::create_dir_all(&dir)?;
            Ok(vec![dir.join(format!("{BIN_NAME}.elv"))])
        }
        _ => {
            let dir = home.join(".local/share/completions");
            fs_err::create_dir_all(&dir)?;
            Ok(vec![dir.join(format!("{BIN_NAME}.{shell}"))])
        }
    }
}

/// Automatically installs shell completion scripts into standard user completion directories.
///
/// If `shell_opt` is `None`, this function attempts to infer the active shell
/// using [`detect_shell`]. Once the target shell is known, it generates the completion
/// script and writes it to the user's shell configuration directories.
///
/// # Arguments
///
/// * `cmd` - The [`clap::Command`] instance representing the CLI structure.
/// * `shell_opt` - Optional shell target. If `None`, automatically detected.
///
/// # Errors
///
/// * Returns [`io::ErrorKind::InvalidInput`] if the shell cannot be detected.
/// * Returns [`io::ErrorKind::NotFound`] if the user's home directory cannot be determined.
/// * Returns an [`io::Error`] if file creation or write fails.
pub fn install_completion(mut cmd: Command, shell_opt: Option<Shell>) -> io::Result<()> {
    let shell = match shell_opt.or_else(detect_shell) {
        Some(s) => s,
        None => {
            eprintln!(
                "Could not automatically detect your active shell.\n\
                 Please specify your shell explicitly: rtouch --install-completion <bash|zsh|fish|powershell|elvish>"
            );
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "Unknown shell for completion installation",
            ));
        }
    };

    let home = dirs_next::home_dir()
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "Could not determine home directory"))?;

    let target_paths = resolve_target_paths(&home, shell)?;

    let mut buf = Vec::new();
    clap_complete::generate(shell, &mut cmd, BIN_NAME, &mut buf);

    let mut successfully_written = Vec::new();
    for target in target_paths {
        if fs_err::write(&target, &buf).is_ok() {
            successfully_written.push(target);
        }
    }

    if successfully_written.is_empty() {
        return Err(io::Error::new(
            ErrorKind::PermissionDenied,
            "Failed to write completion script to target paths",
        ));
    }

    println!("✓ Successfully installed {shell} completions to:");
    for path in &successfully_written {
        println!("    {}", path.display());
    }

    match shell {
        Shell::Bash => {
            println!("\nTo activate immediately in your current terminal session, run:");
            println!("  source {}", successfully_written[0].display());
        }
        Shell::Zsh => {
            println!(
                "\nEnsure your ~/.zshrc includes the completions directory in your fpath, e.g.:"
            );
            println!("  fpath=(~/.zsh/completions $fpath)");
            println!("  autoload -Uz compinit && compinit");
        }
        Shell::Fish => {
            println!("\nFish will automatically load the completions in new sessions.");
        }
        _ => {}
    }

    Ok(())
}

/// Generates raw shell completion script directly to any stream implementing [`Write`].
///
/// This is typically used to print completions directly to standard output for
/// custom redirection or packaging scripts.
///
/// # Arguments
///
/// * `cmd` - The [`clap::Command`] instance representing the CLI structure.
/// * `shell` - The target shell for which to generate completions.
/// * `out` - Destination buffer or standard output stream.
pub fn generate_completion(mut cmd: Command, shell: Shell, out: &mut impl Write) {
    clap_complete::generate(shell, &mut cmd, BIN_NAME, out);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_completion_contains_bin_name() {
        let cmd = Command::new("rtouch");
        let mut buf = Vec::new();
        generate_completion(cmd.clone(), Shell::Bash, &mut buf);
        let output = String::from_utf8(buf).expect("valid utf-8 output");
        assert!(output.contains("rtouch"));
    }

    #[test]
    fn test_resolve_target_paths_all_shells() {
        let temp_home = std::env::temp_dir().join("rtouch_test_home");
        let shells = [
            Shell::Bash,
            Shell::Zsh,
            Shell::Fish,
            Shell::PowerShell,
            Shell::Elvish,
        ];
        for shell in shells {
            let paths = resolve_target_paths(&temp_home, shell).unwrap();
            assert!(!paths.is_empty(), "Paths should not be empty for {shell}");
        }
        let _ = fs_err::remove_dir_all(&temp_home);
    }
}
