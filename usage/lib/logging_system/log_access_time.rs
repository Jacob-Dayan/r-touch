//! # Logging access-time and modification-time events
//!
//! Two dedicated loggers cover the time-update path:
//!
//! | Function | Log file | When to use |
//! |----------|----------|-------------|
//! | [`rtouch::log::logmgr::atime_modification_success`] | `time_modifications/atime_modification.log` | Access-time update succeeded |
//! | [`rtouch::log::logmgr::time_modification_failure`] | `crashes/*`     | Date parsing or update failed |
//!
//! Both loggers share the same `<data_local_dir>/R-touch/logs/` root but write
//! to separate files so that successes and failures can be audited independently.

use std::io;

fn main() -> io::Result<()> {
    let cfg = rtouch::LogConfig::from_env_defaults();

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
    /// `atime_modification_success` must write without error.
    #[test]
    fn access_time_success_returns_ok() {
        let cfg = rtouch::LogConfig::from_env_defaults();
        let result = rtouch::log::logmgr::atime_modification_success(&cfg, &format_args!(
            "test: access_time_success_returns_ok"
        ));
        assert!(
            result.is_ok(),
            "access_time_success failed: {:?}",
            result.err()
        );
    }

    /// `time_modification_failure` must write without error.
    #[test]
    fn access_time_failure_returns_ok() {
        let cfg = rtouch::LogConfig::from_env_defaults();
        let result = rtouch::log::logmgr::time_modification_failure(&cfg, &format_args!(
            "test: access_time_failure_returns_ok"
        ));
        assert!(
            result.is_ok(),
            "access_time_failure failed: {:?}",
            result.err()
        );
    }
}
