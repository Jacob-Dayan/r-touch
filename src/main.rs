use std::{
    env,
    fs::{self, File},
    io,
};
mod logger;
mod logmgr;
fn main() {
    let args: Vec<String> = env::args().collect();
    let args = gen_path(&args).unwrap_or_else(|error| {
        println!("{error}");
        std::process::exit(1);
    });

    let path = args.0;
    let create_parents = args.1;

    create(path, create_parents).unwrap_or_else(|error| {
        println!("{error}");
        std::process::exit(1);
    });
    //logging section
    println!("Success!");
    let message = format!("File Created: {path}");
    logmgr::log_manager(&message);
}
#[rustfmt::skip]
fn gen_path(args: &[String]) -> Result<(&str, bool), String> {
    if args.len() < 2 {
        return Err("You need to pass in the path to the file.".to_string());
    }

    let mut create_parents = false;
    let mut path = "";

    for arg in args.iter().skip(1) { //check if has got any arguments
        if arg == "-p" || arg == "--parents" { //if got the argument "-p" or "--parents":
            create_parents = true; //setting the bool to true
        } else {
            path = arg.as_str(); //else return it without touching
        }
    }

    if path.is_empty() {
        //if he used "-p" argument but didn't pass it a file (only a parent dir)
        return Err("You need to pass in the path to the file.".to_string()); //return error (and then in main exit)
    }

    Ok((path, create_parents)) //if passed all the shi above return Ok status with the bool of create parents and the path
}

fn create(path: &str, create_parents: bool) -> Result<(), String> {
    //conversing the str to a Path that rust can understand itself without us manually explaining to it what path is

    let path_buf = std::path::Path::new(path);

    if create_parents {
        //if the bool from the function above is true
        if let Some(parent) = path_buf.parent() {
            //I honestly don't know what that is a friend helped me lol
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directories: {e}"))?;
            }
        }
    }

    if path_buf.is_dir() {
        //if passed an existinng dir
        replace(path).map_err(|e| format!("Failed to replace directory: {e}"))?; // maiking an own-costumed Error
        File::create(path).map_err(|e| format!("Failed to create file: {e}"))?; //same
    }

    Ok(())
}
enum Action {
    Abort,
    Accept,
}
impl Action {
    fn new(path: &str) -> Self {
        // returns Action
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
            "n" => Action::Abort,
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
