//! # Logging access-time and modification-time events
//!
//! Two dedicated loggers cover the time-update path:
//!
//! | Function | Log file | When to use |
//! |----------|----------|-------------|
//! | [`rtouch::log::logmgr::atime_modification_success`] | `time_modifications/atime_modification.log` | Access-time update succeeded |
//! | [`rtouch::log::logmgr::time_modification_failure`] | `crashes/*`     | Date parsing or update failed |
//!
//! Both loggers share the same `/var/log/R-touch` (or `%LOCALAPPDATA%\R-touch\logs\` on Windows) root but write
//! to separate files so that successes and failures can be audited independently.

use std::io;

fn main() -> io::Result<()> {
    let cfg = rtouch::LogConfig::from_env_defaults_for("R-touch");

    rtouch::log::logmgr::atime_modification_success(&cfg, &format_args!(
        "example: atime updated to yesterday for report.pdf"
    ))?;

    rtouch::log::logmgr::time_modification_failure(&cfg, &format_args!(
        "example: failed to parse date expression 'blarg'"
    ))?;

    println!("Access-time success and failure entries written.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    fn test_config() -> (rtouch::LogConfig, PathBuf) {
        let temp_dir = std::env::temp_dir().join(format!("rtouch_test_log_access_time_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        let cfg = rtouch::LogConfig::new(
            temp_dir.join("r-touch.log"),
            temp_dir.join("crashes").join("file_creations.log"),
            temp_dir.join("time_modifications").join("atime_modification.log"),
            temp_dir.join("time_modifications").join("mtime_modification.log"),
        );
        (cfg, temp_dir)
    }

    /// `atime_modification_success` must write without error.
    #[test]
    fn access_time_success_returns_ok() {
        let (cfg, temp_dir) = test_config();
        let result = rtouch::log::logmgr::atime_modification_success(&cfg, &format_args!(
            "test: access_time_success_returns_ok"
        ));
        assert!(
            result.is_ok(),
            "access_time_success failed: {:?}",
            result.err()
        );
        assert!(cfg.atime_log.exists());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// `time_modification_failure` must write without error.
    #[test]
    fn access_time_failure_returns_ok() {
        let (cfg, temp_dir) = test_config();
        let result = rtouch::log::logmgr::time_modification_failure(&cfg, &format_args!(
            "test: access_time_failure_returns_ok"
        ));
        assert!(
            result.is_ok(),
            "access_time_failure failed: {:?}",
            result.err()
        );
        assert!(cfg.error_log.exists());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn from_env_defaults_has_expected_paths() {
        let cfg = rtouch::LogConfig::from_env_defaults_for("R-touch");
        #[cfg(target_family = "unix")]
        {
            assert_eq!(
                cfg.atime_log,
                PathBuf::from("/var/log/R-touch/time_modifications/atime_modification.log")
            );
            assert_eq!(
                cfg.error_log,
                PathBuf::from("/var/log/R-touch/crashes/file_creations.log")
            );
        }
        #[cfg(target_family = "windows")]
        {
            assert!(cfg.atime_log.ends_with(r"R-touch\logs\time_modifications\atime_modification.log"));
            assert!(cfg.error_log.ends_with(r"R-touch\logs\crashes\file_creations.log"));
        }
    }
}
