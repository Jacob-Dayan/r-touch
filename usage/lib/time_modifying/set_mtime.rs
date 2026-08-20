//! # Update only the modification time (`mtime`)
//!
//! [`rtouch_core::set_modification_time`] changes only the modification
//! timestamp of an **existing** file, leaving the access time untouched.

use std::io;

fn main() -> io::Result<()> {
    let path = std::env::temp_dir().join("rtouch_usage_mtime.txt");
    std::fs::write(&path, b"")?;

    let three_days_ago = rtouch_core::datetime::parse_time_expression("3 days ago")
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
    rtouch_core::set_modification_time(&path, three_days_ago)?;
    println!(
        "Modification time set to three days ago: {}",
        path.display()
    );

    std::fs::remove_file(&path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    /// After calling `set_modification_time`, the file's mtime must match
    /// the requested value (within a 2-second tolerance).
    #[test]
    fn mtime_is_updated() {
        let path = std::env::temp_dir().join("rtouch_usage_set_mtime.txt");
        std::fs::write(&path, b"").unwrap();

        let target = rtouch_core::datetime::parse_time_expression("3 days ago").unwrap();
        rtouch_core::set_modification_time(&path, target).unwrap();

        let got = std::fs::metadata(&path).unwrap().modified().unwrap();
        let diff = if got > target {
            got.duration_since(target)
        } else {
            target.duration_since(got)
        };
        assert!(diff.unwrap() < Duration::from_secs(2), "mtime mismatch");

        std::fs::remove_file(&path).unwrap();
    }
}
