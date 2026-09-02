//! # Logging errors and crash events
//!
//! [`rtouch::log::logmgr::error_log`] appends an entry to the error/crash
//! log file located at:
//!
//! - **Linux/macOS**: `/var/log/R-touch/crashes/file_creations.log`
//! - **Windows**: `%LOCALAPPDATA%\R-touch\logs\crashes\file_creations.log`
//!
//! Use this logger whenever a file-operation fails and you want a persistent
//! audit trail of the failure.

use std::io;

fn main() -> io::Result<()> {
    let cfg = rtouch::LogConfig::from_env_defaults_for("R-touch");
    rtouch::log::logmgr::error_log(&cfg, &format_args!(
        "example: could not create /root/protected.txt — permission denied"
    ))?;
    println!("Error entry written to crash log.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    fn test_config() -> (rtouch::LogConfig, PathBuf) {
        let temp_dir = std::env::temp_dir().join(format!("rtouch_test_log_error_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        let cfg = rtouch::LogConfig::new(
            temp_dir.join("r-touch.log"),
            temp_dir.join("crashes").join("file_creations.log"),
            temp_dir.join("time_modifications").join("atime_modification.log"),
            temp_dir.join("time_modifications").join("mtime_modification.log"),
        );
        (cfg, temp_dir)
    }

    /// `error_log` must return `Ok(())` when the crash log directory is accessible.
    #[test]
    fn error_log_returns_ok() {
        let (cfg, temp_dir) = test_config();
        let result = rtouch::log::logmgr::error_log(&cfg, &format_args!("test: error_log_returns_ok"));
        assert!(result.is_ok(), "error_log failed: {:?}", result.err());
        assert!(cfg.error_log.exists());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn from_env_defaults_has_expected_paths() {
        let cfg = rtouch::LogConfig::from_env_defaults_for("R-touch");
        #[cfg(target_family = "unix")]
        assert_eq!(
            cfg.error_log,
            PathBuf::from("/var/log/R-touch/crashes/file_creations.log")
        );
        #[cfg(target_family = "windows")]
        assert!(cfg.error_log.ends_with(r"R-touch\logs\crashes\file_creations.log"));
    }
}
