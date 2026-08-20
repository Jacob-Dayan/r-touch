//! # Create a file and its parent directories
//!
//! When `create_parents` is `true`, [`rtouch_core::touch`] builds any missing
//! parent directories before creating the file — equivalent to `mkdir -p`.

use std::{io, path::PathBuf};

fn main() -> io::Result<()> {
    let path = PathBuf::from("deep/nested/folder/example.txt");
    rtouch_core::touch(&path, true, None, false, false)?;
    println!("Created: {}", path.display());
    std::fs::remove_dir_all("deep")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    /// Both the file and its intermediate parent directories must exist.
    #[test]
    fn creates_parents_and_file() {
        let dir = std::env::temp_dir().join("rtouch_usage_parents");
        let path = dir.join("a/b/c/file.txt");
        let _ = std::fs::remove_dir_all(&dir);

        rtouch_core::touch(&path, true, None, false, false).unwrap();

        assert!(path.exists(), "file should exist");
        assert!(path.parent().unwrap().is_dir(), "parent dirs should exist");

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
