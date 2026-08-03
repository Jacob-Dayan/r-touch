use clap::Parser;
use rtouch::{ReplResult, create, log::logmgr};
use std::{
    borrow::Cow,
    io,
    path::{Path, PathBuf},
};

// Command line arguments parsing
#[derive(Parser, Debug)]
#[command(
    name = "R-touch",
    version = "1.1.0, Latest until <date-of-new-version> ", // I'll put a date here when bumping version
    about = "A custom touch implementation in Rust"
)]
struct Cli {
    #[arg(required = true)]
    paths: Vec<String>,

    #[arg(short, long)]
    parents: bool,

    #[arg(long = "no-log", default_value_t = true, action = clap::ArgAction::SetFalse)]
    should_log: bool,
}

struct TouchArgs<'a> {
    paths: Vec<Cow<'a, Path>>,
    create_parents: bool,
    should_log: bool,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

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

    // Main creation loop for provided paths
    for path in &touch_args.paths {
        match create(path, touch_args.create_parents) {
            Ok(ReplResult::Aborted) => {
                logmgr::success_log("Aborted a replacement of a directory in a file.")?;
                return Ok(());
            }
            Ok(ReplResult::Completed) => {
                if touch_args.should_log {
                    logmgr::success_log(&format!(
                        "Replaced directory with file: {}",
                        path.display()
                    ))?;
                }
            }
            Ok(ReplResult::NotRequired) => {
                // Logging section
                if touch_args.should_log {
                    if touch_args.create_parents {
                        logmgr::success_log(&format!(
                            "File & parent folder created: {}",
                            path.display()
                        ))?;
                    } else {
                        logmgr::success_log(&format!("File Created: {}", path.display()))?;
                    }
                }
            }
            Err(error) => {
                eprintln!("{error}");
                logmgr::error_log(&format!("Unexpected Error : {error}"))?;
                continue;
            }
        }
    }

    Ok(())
}
