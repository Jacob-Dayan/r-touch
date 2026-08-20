//! # Set both `atime` and `mtime` through `touch`
//!
//! When both `atime` and `mtime` are `false` (the defaults), [`rtouch_core::touch`]
//! updates **both** timestamps to the supplied time — or to `SystemTime::now()`
//! when no explicit time is given.

use std::{io, time::{Duration, SystemTime}};

fn main() -> io::Result<()> {
    let path = std::env::temp_dir().join("rtouch_usage_both_times.txt");
    std::fs::write(&path, b"")?;

    let target = SystemTime::now() - Duration::from_secs(3_600 * 48);
    // atime=false, mtime=false → both are updated
    rtouch_core::touch(&path, false, Some(target), false, false)?;
    println!("Both timestamps set to 48 h ago: {}", path.display());

    std::fs::remove_file(&path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    /// Both atime and mtime must reflect the requested time.
    #[test]
    fn both_timestamps_updated() {
        let path = std::env::temp_dir().join("rtouch_usage_both_times_t1.txt");
        std::fs::write(&path, b"").unwrap();

        let target = SystemTime::now() - Duration::from_secs(3_600 * 48);
        rtouch_core::touch(&path, false, Some(target), false, false).unwrap();

        let meta = std::fs::metadata(&path).unwrap();
        for got in [meta.accessed().unwrap(), meta.modified().unwrap()] {
            let diff = if got > target { got.duration_since(target) } else { target.duration_since(got) };
            assert!(diff.unwrap() < Duration::from_secs(2));
        }

        std::fs::remove_file(&path).unwrap();
    }

    /// Passing `atime=true` and `mtime=true` simultaneously also updates both.
    #[test]
    fn explicit_both_flags_same_result() {
        let path = std::env::temp_dir().join("rtouch_usage_both_times_t2.txt");
        std::fs::write(&path, b"").unwrap();

        let target = SystemTime::now() - Duration::from_secs(3_600);
        // atime=true AND mtime=true → both updated (same as both-false)
        rtouch_core::touch(&path, false, Some(target), true, true).unwrap();

        let meta = std::fs::metadata(&path).unwrap();
        for got in [meta.accessed().unwrap(), meta.modified().unwrap()] {
            let diff = if got > target { got.duration_since(target) } else { target.duration_since(got) };
            assert!(diff.unwrap() < Duration::from_secs(2));
        }

        std::fs::remove_file(&path).unwrap();
    }
}
