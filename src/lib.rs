use fs_err::{self as fs, File, OpenOptions};
#[rustfmt::skip]
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

pub fn create<P: AsRef<Path>>(path: P, create_parents: bool) -> io::Result<ReplResult> {
    let path_ref = path.as_ref();

    if create_parents {
        if let Some(parent) = path_ref.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
    }

    if path_ref.is_dir() {
        let res = replace_dir::replace(path_ref)?;
        return Ok(res);
    }

    if !path_ref.exists() {
        File::create(path_ref)?;
    } else {
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