//! # Set both `atime` and `mtime` through `touch`
//!
//! When both `atime` and `mtime` are `false` (the defaults), [`rtouch::touch`]
//! updates **both** timestamps to the supplied time — or to `SystemTime::now()`
//! when no explicit time is given.

use std::io;

fn main() -> io::Result<()> {
    let path = std::env::temp_dir().join("rtouch_usage_both_times.txt");
    std::fs::write(&path, b"")?;

    let target = rtouch::datetime::parse_time_expression("2 days ago")
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
    // atime=false, mtime=false — both are updated
    rtouch::touch(&path, false, Some(target), false, false)?;
    println!("Both timestamps set to 48 h ago: {}", path.display());

    std::fs::remove_file(&path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    /// Both atime and mtime must reflect the requested time.
    #[test]
    fn both_timestamps_updated() {
        let path = std::env::temp_dir().join("rtouch_usage_both_times_t1.txt");
        std::fs::write(&path, b"").unwrap();

        let target = rtouch::datetime::parse_time_expression("2 days ago").unwrap();
        rtouch::touch(&path, false, Some(target), false, false).unwrap();

        let meta = std::fs::metadata(&path).unwrap();
        for got in [meta.accessed().unwrap(), meta.modified().unwrap()] {
            let diff = if got > target {
                got.duration_since(target).unwrap()
            } else {
                target.duration_since(got).unwrap()
            };
            assert!(diff < Duration::from_secs(2));
        }

        std::fs::remove_file(&path).unwrap();
    }

    /// Passing `atime=true` and `mtime=true` simultaneously also updates both.
    #[test]
    fn explicit_both_flags_same_result() {
        let path = std::env::temp_dir().join("rtouch_usage_both_times_t2.txt");
        std::fs::write(&path, b"").unwrap();

        let target = rtouch::datetime::parse_time_expression("1 hour ago").unwrap();
        // atime=true AND mtime=true — both updated (same as both-false)
        rtouch::touch(&path, false, Some(target), true, true).unwrap();

        let meta = std::fs::metadata(&path).unwrap();
        for got in [meta.accessed().unwrap(), meta.modified().unwrap()] {
            let diff = if got > target {
                got.duration_since(target).unwrap()
            } else {
                target.duration_since(got).unwrap()
            };
            assert!(diff < Duration::from_secs(2));
        }

        std::fs::remove_file(&path).unwrap();
    }
}
