use std::{
    env,
    fs::{self, File},
    io,
};
mod logger; //the logging logic (logger.rs).
mod logmgr; //log-manager (logmgr.rs)
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
    if !create_parents {
        //if created a file in a regular path (in an existing dir)
        let message = format!("File Created: {path}");
        logmgr::log_manager(&message);
    } else {
        //if DID create the folder
        let message = format!("File & parent folder created: {path}");
        logmgr::log_manager(&message);
    }
}

fn gen_path(args: &[String]) -> Result<(&str, bool), String> {
    if args.len() < 2 {
        return Err("You need to pass in the path to the file.".to_string());
    }

    let mut create_parents = false;
    let mut path = "";

    for arg in args.iter().skip(1) {
        //check if has got any arguments
        if arg == "-p" || arg == "--parents" {
            //if got the argument "-p" or "--parents":
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
    // FIX: The parent directory creation block ends here. We do not create the file inside this block
    // because we want the file to be created even when `create_parents` is false.

    // Checking if the requested path is an existing directory on the disk
    if path_buf.is_dir() {
        //if passed an existinng dir
        replace(path).map_err(|e| format!("Failed to replace directory: {e}"))?; // maiking an own-costumed Error

    // FIX: Removed the duplicate `File::create` that was here in your original code,
    // since the `replace` function already handles creating the file if the user confirms with 'y'.
    } else {
        // FIX: This block solves the main issue.
        // If the path is not an existing directory (the standard case for creating a new file),
        // the code falls into this block and creates the file safely on the disk.
        File::create(path).map_err(|e| format!("Failed to create file: {e}"))?;
    }

    Ok(()) //if passed all the shi above return Ok status
}
enum Action {
    Abort,
    Accept,
}
impl Action {
    fn new(path: &str) -> Self {
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
fn replace(path: &str) -> io::Result<()> {
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
