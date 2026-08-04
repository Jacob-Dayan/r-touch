use fs_err::{self as fs, File, OpenOptions};
#[rustfmt::skip] // So rustfmt won't make this 1 line (it becomes hard to read)
use std::{
    fs::FileTimes,
    io,
    path::Path,
    time::SystemTime,
};

pub mod log {
    pub mod logger;
    pub mod logmgr;
}
pub mod replace_dir;

pub use replace_dir::ReplResult;

// Core file creation and timestamp management logic
pub fn create<P: AsRef<Path>>(path: P, create_parents: bool) -> io::Result<ReplResult> {
    let path_ref = path.as_ref();

    // Ensure parent directories exist when explicitly requested (-p / --parents)
    if create_parents {
        if let Some(parent) = path_ref.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
    }

    // Handle directory replacement if target path is an existing directory
    if path_ref.is_dir() {
        let res = replace_dir::replace(path_ref)?;
        return Ok(res);
    }

    if !path_ref.exists() {
        // Create a new empty file if it doesn't exist yet
        File::create(path_ref)?;
    } else {
        // File exists: update timestamps without truncating content, touch-like behavior
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path_ref)?;

        let now = SystemTime::now();

        let times = FileTimes::new().set_accessed(now).set_modified(now);

        file.set_times(times)?;
    }

    Ok(ReplResult::NotRequired)
}