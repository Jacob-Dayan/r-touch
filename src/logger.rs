use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

pub struct Logger;

impl Logger {
    pub fn log(file_path: &str, message: &str) -> std::io::Result<()> {
        let mut path = PathBuf::new();

        if file_path.starts_with("~/") {
            let home = match home::home_dir() {
                Some(p) => p,
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Could not determine the home directory",
                    ));
                }
            };
            path.push(home);
            path.push(&file_path[2..]);
        } else {
            path.push(file_path);
        }

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut file = OpenOptions::new().create(true).append(true).open(path)?;

        let timestamp = SystemTime::now();
        let log_line = format!("{:?}: {}\n", timestamp, message);

        file.write_all(log_line.as_bytes())?;
        file.flush()?;

        Ok(())
    }
}
