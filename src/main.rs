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
    version = "1.3.0, Latest until <date-of-new-version> ", // I'll put a date here when bumping version
    about = "A custom touch implementation, written in Rust"
)]
struct Cli {
    #[arg(required = true)]
    paths: Vec<String>,

    #[arg(short, long)]
    parents: bool,

    #[arg(short = 'a', long = "access-time")]
    pub access_time: Option<String>,

    #[arg(long = "no-log", default_value_t = true, action = clap::ArgAction::SetFalse)]
    should_log: bool,
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
        Err(_) => {
            // Error details are already printed to stderr per path during execution.
            process::ExitCode::FAILURE
        }
    }
}

/// Runs the rtouch operations for all specified paths.
/// Iterates over all requested paths without stopping on individual failures,
/// allowing valid targets to be processed while capturing execution state.
fn run() -> io::Result<()> {
    let cli = Cli::parse();

    // Tracks if any single file operation or access time update fails.
    let mut has_failed = false;

    // Prepare paths for Windows or Unix
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

    // Parse access time once outside the loop.
    // Invalid time formats cause immediate failure before modifying files.
    let parsed_access_time = match &cli.access_time {
        Some(time_str) => match rtouch::datetime::parse_time_expression(time_str) {
            Ok(t) => Some(t),
            Err(parse_err) => {
                let error_message = format!("Failed to parse access time: {parse_err}");
                if touch_args.should_log {
                    logmgr::access_time_failure(&error_message).unwrap_or_else(|e| {
                        eprintln!("Failed to log access time failure: {e}");
                    });
                }
                eprintln!("{error_message}");
                return Err(io::Error::new(ErrorKind::InvalidInput, error_message));
            }
        },
        None => None,
    };

    // Unified path processing loop
    for path in &touch_args.paths {
        match create(path, touch_args.create_parents) {
            Ok(repl_res) => {
                // Log creation status
                match repl_res {
                    ReplResult::Aborted => {
                        if touch_args.should_log {
                            logmgr::success_log("Aborted a replacement of a directory in a file.")
                                .unwrap_or_else(|e| {
                                    eprintln!(
                                        "Failed to log abort status for {}: {e}",
                                        path.display()
                                    );
                                });
                        }
                        continue;
                    }
                    ReplResult::Completed => {
                        if touch_args.should_log {
                            logmgr::success_log(&format!(
                                "Replaced directory with file: {}",
                                path.display()
                            ))
                            .unwrap_or_else(|e| {
                                eprintln!("Failed to log completion for {}: {e}", path.display());
                            });
                        }
                    }
                    ReplResult::NotRequired => {
                        if touch_args.should_log {
                            let msg = if touch_args.create_parents {
                                format!("File & parent folder created: {}", path.display())
                            } else {
                                format!("File Created: {}", path.display())
                            };
                            logmgr::success_log(&msg).unwrap_or_else(|e| {
                                eprintln!("Failed to log creation for {}: {e}", path.display());
                            });
                        }
                    }
                }

                // Update access time if provided
                if let Some(access_time) = parsed_access_time {
                    if let Err(err) = rtouch::update_access_time(path, access_time) {
                        has_failed = true;
                        eprintln!("Failed to set access time for {}: {err}", path.display());

                        if touch_args.should_log {
                            let error_message =
                                format!("Failed to set access time for {}: {err}", path.display());
                            logmgr::access_time_failure(&error_message).unwrap_or_else(|e| {
                                eprintln!(
                                    "Failed to log access time failure for {}: {e}",
                                    path.display()
                                );
                            });
                        }
                    } else if touch_args.should_log {
                        logmgr::access_time_success(&format!(
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
                        logmgr::error_log(&format!(
                            "Attempted to touch directory: {}",
                            path.display()
                        ))
                    } else {
                        logmgr::error_log(&format!("Unexpected Error : {error}"))
                    };

                    log_res.unwrap_or_else(|e| {
                        eprintln!("Failed to log error for {}: {e}", path.display());
                    });
                }
            }
        }
    }

    // Return a general error if any individual file operation failed,
    // signaling process failure without prematurely terminating execution during loop processing.
    if has_failed {
        return Err(io::Error::other(
            "One or more file operations failed during execution",
        ));
    }

    Ok(())
}
