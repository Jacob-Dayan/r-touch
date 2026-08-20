//! # Touch a file with a relative date expression
//!
//! [`rtouch_core::datetime::parse_time_expression`] converts human-readable
//! strings such as `"yesterday"` or `"2 days ago"` into a [`std::time::SystemTime`],
//! which can then be passed directly to [`rtouch_core::touch`].

use std::io;

fn touch_with_expr(path: &str, expr: &str) -> io::Result<rtouch_core::ReplResult> {
    let time = rtouch_core::datetime::parse_time_expression(expr)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
    rtouch_core::touch(path, false, Some(time), true, false)
}

fn main() -> io::Result<()> {
    let path = std::env::temp_dir().join("rtouch_usage_relative.txt");
    touch_with_expr(path.to_str().unwrap(), "yesterday")?;
    println!("Touched with 'yesterday': {}", path.display());
    std::fs::remove_file(&path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use super::touch_with_expr;

    /// The file's atime should be approximately 24 h in the past.
    #[test]
    fn yesterday_sets_atime_to_24h_ago() {
        let path = std::env::temp_dir().join("rtouch_usage_relative_t1.txt");
        let _ = std::fs::remove_file(&path);

        touch_with_expr(path.to_str().unwrap(), "yesterday").unwrap();

        let atime = std::fs::metadata(&path).unwrap().accessed().unwrap();
        let expected = rtouch_core::datetime::parse_time_expression("yesterday").unwrap();
        let diff = if atime > expected { atime.duration_since(expected) } else { expected.duration_since(atime) };
        // Allow up to 5 seconds of tolerance (test runner startup + OS rounding).
        assert!(diff.unwrap() < Duration::from_secs(5));

        std::fs::remove_file(&path).unwrap();
    }
}
