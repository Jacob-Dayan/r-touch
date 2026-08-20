//! # Logging errors and crash events
//!
//! [`rtouch::log::logmgr::error_log`] appends an entry to the error/crash
//! log file located at:
//!
//! - **Linux/macOS**: `~/.local/share/R-touch/logs/crashes/file_creations.log`
//! - **Windows**: `%LOCALAPPDATA%\R-touch\logs\crashes\file_creations.log`
//!
//! Use this logger whenever a file-operation fails and you want a persistent
//! audit trail of the failure.

use std::io;

fn main() -> io::Result<()> {
    let cfg = rtouch::LogConfig::from_env_defaults();
    rtouch::log::logmgr::error_log(&cfg, &format_args!(
        "example: could not create /root/protected.txt — permission denied"
    ))?;
    println!("Error entry written to crash log.");
    Ok(())
}

#[cfg(test)]
mod tests {
    /// `error_log` must return `Ok(())` when the crash log directory is accessible.
    #[test]
    fn error_log_returns_ok() {
        let cfg = rtouch::LogConfig::from_env_defaults();
        let result = rtouch::log::logmgr::error_log(&cfg, &format_args!("test: error_log_returns_ok"));
        assert!(result.is_ok(), "error_log failed: {:?}", result.err());
    }
}
