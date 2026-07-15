use crate::log::logmgr;

use fs_err::{self as fs, File};
use std::io::{self, ErrorKind};
use std::path::Path;

pub enum Action {
    Abort,
    Accept,
}

pub enum ReplResult {
    Completed,
    Aborted,
    NotRequired,
}

impl Action {
    // Prompt user input in terminal
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        println!(
            "'{}' is a directory. Do you want to delete directory and replace it with the file? (y/n)",
            path.as_ref().display()
        );
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return Action::Abort;
        }
        match input.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => Action::Accept,
            _ => Action::Abort,
        }
    }
}

pub fn replace<P: AsRef<Path>>(path: P) -> io::Result<ReplResult> {
    let path_ref = path.as_ref();
    let action = Action::new(path_ref);

    match action {
        Action::Accept => {
            fs::remove_dir_all(path_ref)?;
            match File::create(path_ref) {
                Ok(_) => {
                    logmgr::success_log(&format!(
                        "Replaced directory with file: {}",
                        path_ref.display()
                    ));
                    Ok(ReplResult::Completed)
                }
                Err(e) => {
                    match e.kind() {
                        ErrorKind::IsADirectory => {
                            eprintln!(
                                "Error:{e}\nconsider removing the '/' char at the end of the path."
                            );
                        }
                        _ => eprint!("{e}"),
                    }
                    Err(e)
                }
            }
        }
        Action::Abort => {
            println!("Abort");
            Ok(ReplResult::Aborted)
        }
    }
}
