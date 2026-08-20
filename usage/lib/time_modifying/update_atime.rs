//! # Update the access time (`atime`) to a relative date expression
//!
//! Demonstrates combining [`rtouch_core::datetime::parse_time_expression`]
//! with [`rtouch_core::touch`] to set the access time of a file to a
//! human-readable relative date such as `"yesterday"`.

use std::{fs, io, path::Path};

use std::time::UNIX_EPOCH;

/// Touches `path`, setting only its access time to `atime` (a date expression).
fn set_access_time_of_file(path: &str, atime: &str) -> Result<rtouch_core::ReplResult, io::Error> {
    let time = rtouch_core::datetime::parse_time_expression(atime)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
    rtouch_core::touch(
        path,
        false, // don't create parent directories
        Some(time),
        true,  // update access time
        false, // don't update modification time
    )
}

fn main() -> io::Result<()> {
    println!("Setting the access time of foo.txt to yesterday!");
    if !Path::new("foo.txt").exists() {
        eprintln!("Oh, no! `foo.txt` not found!");
        println!("Creating foo.txt with access time of yesterday...");
    }
    set_access_time_of_file("foo.txt", "yesterday")?;

    let file_access_time = fs::metadata("foo.txt")?.accessed()?;
    let expected_time = rtouch_core::datetime::parse_time_expression("yesterday").unwrap();

    let file_secs = file_access_time
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let expected_secs = expected_time.duration_since(UNIX_EPOCH).unwrap().as_secs();

    assert_eq!(file_secs, expected_secs);
    println!("Access time updated successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::set_access_time_of_file;
    use std::time::UNIX_EPOCH;

    /// After updating, the stored atime must match `yesterday` within 5 s.
    #[test]
    fn atime_matches_yesterday() {
        let path = std::env::temp_dir().join("rtouch_usage_update_atime.txt");
        std::fs::write(&path, b"").unwrap();

        set_access_time_of_file(path.to_str().unwrap(), "yesterday").unwrap();

        let atime_secs = std::fs::metadata(&path)
            .unwrap()
            .accessed()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expected_secs = rtouch_core::datetime::parse_time_expression("yesterday")
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let diff = atime_secs.abs_diff(expected_secs);
        assert!(diff < 5, "atime differs by {diff}s from expected");

        std::fs::remove_file(&path).unwrap();
    }
}
