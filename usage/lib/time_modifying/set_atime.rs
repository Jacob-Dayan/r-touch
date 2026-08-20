//! # Update only the access time (`atime`)
//!
//! [`rtouch_core::set_access_time`] is the focused API for changing only
//! the access timestamp of an **existing** file without touching its
//! modification time.

use std::io;

fn main() -> io::Result<()> {
    // Ensure the file exists first.
    let path = std::env::temp_dir().join("rtouch_usage_atime.txt");
    std::fs::write(&path, b"")?;

    let one_hour_ago = rtouch_core::datetime::parse_time_expression("1 hour ago")
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
    rtouch_core::set_access_time(&path, one_hour_ago)?;
    println!("Access time set to one hour ago: {}", path.display());

    std::fs::remove_file(&path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    /// After calling `set_access_time`, the file's atime must match the
    /// requested value (within a 2-second tolerance for OS rounding).
    #[test]
    fn atime_is_updated() {
        let path = std::env::temp_dir().join("rtouch_usage_set_atime.txt");
        std::fs::write(&path, b"").unwrap();

        let target = rtouch_core::datetime::parse_time_expression("2 hours ago").unwrap();
        rtouch_core::set_access_time(&path, target).unwrap();

        let got = std::fs::metadata(&path).unwrap().accessed().unwrap();
        let diff = if got > target {
            got.duration_since(target)
        } else {
            target.duration_since(got)
        };
        assert!(diff.unwrap() < Duration::from_secs(2), "atime mismatch");

        std::fs::remove_file(&path).unwrap();
    }
}
