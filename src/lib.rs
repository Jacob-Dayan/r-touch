use fs_err::{self as fs, File, OpenOptions};
use std::path::Path;
use std::time::SystemTime;

pub mod log {
    pub mod logger;
    pub mod logmgr;
}
pub mod replace_dir;

pub use replace_dir::ReplResult;

// Core file creation and timestamp management logic
pub fn create<P: AsRef<Path>>(path: P, create_parents: bool) -> Result<ReplResult, String> {
    let path_ref = path.as_ref();

    // Ensure parent directories exist when explicitly requested (-p / --parents)
    if create_parents {
        if let Some(parent) = path_ref.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }
    }

    // Handle directory replacement if target path is an existing directory
    if path_ref.is_dir() {
        let res = replace_dir::replace(path_ref)
            .map_err(|e| format!("Failed to replace directory: {e}"))?;
        return Ok(res);
    }

    if !path_ref.exists() {
        // Create a new empty file if it doesn't exist yet
        File::create(path_ref)
            .map_err(|e| format!("Unexpected Error: {e}.\nConsider running with `-p`."))?;
    } else {
        // File exists: update timestamps without truncating content, touch-like behavior
        let file = OpenOptions::new()
            .open(path_ref)
            .map_err(|e| format!("Failed to open existing file: {e}"))?;

        let now = SystemTime::now();

        file.file()
            .set_times(
                std::fs::FileTimes::new()
                    .set_accessed(now)
                    .set_modified(now),
            )
            .map_err(|e| format!("Failed to update timestamps: {e}"))?;
    }

    Ok(ReplResult::NotRequired)
}
