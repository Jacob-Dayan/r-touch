//! # Logging successful operations
//!
//! [`rtouch::log::logmgr::success_log`] appends an entry to the general
//! success log file located at:
//!
//! - **Linux/macOS**: `/var/log/R-touch/r-touch.log`
//! - **Windows**: `%LOCALAPPDATA%\R-touch\logs\r-touch.log`
//!
//! Each entry is timestamped automatically by [`rtouch::log::log_core::LogCore`].
//! The logger is initialised once (via [`std::sync::LazyLock`]) and reused for
//! all subsequent calls, so there is no repeated path resolution overhead.

use std::io;

fn main() -> io::Result<()> {
    let cfg = rtouch::LogConfig::from_env_defaults_for("R-touch");
    rtouch::log::logmgr::success_log(&cfg, &format_args!("example: file created successfully"))?;
    println!("Success entry written to log.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    fn test_config() -> (rtouch::LogConfig, PathBuf) {
        let temp_dir = std::env::temp_dir().join(format!("rtouch_test_log_success_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        let cfg = rtouch::LogConfig::new(
            temp_dir.join("r-touch.log"),
            temp_dir.join("crashes").join("file_creations.log"),
            temp_dir.join("time_modifications").join("atime_modification.log"),
            temp_dir.join("time_modifications").join("mtime_modification.log"),
        );
        (cfg, temp_dir)
    }

    /// `success_log` must return `Ok(())` when the log directory is accessible.
    #[test]
    fn success_log_returns_ok() {
        let (cfg, temp_dir) = test_config();
        let result = rtouch::log::logmgr::success_log(&cfg, &format_args!("test: success_log_returns_ok"));
        assert!(result.is_ok(), "success_log failed: {:?}", result.err());
        assert!(cfg.success_log.exists());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn from_env_defaults_has_expected_paths() {
        let cfg = rtouch::LogConfig::from_env_defaults_for("R-touch");
        #[cfg(target_family = "unix")]
        assert_eq!(cfg.success_log, PathBuf::from("/var/log/R-touch/r-touch.log"));
        #[cfg(target_family = "windows")]
        assert!(cfg.success_log.ends_with(r"R-touch\logs\r-touch.log"));
    }
}
