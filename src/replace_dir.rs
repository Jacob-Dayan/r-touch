use super::logmgr;
use std::fs::{self, File};
use std::io;
pub enum Action {
    Abort,
    Accept,
}
impl Action {
    pub fn new(path: &str) -> Self {
        // returns Action
        println!(
            "'{path}' is a directory. Do you want the program to delete the directory and replace it with the file? (y/n)" //asking the user to accept the replacing action
        );
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Faild reading the line.");
        //removed "no" and replaced it with "y" | "yes" (cause the _ already takes the "no" case)
        match input.trim().to_ascii_lowercase().as_str() {
            //matching in lowercase as str
            "y" | "yes" => {
                return Action::Accept;
            }

            _ => Action::Abort, //if said anything else that yes/y then return abort and then in replace function quit
        }
    }
}
pub fn replace(path: &str) -> io::Result<()> {
    let action = Action::new(&path);
    match action {
        Action::Accept => {
            fs::remove_dir_all(path)?;
            File::create(path)?;
            let message = format!("Replaced directory with file: {path}");
            logmgr::log_manager(&message);
            Ok(())
        }
        Action::Abort => {
            println!("Abort");
            logmgr::log_manager("Aborted a replacement of a directory in a file. ");
            std::process::exit(0) //quit with success code (0)
        }
    }
}
