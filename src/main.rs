use log::logmgr;
use std::{
    env,
    fs::{self, File},
};
mod log {
    pub mod logger; //logging locic
    pub mod logmgr; // logging manager (wrapper)
}
mod replace_dir; //the file that is taking care on replacing folders with files (take a look)

struct TouchArgs<'a> {
    paths: Vec<&'a str>,
    create_parents: bool,
    should_log: bool,
}

fn main() -> Result<(), i32> {
    let args: Vec<String> = env::args().collect();
    let touch_args = match gen_path(&args) {
        Ok(args) => args,
        Err(error) => {
            println!("{error}");
            log::logmgr::error_log(&format!("Unexpected Error : {error}"));
            return Err(1);
        }
    };

    for path in touch_args.paths {
        create(path, touch_args.create_parents).unwrap_or_else(|error| {
            println!("{error}");
            log::logmgr::error_log(&format!("Unexpected Error : {error}"));
            return;
        });
        //logging section
        // println!("Success!");
        if !touch_args.create_parents && touch_args.should_log {
            //if created a file in a regular path (in an existing dir) and didn't run with --no-log
            log::logmgr::success_log(&format!("File Created: {path}"));
        } else {
            //if DID create the folder
            if touch_args.should_log {
                log::logmgr::success_log(&format!("File & parent folder created: {path}"))
            };
        }
    }

    Ok(())
}

fn gen_path(args: &[String]) -> Result<TouchArgs<'_>, String> {
    if args.len() < 2 {
        return Err("You need to pass in the path to the file.".to_string());
    }

    let mut create_parents = false;
    let mut paths = Vec::new();
    let mut should_log: bool = true;
    for arg in args.iter().skip(1) {
        //check if has got any arguments
        if arg == "-p" || arg == "--parents" {
            create_parents = true;
        } else if arg == "--no-log" {
            should_log = false;
        } else {
            paths.push(arg.as_str()); //else return it without touching
        }
    }

    if paths.is_empty() {
        //if he used "-p" argument but didn't pass it a file (only a parent dir)
        log::logmgr::error_log(&format!(
            "Error: passed in parent folder, expected parent dir + file."
        ));
        return Err("You need to pass in the path to the file.".to_string()); //return error (and then in main exit)
    }

    Ok(TouchArgs {
        paths,
        create_parents,
        should_log,
    }) //if passed all the shi above return Ok status with TouchArgs
}

fn create(path: &str, create_parents: bool) -> Result<(), String> {
    let path_buf = std::path::Path::new(path);

    if create_parents {
        //if the bool from the function above is true
        if let Some(parent) = path_buf.parent() {
            if !parent.as_os_str().is_empty() {
                if let Err(e) = fs::create_dir_all(parent) {
                    let err_msg = format!("Faild to create paret directories. Error: {e}");
                    log::logmgr::error_log(&err_msg);
                    return Err(err_msg); //here returning the error formatted to print out in line 22
                }
            }
        }
    }
    if path_buf.is_dir() {
        //if attempt to create a file in a name of an existing folder
        if let Err(e) = replace_dir::replace(path) {
            let err_msg = format!("Faild to replace direcrory : {e}");
            log::logmgr::error_log(&err_msg);
            return Err(err_msg);
        }
    } else {
        // If the path is not an existing directory (the standard case for creating a new file),
        // the code falls into this block and creates the file on the disk
        File::create(path).map_err(|e| format!("Failed to create file: {e}"))?;
    }

    Ok(()) //if passed all the shi above return Ok status
}
