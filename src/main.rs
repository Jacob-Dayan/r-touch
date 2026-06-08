//Recreation of touch command
//It is called "R-touch" (ro just "rtouch")
//It can still be really omproved :)

use std::{
    env,
    fs::{self, File},
    io,
};
fn main() {
    let args: Vec<String> = env::args().collect();
    create(&args).unwrap_or_else(|error| {
        println!("{error}");
        std::process::exit(1);
    });
    println!("Success!");
}

fn create(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("You need to pass in the path to the file.".to_string());
    }
    let path = &args[1];

    let path_buf = std::path::Path::new(path);
    if path_buf.is_dir() {
        replace(path).map_err(|e| format!("Failed to replace directory: {e}"))?;
    } else {
        File::create(path).map_err(|e| format!("Failed to create file: {e}"))?;
    }

    Ok(())
}
enum Action {
    Abort,
    Accept,
}
impl Action {
    fn new(path: &str) -> Self {
        println!(
            "'{path}' is a directory. Do you want the program to delete the directory and replace it with the file? (y/n)"
        );
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Faild reading the line.");
        match input.trim().to_ascii_lowercase().as_str() {
            "y" => {
                return Action::Accept;
            }
            "abort" => Action::Abort,
            _ => Action::Abort,
        }
    }
}
fn replace(path: &str) -> io::Result<()> {
    let action = Action::new(&path);
    match action {
        Action::Accept => {
            fs::remove_dir_all(path)?;
            File::create(path)?;
            Ok(())
        }
        Action::Abort => {
            println!("Abort");
            std::process::exit(0)
        }
    }
}
