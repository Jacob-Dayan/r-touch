//! # Logging successful operations
//!
//! [`rtouch::log::logmgr::success_log`] appends an entry to the general
//! success log file located at:
//!
//! - **Linux/macOS**: `~/.local/share/R-touch/logs/r-touch.log`
//! - **Windows**: `%LOCALAPPDATA%\R-touch\logs\r-touch.log`
//!
//! Each entry is timestamped automatically by [`rtouch::log::log_core::LogCore`].
//! The logger is initialised once (via [`std::sync::LazyLock`]) and reused for
//! all subsequent calls, so there is no repeated path resolution overhead.

use std::io;

fn main() -> io::Result<()> {
    let cfg = rtouch::LogConfig::from_env_defaults();
    rtouch::log::logmgr::success_log(&cfg, &format_args!("example: file created successfully"))?;
    println!("Success entry written to log.");
    Ok(())
}

#[cfg(test)]
mod tests {
    /// `success_log` must return `Ok(())` when the log directory is accessible.
    #[test]
    fn success_log_returns_ok() {
        let cfg = rtouch::LogConfig::from_env_defaults();
        let result = rtouch::log::logmgr::success_log(&cfg, &format_args!("test: success_log_returns_ok"));
        assert!(result.is_ok(), "success_log failed: {:?}", result.err());
    }
}
