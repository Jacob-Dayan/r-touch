use crate::log::logger;
use std::{io, path::PathBuf};
#[cfg(target_family = "windows")]
const OS_ROOT: Option<&str> = Some("C:\\Users\\Public");
#[cfg(target_family = "unix")]
const OS_ROOT: Option<&str> = Some("/var/log");
#[cfg(not(any(target_family = "windows", target_family = "unix")))]
// used to log from `.`,
//  but in a case when both - dirs-next can't find data directory and we are working on an unfamiliar OS
// we should just give up on logging
const OS_ROOT: Option<&str> = None;

// Logging of successful actions
pub fn success_log(message: &str) -> io::Result<()> {
    let path = match dirs_next::data_local_dir() {
        Some(dir) => dir.join("R-touch").join("logs").join("r-touch.log"),
        None => match OS_ROOT {
            Some(root) => PathBuf::from(root),
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Cannot log actions.",
                ));
            }
        },
    };

    if let Err(e) = logger::Logger::log(&path, message) {
        let e = format!("Cannot log actions: {e}");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
    }
    Ok(())
}

// Logging of crash and error events
pub fn error_log(message: &str) -> io::Result<()> {
    match dirs_next::data_local_dir() {
        Some(dir) => {
            let path = dir
                .join("R-touch")
                .join("logs")
                .join("crashes")
                .join("r-touch_err.log");
            logger::Logger::log(&path, message)?;
        }
        None => {
            match OS_ROOT {
                Some(root) => PathBuf::from(root),
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Cannot log actions.",
                    ));
                }
            };
        }
    };

    Ok(())
}
