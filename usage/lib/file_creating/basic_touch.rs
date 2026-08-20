//! # Basic file creation with `touch`
//!
//! Demonstrates the simplest use of [`rtouch::touch`]:
//! creating a new file when it does not yet exist.
//!
//! Running `main` creates `example_basic.txt`, then immediately removes it.
//! The two tests check the return value and the file's on-disk presence
//! independently.

use std::io;

fn main() -> io::Result<()> {
    rtouch::touch("example_basic.txt", false, None, false, false)?;
    println!("example_basic.txt created.");
    std::fs::remove_file("example_basic.txt")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rtouch::ReplResult;

    /// `touch` must succeed and signal that no directory replacement was needed.
    #[test]
    fn touch_returns_not_required() {
        let path = std::env::temp_dir().join("rtouch_usage_basic_touch_1.txt");
        let _ = std::fs::remove_file(&path);

        let result = rtouch::touch(&path, false, None, false, false).unwrap();
        assert!(matches!(result, ReplResult::NotRequired));

        std::fs::remove_file(&path).unwrap();
    }

    /// The file must actually exist on disk after `touch`.
    #[test]
    fn file_exists_after_touch() {
        let path = std::env::temp_dir().join("rtouch_usage_basic_touch_2.txt");
        let _ = std::fs::remove_file(&path);

        rtouch::touch(&path, false, None, false, false).unwrap();
        assert!(std::path::Path::new(&path).exists());

        std::fs::remove_file(&path).unwrap();
    }
}
