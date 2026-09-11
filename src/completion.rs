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

//! shell completion generation and automated installation routines for `rtouch`
//!
//! utilities to automatically detect active user shells, generate completion scripts
//! using `clap_complete`, and install them into standard user configuration directories

use clap::Command;
use fs_err as fs;
use std::{
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
};

/// supported shell targets for completion script generation and installation
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, clap::ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    #[value(alias = "pwsh7", alias = "pwsh-7")]
    Pwsh,
    Zsh,
}

impl Shell {
    /// convert to [`clap_complete::Shell`]
    #[must_use]
    pub const fn to_clap_shell(self) -> clap_complete::Shell {
        match self {
            Self::Bash => clap_complete::Shell::Bash,
            Self::Elvish => clap_complete::Shell::Elvish,
            Self::Fish => clap_complete::Shell::Fish,
            Self::PowerShell | Self::Pwsh => clap_complete::Shell::PowerShell,
            Self::Zsh => clap_complete::Shell::Zsh,
        }
    }
}

impl From<Shell> for clap_complete::Shell {
    fn from(shell: Shell) -> Self {
        shell.to_clap_shell()
    }
}

impl clap_complete::Generator for Shell {
    fn file_name(&self, name: &str) -> String {
        self.to_clap_shell().file_name(name)
    }

    fn generate(&self, cmd: &Command, buf: &mut dyn Write) {
        self.to_clap_shell().generate(cmd, buf)
    }
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bash => write!(f, "bash"),
            Self::Elvish => write!(f, "elvish"),
            Self::Fish => write!(f, "fish"),
            Self::PowerShell => write!(f, "powershell"),
            Self::Pwsh => write!(f, "pwsh"),
            Self::Zsh => write!(f, "zsh"),
        }
    }
}

/// binary name used when generating shell completion definitions
pub const BIN_NAME: &str = "rtouch";

/// inspect environment to determine active shell
///
/// checks `$SHELL` on Unix (extracting binary name like `bash`, `zsh`, `fish`, `elvish`, `pwsh`)
/// or standard PowerShell environment indicators on Windows
///
/// # Returns
///
/// * `Some([`Shell`])` if a supported shell is identified
/// * `None` if active shell cannot be determined
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
                "pwsh" => return Some(Shell::Pwsh),
                "powershell" => return Some(Shell::PowerShell),
                _ => {}
            }
        }
    }

    #[cfg(target_family = "windows")]
    {
        if let Ok(ps_module_path) = std::env::var("PSModulePath") {
            let is_pwsh = std::env::var_os("POWERSHELL_DISTRIBUTION_CHANNEL").is_some()
                || ps_module_path.split(';').any(|seg| {
                    let seg_lower = seg.to_lowercase();
                    seg_lower.contains(r"\powershell\7")
                        || seg_lower.contains(r"/powershell/7")
                        || (seg_lower.contains(r"\powershell\modules")
                            && !seg_lower.contains(r"windowspowershell"))
                        || (seg_lower.contains(r"/powershell/modules")
                            && !seg_lower.contains(r"windowspowershell"))
                });
            if is_pwsh {
                return Some(Shell::Pwsh);
            }
            return Some(Shell::PowerShell);
        }
    }

    None
}

/// resolve user-level target file path for a given shell completion script
///
/// creates any required parent directories under user's home directory
///
/// # Supported Shell Locations:
/// - **Bash**: `~/.local/share/bash-completion/completions/rtouch`
/// - **Fish**: `~/.config/fish/completions/rtouch.fish`
/// - **Zsh**: `~/.zsh/completions/_rtouch` & `~/.local/share/zsh/site-functions/_rtouch`
/// - **PowerShell**: Windows PowerShell documents, PowerShell Core documents, or `~/.config/powershell/`
/// - **PowerShell Core (pwsh)**: `~/Documents/PowerShell/` on Windows or `~/.config/powershell/` on Unix
/// - **Elvish**: `~/.elvish/lib/rtouch.elv`
///
/// # Errors
///
/// returns an [`io::Error`] if home directory cannot be located or directory creation fails
fn resolve_target_paths(home: &Path, shell: Shell) -> io::Result<Vec<PathBuf>> {
    match shell {
        Shell::Bash => {
            let dir = home.join(".local/share/bash-completion/completions");
            fs::create_dir_all(&dir)?;
            Ok(vec![dir.join(BIN_NAME)])
        }
        Shell::Fish => {
            let dir = home.join(".config/fish/completions");
            fs::create_dir_all(&dir)?;
            Ok(vec![dir.join(format!("{BIN_NAME}.fish"))])
        }
        Shell::Zsh => {
            let mut paths = Vec::with_capacity(2);
            let dir1 = home.join(".zsh/completions");
            if fs::create_dir_all(&dir1).is_ok() {
                paths.push(dir1.join(format!("_{BIN_NAME}")));
            }

            let dir2 = home.join(".local/share/zsh/site-functions");
            if fs::create_dir_all(&dir2).is_ok() {
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
            {
                let mut paths = Vec::with_capacity(2);
                let win_ps = home.join("Documents/WindowsPowerShell");
                if fs::create_dir_all(&win_ps).is_ok() {
                    paths.push(win_ps.join(format!("{BIN_NAME}_completion.ps1")));
                }
                let pwsh = home.join("Documents/PowerShell");
                if fs::create_dir_all(&pwsh).is_ok() {
                    paths.push(pwsh.join(format!("{BIN_NAME}_completion.ps1")));
                }
                if paths.is_empty() {
                    paths.push(win_ps.join(format!("{BIN_NAME}_completion.ps1")));
                }
                Ok(paths)
            }
            #[cfg(not(target_family = "windows"))]
            {
                let dir = home.join(".config/powershell");
                fs::create_dir_all(&dir)?;
                Ok(vec![dir.join(format!("{BIN_NAME}_completion.ps1"))])
            }
        }
        Shell::Pwsh => {
            #[cfg(target_family = "windows")]
            let dir = home.join("Documents/PowerShell");
            #[cfg(not(target_family = "windows"))]
            let dir = home.join(".config/powershell");
            fs::create_dir_all(&dir)?;
            Ok(vec![dir.join(format!("{BIN_NAME}_completion.ps1"))])
        }
        Shell::Elvish => {
            let dir = home.join(".elvish/lib");
            fs::create_dir_all(&dir)?;
            Ok(vec![dir.join(format!("{BIN_NAME}.elv"))])
        }
    }
}

/// automatically install shell completion scripts into standard user completion directories
///
/// infers active shell via [`detect_shell`] if `shell_opt` is `None`, then generates
/// completion script and writes to user shell configuration directories
///
/// # Arguments
///
/// * `cmd` - [`clap::Command`] instance representing CLI structure
/// * `shell_opt` - optional shell target (inferred if `None`)
///
/// # Errors
///
/// * returns [`io::ErrorKind::InvalidInput`] if shell cannot be detected
/// * returns [`io::ErrorKind::NotFound`] if user home directory cannot be determined
/// * returns an [`io::Error`] if file creation or write fails
pub fn install_completion(mut cmd: Command, shell_opt: Option<Shell>) -> io::Result<()> {
    let shell = match shell_opt.or_else(detect_shell) {
        Some(s) => s,
        None => {
            eprintln!(
                "Could not automatically detect your active shell.\n\
                 Please specify your shell explicitly: rtouch --install-completion <bash|zsh|fish|powershell|pwsh|elvish>"
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
        if fs::write(&target, &buf).is_ok() {
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

/// generate raw shell completion script directly to any stream implementing [`Write`]
///
/// typically used to print completions directly to standard output for
/// custom redirection or packaging scripts
///
/// # Arguments
///
/// * `cmd` - [`clap::Command`] instance representing CLI structure
/// * `shell` - target shell for which to generate completions
/// * `out` - destination buffer or standard output stream
pub fn generate_completion(mut cmd: Command, shell: Shell, out: &mut impl Write) {
    clap_complete::generate(shell, &mut cmd, BIN_NAME, out);
}

/// check if completion files already exist for whatever shell the user is running
#[must_use]
pub fn are_completions_installed() -> bool {
    let Some(shell) = detect_shell() else {
        return true;
    };
    let Some(home) = dirs_next::home_dir() else {
        return true;
    };
    let Ok(paths) = resolve_target_paths(&home, shell) else {
        return true;
    };
    paths.iter().any(|p| p.exists())
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
    fn test_generate_completion_pwsh() {
        let cmd = Command::new("rtouch");
        let mut buf = Vec::new();
        generate_completion(cmd.clone(), Shell::Pwsh, &mut buf);
        let output = String::from_utf8(buf).expect("valid utf-8 output");
        assert!(output.contains("rtouch"));
        assert!(output.contains("Register-ArgumentCompleter"));
    }

    #[test]
    fn test_resolve_target_paths_all_shells() {
        let temp_home = std::env::temp_dir().join("rtouch_test_home");
        let shells = [
            Shell::Bash,
            Shell::Zsh,
            Shell::Fish,
            Shell::PowerShell,
            Shell::Pwsh,
            Shell::Elvish,
        ];
        for shell in shells {
            let paths = resolve_target_paths(&temp_home, shell).unwrap();
            assert!(!paths.is_empty(), "Paths should not be empty for {shell}");
        }
        let _ = fs::remove_dir_all(&temp_home);
    }

    #[test]
    fn test_are_completions_installed() {
        let temp_home = std::env::temp_dir().join("rtouch_test_installed_check");
        let _ = fs::remove_dir_all(&temp_home);
        let paths = resolve_target_paths(&temp_home, Shell::Bash).unwrap();
        assert!(!paths.iter().any(|p| p.exists()));

        fs::write(&paths[0], b"mock completion").unwrap();
        assert!(paths.iter().any(|p| p.exists()));
        let _ = fs::remove_dir_all(&temp_home);
    }
}
