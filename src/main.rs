use clap::Parser;
use rtouch_core::{ReplResult, create, log::logmgr};
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
    version = "1.4.0, latest until 21th of August, 2026",
    about = "A custom touch implementation, written in Rust"
)]
pub struct Cli {
    /// File paths to touch or create.
    #[arg(required = true)]
    pub paths: Vec<String>,

    /// Create parent directories if they do not exist.
    #[arg(short, long)]
    pub parents: bool,

    /// Change only the access time.
    #[arg(short = 'a', long = "atime", alias = "access-time")]
    pub atime: bool,

    /// Change only the modification time.
    #[arg(short = 'm', long = "mtime", alias = "modification-time")]
    pub mtime: bool,

    /// Parse date string expression and use it instead of current time.
    #[arg(short = 'd', long = "date")]
    pub date: Option<String>,

    /// Disable logging to log files.
    #[arg(long = "no-log", default_value_t = true, action = clap::ArgAction::SetFalse)]
    pub should_log: bool,
}

/// Internal options passed down to business logic processing.
struct TouchArgs<'a> {
    paths: Vec<Cow<'a, Path>>,
    create_parents: bool,
    should_log: bool,
    atime: bool,
    mtime: bool,
}

/// Matches the [`run`] function and returns the appropriate exit code.
fn main() -> process::ExitCode {
    match run() {
        Ok(_) => process::ExitCode::SUCCESS,
        Err(_) => process::ExitCode::FAILURE,
    }
}

/// Runs the rtouch operations for all specified paths.
///
/// Parses the CLI arguments and processes each path.
///
/// When an error occurs, we continue to next `path` in `cli.paths` until there are no more paths to process.
/// But that creates a problem: in CLI tools where you call rtouch with `&&`,
/// you want to send a signal to the shell that tells whether the process has failed or not.
/// So, we create a mutable local boolean `has_failed` that is set to `true` when an error occurs.
/// Then, at the end, instead of returning `Ok(())`, we check `has_failed` and return `Err` if it is `true`:
///
/// ```no_run
///    // ...
///    if has_failed {
///        return Err(io::Error::other(
///            "One or more file operations failed during execution",
///        ));
///    }
///
///    Ok(())
/// ```
///
/// We create a mutable local Vector `paths` with a capacity of `cli.paths.len()` to avoid reallocations.
/// Then iterating over `cli.paths` with `for path_str in cli.paths` and replacing `/` with `\` if on Windows to normalize the path.
/// We use [`Cow::Owned`] to avoid unnecessary allocations when normalizing the path.
///
/// After normalizing the paths, we push each path into `paths`.
///
/// Then, we create a [`TouchArgs`]:
///
/// ```no_run
/// let touch_args = TouchArgs {
///     paths,
///     create_parents: cli.parents,
///     should_log: cli.should_log,
///     atime: cli.atime,
///     mtime: cli.mtime,
/// };
/// ```
///
/// Next, if a `--date` / `-d` string argument was supplied in `cli.date`, we parse it using
/// [`rtouch_core::datetime::parse_time_expression`]. If parsing fails, an error is logged (if logging is enabled),
/// an error message is printed to `stderr`, and an [`io::ErrorKind::InvalidInput`] error is returned immediately.
///
/// We then iterate through each normalized path in `touch_args.paths` and invoke [`rtouch_core::create`], passing:
/// - `path`: The path to touch or create.
/// - `create_parents`: Whether to create parent directories (`-p`, `--parents`).
/// - `parsed_date`: Optional parsed target timestamp (or current time if not specified).
/// - `atime`: Whether to update only the access time (`-a`, `--atime`).
/// - `mtime`: Whether to update only the modification time (`-m`, `--mtime`).
///
/// For each path, the result is matched:
/// - [`ReplResult::Aborted`]: If directory replacement was aborted by the user, we log the status and continue to the next path.
/// - [`ReplResult::Completed`]: If directory replacement completed, we log the success and timestamp update.
/// - [`ReplResult::NotRequired`]: If standard file creation/touch was performed, we log the creation and timestamp update.
/// - `Err(error)`: We set `has_failed = true`, print an informative error message (suggesting `-p` if the parent directory was missing, or warning if a trailing slash on a directory path caused an issue), and record the error to the log.
///
/// # Errors
///
/// Returns an [`io::Error`] if:
/// - A date expression provided via `--date` fails to parse ([`io::ErrorKind::InvalidInput`]).
/// - One or more file operations failed during the touch execution loop.
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
        atime: cli.atime,
        mtime: cli.mtime,
    };

    let parsed_date = match &cli.date {
        Some(time_str) => match rtouch_core::datetime::parse_time_expression(time_str) {
            Ok(t) => Some(t),
            Err(parse_err) => {
                let error_message = format_args!("Failed to parse date expression: {parse_err}");
                if touch_args.should_log {
                    logmgr::access_time_failure(&error_message).unwrap_or_else(|e| {
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
        match create(
            path,
            touch_args.create_parents,
            parsed_date,
            touch_args.atime,
            touch_args.mtime,
        ) {
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
                        if parsed_date.is_some() || touch_args.atime || touch_args.mtime {
                            logmgr::access_time_success(&format_args!(
                                "Successfully updated timestamp for {}",
                                path.display()
                            ))
                            .unwrap_or_else(|e| {
                                eprintln!(
                                    "Failed to log timestamp success for {}: {e}",
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
                        if parsed_date.is_some() || touch_args.atime || touch_args.mtime {
                            logmgr::access_time_success(&format_args!(
                                "Successfully updated timestamp for {}",
                                path.display()
                            ))
                            .unwrap_or_else(|e| {
                                eprintln!(
                                    "Failed to log timestamp success for {}: {e}",
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
        assert_eq!(cli.date, None);
        assert!(!cli.parents);
        assert!(!cli.atime);
        assert!(!cli.mtime);
        assert!(cli.should_log);
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

        let cli2 =
            Cli::try_parse_from(["rtouch", "--date=2026-08-18 14:00", "file.txt"]).unwrap();
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
