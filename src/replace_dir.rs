use super::logmgr;
use fs_err::{self as fs, File};
use std::io::{self, ErrorKind};
pub enum Action {
    Abort,
    Accept,
}
impl Action {
    #[rustfmt::skip]
    pub fn new(path: &str) -> Self {
        println!("'{path}' is a directory. Do you want to delete directory and replace it with the file? (y/n)");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Faild reading the line.");
        match input.trim().to_ascii_lowercase().as_str() {
            //matching as lowercased str
            "y" | "yes" => Action::Accept,
            _ => Action::Abort, //anything else than `y`/`yes` returns abort
        }
    }
}
#[rustfmt::skip]
pub fn replace(path: &str) -> io::Result<()> {
    let action = Action::new(path);

    match action {
        Action::Accept => {
            fs::remove_dir_all(path)?;
            match File::create(path) {
                Ok(_) => {
                    logmgr::success_log(&format!("Replaced directory with file: {path}"));
                    Ok(())
                }
                Err(e) => {
                    match e.kind() {
                        ErrorKind::IsADirectory => {
                            eprintln!("Error:{e}\nconsider removing the '/' char at the end of the path.");
                        }
                        _ => eprintln!("{e}"),
                    }
                    Err(e)
                }
            }
        }
        Action::Abort => {
            println!("Abort");
            logmgr::success_log("Aborted a replacement of a directory in a file.");
            std::process::exit(0) //quit with success code (0)
        }
    }
}
