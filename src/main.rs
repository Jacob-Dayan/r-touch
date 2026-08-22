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

use clap::Parser;
use rtouch::{ReplResult, log::logmgr, replace_dir, touch};
use std::{
    borrow::Cow,
    ffi::OsString,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    process,
    sync::LazyLock,
};

/// Command line arguments parsing structure.
#[derive(Parser, Debug)]
#[command(
    name = "R-touch",
    version = "1.5.1, patch of 1.5.0",
    about = "A custom touch implementation, written in Rust"
)]
pub struct Cli {
    /// File paths to touch or create.
    #[arg(required = true)]
    pub paths: Vec<String>,

    /// Create parent directories if they do not exist.
    #[arg(short, long)]
    pub parents: bool,

    /// Replace an existing directory with an empty file.
    #[arg(short = 'r', long = "replace-directory")]
    pub replace_directory: bool,

    /// Force deletion of a non-empty directory when replacing it.
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Change only the access time.
    #[arg(short = 'a', long = "atime", alias = "access-time")]
    pub atime: bool,

    /// Change only the modification time.
    #[arg(short = 'm', long = "mtime", alias = "modification-time")]
    pub mtime: bool,

    /// Parse date string expression and use it instead of current time.
    #[arg(short = 'd', long = "date", allow_hyphen_values = true)]
    pub date: Option<String>,

    /// Disable logging to log files.
    #[arg(long = "no-log", default_value_t = true, action = clap::ArgAction::SetFalse)]
    pub should_log: bool,
}

/// Internal options passed down to business logic processing.
struct TouchArgs<'a> {
    paths: Vec<Cow<'a, Path>>,
    create_parents: bool,
    replace_directory: bool,
    force: bool,
    should_log: bool,
    atime: bool,
    mtime: bool,
}

const APP_NAME: &str = "R-touch";

// Default LogConfig for the binary. Using a LazyLock ensures the default
// paths are computed once at startup and can be referenced throughout the
// process lifetime. The app-specific directory name is kept here so the library
// remains reusable for other crates and applications.
static DEFAULT_LOG_CONFIG: LazyLock<rtouch::LogConfig> =
    LazyLock::new(|| rtouch::LogConfig::from_env_defaults_for(APP_NAME));

/// Matches the [`run`] function and returns the appropriate exit code.
fn main() -> process::ExitCode {
    match run(&DEFAULT_LOG_CONFIG) {
        Ok(_) => process::ExitCode::SUCCESS,
        Err(_) => process::ExitCode::FAILURE,
    }
}

/// Shortcut for `std::io::Error::new(std::io::ErrorKind::Other, e)`
macro_rules! new_io_error {
    ($e:expr) => {
        std::io::Error::new(std::io::ErrorKind::Other, $e)
    };
}

fn normalize_cli_args(args: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    args.into_iter()
        .map(|arg| {
            let arg_str = arg.to_string_lossy();
            if arg_str == "-rd" {
                OsString::from("--replace-directory")
            } else {
                arg
            }
        })
        .collect()
}

/// Runs the rtouch operations for all specified paths.
///
/// Parses the CLI arguments and processes each path.
pub fn run(cfg: &rtouch::LogConfig) -> io::Result<()> {
    let cli = Cli::parse_from(normalize_cli_args(std::env::args_os()));

    let mut has_failed = false;

    let mut paths = Vec::with_capacity(cli.paths.len());
    for path_str in cli.paths {
        #[cfg(target_family = "windows")]
        {
            if path_str.contains('/') {
                paths.push(Cow::Owned(PathBuf::from(path_str.replace('/', "\\"))));
                continue;
            }
        }
        paths.push(Cow::Owned(PathBuf::from(path_str)));
    }

    let touch_args = TouchArgs {
        paths,
        create_parents: cli.parents,
        replace_directory: cli.replace_directory,
        force: cli.force,
        should_log: cli.should_log,
        atime: cli.atime,
        mtime: cli.mtime,
    };

    // `updated_atime` and `updated_mtime` describe which timestamps will be
    // updated by each touch call. If both flags are false, we update both
    // access and modification times.
    let updated_atime = touch_args.atime || (!touch_args.atime && !touch_args.mtime);
    let updated_mtime = touch_args.mtime || (!touch_args.atime && !touch_args.mtime);

    let parsed_date = match &cli.date {
        Some(time_str) => match rtouch::datetime::parse_time_expression(time_str) {
            Ok(t) => Some(t),
            Err(parse_err) => {
                let error_message = format_args!("Failed to parse date expression: {parse_err}");
                if touch_args.should_log {
                    logmgr::time_modification_failure(cfg, &error_message).unwrap_or_else(|e| {
                        eprintln!("Failed to log date parsing failure: {e}");
                    });
                }
                eprintln!("{error_message}");
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    error_message.to_string(),
                ));
            }
        },
        None => None,
    };

    for path in &touch_args.paths {
        let result = if path.is_dir() && touch_args.replace_directory {
            replace_dir::replace_with_force(path, touch_args.force)
        } else {
            touch(
                path,
                touch_args.create_parents,
                parsed_date,
                touch_args.atime,
                touch_args.mtime,
            )
        };

        match result {
            Ok(repl_res) => match repl_res {
                ReplResult::Aborted => {
                    if touch_args.should_log {
                        logmgr::success_log(
                            cfg,
                            &format_args!("Aborted a replacement of a directory in a file."),
                        )
                        .unwrap_or_else(|e| {
                            eprintln!("Failed to log abort status for {}: {e}", path.display());
                        });
                    }
                    continue;
                }
                ReplResult::Completed => {
                    if touch_args.should_log {
                        logmgr::success_log(
                            cfg,
                            &format_args!("Replaced directory with file: {}", path.display()),
                        )
                        .unwrap_or_else(|e| {
                            eprintln!("Failed to log completion for {}: {e}", path.display());
                        });

                        if parsed_date.is_some() || touch_args.atime || touch_args.mtime {
                            if updated_atime {
                                logmgr::atime_modification_success(
                                    cfg,
                                    &format_args!(
                                        "Successfully updated access time for {}",
                                        path.display()
                                    ),
                                )
                                .unwrap_or_else(|e| {
                                    eprintln!(
                                        "Failed to log atime success for {}: {e}",
                                        path.display()
                                    );
                                });
                            }
                            if updated_mtime {
                                logmgr::mtime_modification_success(
                                    cfg,
                                    &format_args!(
                                        "Successfully updated modification time for {}",
                                        path.display()
                                    ),
                                )
                                .unwrap_or_else(|e| {
                                    eprintln!(
                                        "Failed to log mtime success for {}: {e}",
                                        path.display()
                                    );
                                });
                            }
                        }
                    }
                }
                ReplResult::NotRequired => {
                    if touch_args.should_log {
                        let message = if touch_args.create_parents {
                            format_args!("File & parent folder created: {}", path.display())
                        } else {
                            format_args!("File Created: {}", path.display())
                        };
                        logmgr::success_log(cfg, &message).unwrap_or_else(|e| {
                            eprintln!("Failed to log creation for {}: {e}", path.display());
                        });

                        if parsed_date.is_some() || touch_args.atime || touch_args.mtime {
                            if updated_atime {
                                logmgr::atime_modification_success(
                                    cfg,
                                    &format_args!(
                                        "Successfully updated access time for {}",
                                        path.display()
                                    ),
                                )
                                .unwrap_or_else(|e| {
                                    eprintln!(
                                        "Failed to log atime success for {}: {e}",
                                        path.display()
                                    );
                                });
                            }
                            if updated_mtime {
                                logmgr::mtime_modification_success(
                                    cfg,
                                    &format_args!(
                                        "Successfully updated modification time for {}",
                                        path.display()
                                    ),
                                )
                                .unwrap_or_else(|e| {
                                    eprintln!(
                                        "Failed to log mtime success for {}: {e}",
                                        path.display()
                                    );
                                });
                            }
                        }
                    }
                }
            },
            Err(error) => {
                has_failed = true;

                match error.kind() {
                    ErrorKind::NotFound => {
                        eprintln!(
                            "Unexpected Error: {error}.\nIf attempted to create a parent directory, consider running with `-p`."
                        );
                    }
                    ErrorKind::IsADirectory => {
                        eprintln!(
                            "Error: {error}\nconsider removing the '/' char at the end of the path."
                        );
                    }
                    _ => {
                        eprintln!("{error}");
                    }
                }

                if touch_args.should_log {
                    let log_res = if error.kind() == ErrorKind::IsADirectory {
                        logmgr::error_log(
                            cfg,
                            &format_args!("Attempted to touch directory: {}", path.display()),
                        )
                    } else {
                        logmgr::error_log(cfg, &format_args!("Unexpected Error : {error}"))
                    };

                    log_res.unwrap_or_else(|e| {
                        eprintln!("Failed to log error for {}: {e}", path.display());
                    });
                }
            }
        }
    }

    if has_failed {
        return Err(new_io_error!(
            "One or more file operations failed during execution"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_no_flags() {
        let cli = Cli::try_parse_from(["rtouch", "file.txt"]).unwrap();
        assert_eq!(cli.paths, vec!["file.txt"]);
        assert_eq!(cli.date, None);
        assert!(!cli.parents);
        assert!(!cli.replace_directory);
        assert!(!cli.force);
        assert!(!cli.atime);
        assert!(!cli.mtime);
        assert!(cli.should_log);
    }

    #[test]
    fn test_cli_parsing_replace_directory_flag() {
        let cli = Cli::parse_from(normalize_cli_args([
            OsString::from("rtouch"),
            OsString::from("-rd"),
            OsString::from("dir"),
        ]));
        assert_eq!(cli.paths, vec!["dir"]);
        assert!(cli.replace_directory);

        let cli2 = Cli::try_parse_from(["rtouch", "-r", "dir"]).unwrap();
        assert!(cli2.replace_directory);

        let cli3 = Cli::try_parse_from(["rtouch", "--replace-directory", "dir"]).unwrap();
        assert!(cli3.replace_directory);
    }

    #[test]
    fn test_cli_parsing_force_flag() {
        let cli = Cli::try_parse_from(["rtouch", "-f", "dir"]).unwrap();
        assert!(cli.force);
        assert!(!cli.replace_directory);
    }

    #[test]
    fn test_cli_parsing_atime_flag() {
        let cli = Cli::try_parse_from(["rtouch", "-a", "file.txt"]).unwrap();
        assert_eq!(cli.paths, vec!["file.txt"]);
        assert!(cli.atime);
        assert!(!cli.mtime);
        assert_eq!(cli.date, None);

        let cli2 = Cli::try_parse_from(["rtouch", "--atime", "file.txt"]).unwrap();
        assert!(cli2.atime);

        let cli3 = Cli::try_parse_from(["rtouch", "--access-time", "file.txt"]).unwrap();
        assert!(cli3.atime);
    }

    #[test]
    fn test_cli_parsing_mtime_flag() {
        let cli = Cli::try_parse_from(["rtouch", "-m", "file.txt"]).unwrap();
        assert_eq!(cli.paths, vec!["file.txt"]);
        assert!(!cli.atime);
        assert!(cli.mtime);
        assert_eq!(cli.date, None);

        let cli2 = Cli::try_parse_from(["rtouch", "--mtime", "file.txt"]).unwrap();
        assert!(cli2.mtime);

        let cli3 = Cli::try_parse_from(["rtouch", "--modification-time", "file.txt"]).unwrap();
        assert!(cli3.mtime);
    }

    #[test]
    fn test_cli_parsing_date_flag() {
        let cli = Cli::try_parse_from(["rtouch", "-d", "2 days ago", "file.txt"]).unwrap();
        assert_eq!(cli.paths, vec!["file.txt"]);
        assert_eq!(cli.date, Some("2 days ago".to_string()));

        let cli2 = Cli::try_parse_from(["rtouch", "--date=2026-08-18 14:00", "file.txt"]).unwrap();
        assert_eq!(cli2.paths, vec!["file.txt"]);
        assert_eq!(cli2.date, Some("2026-08-18 14:00".to_string()));
    }

    #[test]
    fn test_cli_parsing_combined_flags() {
        let cli = Cli::try_parse_from([
            "rtouch",
            "-a",
            "-m",
            "-d",
            "yesterday",
            "file1.txt",
            "file2.txt",
        ])
        .unwrap();
        assert_eq!(cli.paths, vec!["file1.txt", "file2.txt"]);
        assert!(cli.atime);
        assert!(cli.mtime);
        assert_eq!(cli.date, Some("yesterday".to_string()));
    }
}
