//! # Logging access-time and modification-time events
//!
//! Two dedicated loggers cover the time-update path:
//!
//! | Function | Log file | When to use |
//! |----------|----------|-------------|
//! | [`rtouch_core::log::logmgr::access_time_success`] | `access_time/access-time_success.log` | Timestamp update succeeded |
//! | [`rtouch_core::log::logmgr::access_time_failure`] | `crashes/access-time_failure.log`     | Date parsing or update failed |
//!
//! Both loggers share the same `<data_local_dir>/R-touch/logs/` root but write
//! to separate files so that successes and failures can be audited independently.

use std::io;

fn main() -> io::Result<()> {
    rtouch_core::log::logmgr::access_time_success(
        &format_args!("example: atime updated to yesterday for report.pdf"),
    )?;

    rtouch_core::log::logmgr::access_time_failure(
        &format_args!("example: failed to parse date expression 'blarg'"),
    )?;

    println!("Access-time success and failure entries written.");
    Ok(())
}

#[cfg(test)]
mod tests {
    /// `access_time_success` must write without error.
    #[test]
    fn access_time_success_returns_ok() {
        let result = rtouch_core::log::logmgr::access_time_success(
            &format_args!("test: access_time_success_returns_ok"),
        );
        assert!(result.is_ok(), "access_time_success failed: {:?}", result.err());
    }

    /// `access_time_failure` must write without error.
    #[test]
    fn access_time_failure_returns_ok() {
        let result = rtouch_core::log::logmgr::access_time_failure(
            &format_args!("test: access_time_failure_returns_ok"),
        );
        assert!(result.is_ok(), "access_time_failure failed: {:?}", result.err());
    }
}
