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

mod completion;

use clap::{CommandFactory, Parser};
use completion::Shell;
use rtouch::{ReplResult, log::logmgr, replace_dir, touch};
use std::{
    borrow::Cow,
    ffi::OsString,
    io::{self, ErrorKind, Read},
    path::{Path, PathBuf},
    process,
    sync::LazyLock,
};

/// command line arguments parsing structure
#[derive(Parser, Debug)]
#[command(
    name = "R-touch",
    version = "1.6.0",
    about = "A custom touch implementation, written in Rust"
)]
pub struct Cli {
    /// file paths to touch or create
    #[arg(required_unless_present_any = ["install_completion", "generate_completion"])]
    pub paths: Vec<String>,

    /// create parent directories if they do not exist
    #[arg(short, long)]
    pub parents: bool,

    /// replace an existing directory with an empty file
    #[arg(short = 'r', long = "replace-directory")]
    pub replace_directory: bool,

    /// force deletion of a non-empty directory when replacing it
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// change only the access time
    #[arg(short = 'a', long = "atime", alias = "access-time")]
    pub atime: bool,

    /// change only the modification time
    #[arg(short = 'm', long = "mtime", alias = "modification-time")]
    pub mtime: bool,

    /// parse date string expression and use it instead of current time
    #[arg(short = 'd', long = "date", allow_hyphen_values = true)]
    pub date: Option<String>,

    /// disable logging to log files
    #[arg(long = "no-log", default_value_t = true, action = clap::ArgAction::SetFalse)]
    pub should_log: bool,

    /// force enable logging to log files (overrides config)
    #[arg(long = "log")]
    pub force_log: bool,

    /// custom directory to store log files
    #[arg(long = "log-dir", value_name = "DIR")]
    pub log_dir: Option<PathBuf>,

    /// automatically install shell completions into the appropriate shell directory
    #[arg(
        long = "install-completion",
        alias = "completion",
        value_enum,
        num_args = 0..=1,
        default_missing_value = None
    )]
    pub install_completion: Option<Option<Shell>>,

    /// print raw shell completion script directly to stdout
    #[arg(long = "generate-completion", value_enum, hide = true)]
    pub generate_completion: Option<Shell>,
}

/// internal options passed down to business logic processing
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

// default [`LogConfig`] for the binary, computed once at startup via LazyLock
// so app-specific directory name stays in binary while library stays reusable
static DEFAULT_LOG_CONFIG: LazyLock<rtouch::LogConfig> =
    LazyLock::new(|| rtouch::LogConfig::from_env_defaults_for(APP_NAME));

fn main() -> process::ExitCode {
    match run(&DEFAULT_LOG_CONFIG) {
        Ok(_) => process::ExitCode::SUCCESS,
        Err(_) => process::ExitCode::FAILURE,
    }
}

/// shortcut for `std::io::Error::new(std::io::ErrorKind::Other, e)`
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

/// run R-touch operations for all specified paths
pub fn run(cfg: &rtouch::LogConfig) -> io::Result<()> {
    let cli = Cli::parse_from(normalize_cli_args(std::env::args_os()));

    let is_first_startup = rtouch::conf::default_config_path().is_some_and(|p| !p.exists());

    let mut config = match rtouch::conf::load_default() {
        Ok(Some(c)) => c,
        _ => rtouch::conf::AppConfig::default(),
    };

    let custom_log_cfg;
    let cfg = if let Some(ref dir) = cli.log_dir {
        custom_log_cfg = rtouch::LogConfig::from_log_dir(dir);
        &custom_log_cfg
    } else if let Some(ref dir) = config.log_dir {
        custom_log_cfg = rtouch::LogConfig::from_log_dir(dir);
        &custom_log_cfg
    } else {
        cfg
    };

    cfg.ensure_permissions();

    if let Some(shell_opt) = cli.install_completion {
        if is_first_startup {
            config.completions = true;
            let _ = rtouch::conf::save_default(&config);
        }
        return completion::install_completion(Cli::command(), shell_opt);
    }

    if let Some(shell) = cli.generate_completion {
        completion::generate_completion(Cli::command(), shell, &mut io::stdout());
        return Ok(());
    }

    if is_first_startup {
        if completion::are_completions_installed() {
            config.completions = true;
        } else {
            eprintln!("Do you want to install shell completions automatically? (y/n)");
            let install = read_confirmation();
            config.completions = install;
            if install {
                let _ = completion::install_completion(Cli::command(), None);
            }
        }
        let _ = rtouch::conf::save_default(&config);
    }

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

    let should_log = if cli.force_log {
        true
    } else if !cli.should_log {
        false
    } else {
        config.should_log
    };

    let (atime, mtime) = config.time_modify.resolve_flags(cli.atime, cli.mtime);

    let touch_args = TouchArgs {
        paths,
        create_parents: cli.parents,
        replace_directory: cli.replace_directory,
        force: cli.force,
        should_log,
        atime,
        mtime,
    };

    // describe which timestamps will be updated (if both flags are false, update both)
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
            replace_dir::replace_with_force(path, touch_args.force, || {
                prompt_replace_directory(path)
            })
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
                    eprintln!("Abort");
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

fn parse_confirmation(mut reader: impl Read) -> bool {
    let mut buf = [0u8; 4];
    let n = match reader.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return true,
    };
    let first = buf[..n].iter().copied().find(|b| !b.is_ascii_whitespace());
    !matches!(first, Some(b'n' | b'N'))
}

fn read_confirmation() -> bool {
    parse_confirmation(io::stdin())
}

fn prompt_replace_directory(path: &Path) -> bool {
    eprintln!(
        "'{p}' is a directory. Do you want to delete directory and replace it with the file? (y/n)",
        p = path.display()
    );
    read_confirmation()
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

    #[test]
    fn test_cli_parsing_completion_flag() {
        let cli_no_val = Cli::try_parse_from(["rtouch", "--install-completion"]).unwrap();
        assert_eq!(cli_no_val.install_completion, Some(None));
        assert!(cli_no_val.paths.is_empty());

        let cli = Cli::try_parse_from(["rtouch", "--completion", "bash"]).unwrap();
        assert_eq!(cli.install_completion, Some(Some(Shell::Bash)));
        assert!(cli.paths.is_empty());

        let cli_zsh = Cli::try_parse_from(["rtouch", "--completion=zsh"]).unwrap();
        assert_eq!(cli_zsh.install_completion, Some(Some(Shell::Zsh)));
        assert!(cli_zsh.paths.is_empty());

        let cli_gen = Cli::try_parse_from(["rtouch", "--generate-completion", "fish"]).unwrap();
        assert_eq!(cli_gen.generate_completion, Some(Shell::Fish));
        assert!(cli_gen.paths.is_empty());
    }

    #[test]
    fn test_generate_completion_output() {
        let mut buf = Vec::new();
        clap_complete::generate(Shell::Bash, &mut Cli::command(), "rtouch", &mut buf);
        let output = String::from_utf8(buf).expect("valid utf8 completion output");
        assert!(output.contains("rtouch"));
        assert!(output.contains("--parents"));
        assert!(output.contains("--replace-directory"));
        assert!(output.contains("--atime"));
        assert!(output.contains("--mtime"));
        assert!(output.contains("--date"));
        assert!(output.contains("--no-log"));
        assert!(output.contains("--log-dir"));
        assert!(output.contains("--install-completion"));
    }

    #[test]
    fn test_cli_parsing_log_dir_flag() {
        let cli = Cli::try_parse_from(["rtouch", "--log-dir", "/custom/logs", "file.txt"]).unwrap();
        assert_eq!(cli.paths, vec!["file.txt"]);
        assert_eq!(cli.log_dir, Some(PathBuf::from("/custom/logs")));

        let cli2 = Cli::try_parse_from(["rtouch", "--log-dir=/another/path", "file.txt"]).unwrap();
        assert_eq!(cli2.log_dir, Some(PathBuf::from("/another/path")));
    }

    #[test]
    fn test_parse_confirmation_starts_with_n_declines() {
        assert!(!parse_confirmation(&b"n"[..]));
        assert!(!parse_confirmation(&b"N"[..]));
        assert!(!parse_confirmation(&b"no"[..]));
        assert!(!parse_confirmation(&b"NO"[..]));
        assert!(!parse_confirmation(&b"No\n"[..]));
        assert!(!parse_confirmation(&b"   n"[..]));
        assert!(!parse_confirmation(&b" \tn"[..]));
        assert!(!parse_confirmation(&b"never\n"[..]));
    }

    #[test]
    fn test_parse_confirmation_other_inputs_accept() {
        assert!(parse_confirmation(&b"y"[..]));
        assert!(parse_confirmation(&b"Y"[..]));
        assert!(parse_confirmation(&b"yes"[..]));
        assert!(parse_confirmation(&b"YES"[..]));
        assert!(parse_confirmation(&b"\n"[..]));
        assert!(parse_confirmation(&b"\r\n"[..]));
        assert!(parse_confirmation(&b"   \n"[..]));
        assert!(parse_confirmation(&b""[..]));
        assert!(parse_confirmation(&b"ok"[..]));
        assert!(parse_confirmation(&b"sure"[..]));
    }
}
