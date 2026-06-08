use std::fs::{self, OpenOptions};
use std::io::{Error, ErrorKind, Result, Write};
use std::path::PathBuf;
use std::time::SystemTime;

pub struct Logger;

impl Logger {
    pub fn log(file_path: &str, message: &str) -> Result<()> {
        let path = if let Some(stripped) = file_path.strip_prefix("~/") {
            let mut home = home::home_dir().ok_or_else(|| {
                Error::new(
                    ErrorKind::NotFound,
                    "Could not determine the home directory",
                )
            })?;
            home.push(stripped);
            home
        } else {
            PathBuf::from(file_path)
        };

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        let log_line = format!("{:?}: {}\n", SystemTime::now(), message);

        file.write_all(log_line.as_bytes())?;
        file.flush()
    }
}
