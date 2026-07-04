use fs_err::{self as fs, OpenOptions};
use std::io::{Result, Write};
use std::path::PathBuf;
use std::time::SystemTime;

pub struct Logger;

impl Logger {
    pub fn log(file_path: &str, message: &str) -> Result<()> {
        let path = PathBuf::from(file_path);

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;

        file.write_all(format!("{:?}: {}\n", SystemTime::now(), message).as_bytes())?;
        file.flush()
    }
}
