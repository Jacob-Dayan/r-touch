use fs_err::{self as fs, File};
use std::path::Path;

pub mod log {
    pub mod logger;
    pub mod logmgr;
}
pub mod replace_dir;

pub use replace_dir::ReplResult;

// Core file creation logic
pub fn create<P: AsRef<Path>>(path: P, create_parents: bool) -> Result<ReplResult, String> {
    let path_ref = path.as_ref();

    if create_parents {
        if let Some(parent) = path_ref.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }
    }

    if path_ref.is_dir() {
        let res = replace_dir::replace(path_ref)
            .map_err(|e| format!("Failed to replace directory: {e}"))?;
        return Ok(res);
    } else {
        File::create(path_ref).map_err(|e| e.to_string())?;
    }

    Ok(ReplResult::NotRequired)
}
