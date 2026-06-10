use std::{
    env,
    fs::{self, File},
};
mod logger; //the logging logic (logger.rs)
mod logmgr; //log-manager (logmgr.rs)
mod replace_dir; //the file that is taking care on replacing folders with files (take a look)
fn main() {
    let args: Vec<String> = env::args().collect();
    let args = gen_path(&args).unwrap_or_else(|error| {
        println!("{error}");
        std::process::exit(1);
    });

    let path = args.0;
    let create_parents = args.1;
    let should_log = args.2;
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
        if should_log {
            logmgr::log_manager(&message)
        };
    }
}

fn gen_path(args: &[String]) -> Result<(&str, bool, bool), String> {
    if args.len() < 2 {
        return Err("You need to pass in the path to the file.".to_string());
    }

    let mut create_parents = false;
    let mut path = "";
    let mut should_log: bool = true;
    for arg in args.iter().skip(1) {
        //check if has got any arguments
        if arg == "-p" || arg == "--parents" {
            //if got the argument "-p" or "--parents":
            create_parents = true; //setting the bool to true
        //added --no-log
        //version 0.2.4
        // FIX
        } else if arg == "--no-log" {
            should_log = false;
        } else {
            path = arg.as_str(); //else return it without touching
            should_log = true;
        }
    }

    if path.is_empty() {
        //if he used "-p" argument but didn't pass it a file (only a parent dir)
        return Err("You need to pass in the path to the file.".to_string()); //return error (and then in main exit)
    }

    Ok((path, create_parents, should_log)) //if passed all the shi above return Ok status with the bool of create parents and the path
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
    // because we want the file to be created even when `create_parents` is false

    // Checking if the requested path is an existing directory on the disk
    if path_buf.is_dir() {
        //if passed an existinng dir
        replace_dir::replace(path).map_err(|e| format!("Failed to replace directory: {e}"))?; // maiking an own-costumed Error

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
