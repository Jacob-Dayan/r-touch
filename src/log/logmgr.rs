use crate::log::logger;
use crate::new_io_error;
use std::{io, path::PathBuf};

// not really OS_ROOT but the root directory for log files
#[cfg(target_family = "windows")]
const OS_ROOT: Option<&str> = Some("C:\\Users\\Public\\AppData\\Local");
#[cfg(target_family = "unix")]
const OS_ROOT: Option<&str> = Some("/var/log");
#[cfg(not(any(target_family = "windows", target_family = "unix")))]
const OS_ROOT: Option<&str> = None;

macro_rules! resolve_log_path {
    ($($path_segments:expr),+ $(,)?) => {{
        let base_dir = match dirs_next::data_local_dir() {
            Some(dir) => dir,
            None => match OS_ROOT {
                Some(root) => PathBuf::from(root),
                None => {
                    return Err(new_io_error!("Cannot log actions."));
                }
            },
        };

        let mut path = base_dir;
        $(
            path = path.join($path_segments);
        )+
        path
    }};
}
// Logging of successful actions
pub fn success_log(message: &str) -> io::Result<()> {
    let path = resolve_log_path!["R-touch", "logs", "r-touch.log"];

    if let Err(e) = logger::Logger::log(&path, message) {
        let e = format!("Cannot log actions: {e}");
        return Err(new_io_error!(e));
    }
    Ok(())
}

// Logging of crash and error events
pub fn error_log(message: &str) -> io::Result<()> {
    let path = resolve_log_path!["R-touch", "logs", "crashes", "file_creations.log"];

    if let Err(e) = logger::Logger::log(&path, message) {
        let e = format!("Cannot log error: {e}");
        return Err(new_io_error!(e));
    }

    Ok(())
}

// Logging of successful access time updates
pub fn access_time_success(message: &str) -> io::Result<()> {
    let path = resolve_log_path!["R-touch", "logs", "access_time", "access-time_success.log"];

    if let Err(e) = logger::Logger::log(&path, message) {
        let e = format!("Cannot log error: {e}");
        return Err(new_io_error!(e));
    }

    Ok(())
}

// Logging of failed access time updates or parsing
pub fn access_time_failure(message: &str) -> io::Result<()> {
    let path = resolve_log_path!["R-touch", "logs", "crashes", "access-time_failure.log"];

    if let Err(e) = logger::Logger::log(&path, message) {
        let e = format!("Cannot log error: {e}");
        return Err(new_io_error!(e));
    }

    Ok(())
}
