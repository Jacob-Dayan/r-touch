use fs_err::{self as fs, File};
use log::logmgr;
use std::borrow::Cow;
use std::env;
use std::process::ExitCode;

mod log {
    pub mod logger; //logging logic
    pub mod logmgr; // logging manager (wrapper)
}
mod replace_dir; //the file that is taking care on replacing folders with files (take a look)

struct TouchArgs<'a> {
    paths: Vec<Cow<'a, str>>,
    create_parents: bool,
    should_log: bool,
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let touch_args = match gen_path(&args) {
        Ok(args) => args,
        Err(error) => {
            println!("{error}");
            log::logmgr::error_log(&format!("Unexpected Error : {error}"));
            return ExitCode::FAILURE;
        }
    };

    for path in &touch_args.paths {
        // CHANGED: Replaced unwrap_or_else with if let Err and continue to properly skip failed iterations
        if let Err(error) = create(path, touch_args.create_parents) {
            println!("{error}");
            log::logmgr::error_log(&format!("Unexpected Error : {error}"));
            continue;
        }
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

    ExitCode::SUCCESS
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
            paths.push(Cow::Borrowed(arg.as_str())); //else return it without touching
        }
    }

    if paths.is_empty() {
        //if used "-p" argument but didn't pass it a file (only a parent dir)
        log::logmgr::error_log(&format!(
            "Error: passed in parent folder, expected parent dir + file."
        ));
        return Err("You need to pass in the path to the file.".to_string()); //return error (and then in main exit)
    }
    #[cfg(target_family = "windows")]
    {
        for path in &mut paths {
            if path.contains('/') {
                *path = Cow::Owned(path.replace('/', "\\"));
            }
        }
    }
    Ok(TouchArgs {
        paths,
        create_parents,
        should_log,
    })
}

fn create(path: &str, create_parents: bool) -> Result<(), String> {
    let path_buf = std::path::Path::new(path);

    if create_parents {
        //if the bool from the function above is true
        if let Some(parent) = path_buf.parent() {
            if !parent.as_os_str().is_empty() {
                // CHANGED: Removed internal error logging, returning error string directly via ?
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }
    }
    if path_buf.is_dir() {
        //if attempt to create a file in a name of an existing folder
        // CHANGED: Removed internal error logging, returning error string directly via ?
        replace_dir::replace(path).map_err(|e| format!("Faild to replace direcrory : {e}"))?;
    } else {
        // If the path is not an existing directory (the standard case for creating a new file),
        // the code falls into this block and creates the file on the disk
        // CHANGED: Removed internal error logging, returning error string directly via ?
        File::create(path).map_err(|e| e.to_string())?;
    }

    Ok(()) //if passed all the shi above return Ok status
}
