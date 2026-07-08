use fs_err::{self as fs, File};
use log::logmgr;
use std::borrow::Cow;
use std::env;
use std::process::ExitCode;

mod log {
    pub mod logger;
    pub mod logmgr;
}
mod replace_dir;

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
            return ExitCode::from(1);
        }
    };

    for path in &touch_args.paths {
        create(path, touch_args.create_parents).unwrap_or_else(|error| {
            println!("{error}");
            log::logmgr::error_log(&format!("Unexpected Error : {error}"));
            return;
        });

        if !touch_args.create_parents && touch_args.should_log {
            log::logmgr::success_log(&format!("File Created: {path}"));
        } else if touch_args.should_log {
            log::logmgr::success_log(&format!("File & parent folder created: {path}"))
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
        if arg == "-p" || arg == "--parents" {
            create_parents = true;
        } else if arg == "--no-log" {
            should_log = false;
        } else {
            paths.push(Cow::Borrowed(arg.as_str()));
        }
    }

    if paths.is_empty() {
        log::logmgr::error_log(&format!(
            "Error: passed in parent folder, expected parent dir + file."
        ));
        return Err("You need to pass in the path to the file.".to_string());
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
        if let Some(parent) = path_buf.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| {
                    let err_msg = e.to_string();
                    log::logmgr::error_log(&err_msg);
                    err_msg
                })?;
            }
        }
    }
    if path_buf.is_dir() {
        if let Err(e) = replace_dir::replace(path) {
            let err_msg = format!("Faild to replace direcrory : {e}");
            log::logmgr::error_log(&err_msg);
            return Err(err_msg);
        }
    } else {
        File::create(path).map_err(|e| {
            let err_msg = e.to_string();
            log::logmgr::error_log(&err_msg);
            err_msg
        })?;
    }

    Ok(())
}
