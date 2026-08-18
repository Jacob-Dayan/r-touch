use clap::Parser;
use rtouch::{ReplResult, create, log::logmgr};
use std::{
    borrow::Cow,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    process,
};

/// Command line arguments parsing structure.
#[derive(Parser, Debug)]
#[command(
    name = "R-touch",
    version = "1.3.1, Latest until <date of new release>",
    about = "A custom touch implementation, written in Rust"
)]
pub struct Cli {
    #[arg(required = true)]
    pub paths: Vec<String>,

    #[arg(short, long)]
    pub parents: bool,

    #[arg(
        short = 'a',
        long = "access-time",
        num_args = 0..=1,
        default_missing_value = "now",
        require_equals = true
    )]
    pub access_time: Option<String>,

    #[arg(long = "no-log", default_value_t = true, action = clap::ArgAction::SetFalse)]
    pub should_log: bool,
}

/// Internal options passed down to business logic processing.
struct TouchArgs<'a> {
    paths: Vec<Cow<'a, Path>>,
    create_parents: bool,
    should_log: bool,
}

fn main() -> process::ExitCode {
    match run() {
        Ok(_) => process::ExitCode::SUCCESS,
        Err(_) => process::ExitCode::FAILURE,
    }
}

/// Runs the rtouch operations for all specified paths.
pub fn run() -> io::Result<()> {
    let cli = Cli::parse();

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
        should_log: cli.should_log,
    };

    let parsed_access_time = match &cli.access_time {
        Some(time_str) => match rtouch::datetime::parse_time_expression(time_str) {
            Ok(t) => Some(t),
            Err(parse_err) => {
                let error_message = format_args!("Failed to parse access time: {parse_err}");
                if touch_args.should_log {
                    logmgr::access_time_failure(&error_message).unwrap_or_else(|e| {
                        eprintln!("Failed to log access time failure: {e}");
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
        match create(path, touch_args.create_parents, parsed_access_time) {
            Ok(repl_res) => match repl_res {
                ReplResult::Aborted => {
                    if touch_args.should_log {
                        logmgr::success_log(&format_args!(
                            "Aborted a replacement of a directory in a file."
                        ))
                        .unwrap_or_else(|e| {
                            eprintln!("Failed to log abort status for {}: {e}", path.display());
                        });
                    }
                    continue;
                }
                ReplResult::Completed => {
                    if touch_args.should_log {
                        logmgr::success_log(&format_args!(
                            "Replaced directory with file: {}",
                            path.display()
                        ))
                        .unwrap_or_else(|e| {
                            eprintln!("Failed to log completion for {}: {e}", path.display());
                        });
                        if parsed_access_time.is_some() {
                            logmgr::access_time_success(&format_args!(
                                "Successfully updated access time for {}",
                                path.display()
                            ))
                            .unwrap_or_else(|e| {
                                eprintln!(
                                    "Failed to log access time success for {}: {e}",
                                    path.display()
                                );
                            });
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
                        logmgr::success_log(&message).unwrap_or_else(|e| {
                            eprintln!("Failed to log creation for {}: {e}", path.display());
                        });
                        if parsed_access_time.is_some() {
                            logmgr::access_time_success(&format_args!(
                                "Successfully updated access time for {}",
                                path.display()
                            ))
                            .unwrap_or_else(|e| {
                                eprintln!(
                                    "Failed to log access time success for {}: {e}",
                                    path.display()
                                );
                            });
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
                        logmgr::error_log(&format_args!(
                            "Attempted to touch directory: {}",
                            path.display()
                        ))
                    } else {
                        logmgr::error_log(&format_args!("Unexpected Error : {error}"))
                    };

                    log_res.unwrap_or_else(|e| {
                        eprintln!("Failed to log error for {}: {e}", path.display());
                    });
                }
            }
        }
    }

    if has_failed {
        return Err(io::Error::other(
            "One or more file operations failed during execution",
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
        assert_eq!(cli.access_time, None);
        assert!(!cli.parents);
        assert!(cli.should_log);
    }

    #[test]
    fn test_cli_parsing_access_time_flag_only() {
        let cli = Cli::try_parse_from(["rtouch", "-a", "file.txt"]).unwrap();
        assert_eq!(cli.paths, vec!["file.txt"]);
        assert_eq!(cli.access_time, Some("now".to_string()));
    }

    #[test]
    fn test_cli_parsing_long_access_time_flag_only() {
        let cli = Cli::try_parse_from(["rtouch", "--access-time", "file.txt"]).unwrap();
        assert_eq!(cli.paths, vec!["file.txt"]);
        assert_eq!(cli.access_time, Some("now".to_string()));
    }

    #[test]
    fn test_cli_parsing_access_time_with_value() {
        let cli = Cli::try_parse_from(["rtouch", "-a=2 days ago", "file.txt"]).unwrap();
        assert_eq!(cli.paths, vec!["file.txt"]);
        assert_eq!(cli.access_time, Some("2 days ago".to_string()));

        let cli2 =
            Cli::try_parse_from(["rtouch", "--access-time=2026-08-18 14:00", "file.txt"]).unwrap();
        assert_eq!(cli2.paths, vec!["file.txt"]);
        assert_eq!(cli2.access_time, Some("2026-08-18 14:00".to_string()));
    }

    #[test]
    fn test_cli_parsing_multiple_paths_with_access_time() {
        let cli = Cli::try_parse_from(["rtouch", "-a", "file1.txt", "file2.txt"]).unwrap();
        assert_eq!(cli.paths, vec!["file1.txt", "file2.txt"]);
        assert_eq!(cli.access_time, Some("now".to_string()));
    }
}
