//! # Replace a directory with a file
//!
//! If the target path points to an **existing directory**, [`rtouch_core::touch`]
//! prompts the user interactively and returns one of three outcomes encoded in
//! [`rtouch_core::ReplResult`]:
//!
//! | Variant | Meaning |
//! |---------|---------|
//! | `Completed` | User confirmed; directory was removed and replaced with a file. |
//! | `Aborted`   | User declined; the directory is intact. |
//! | `NotRequired` | Path was not a directory; normal touch occurred. |
//!
//! This example file documents the variants and tests the `NotRequired` path
//! (since automated tests cannot drive the interactive prompt).

use std::io;

fn main() -> io::Result<()> {
    // When called on a plain file (or a new path), the result is NotRequired.
    let path = std::env::temp_dir().join("rtouch_usage_repl_main.txt");
    let result = rtouch_core::touch(&path, false, None, false, false)?;
    match result {
        rtouch_core::ReplResult::Completed   => println!("Directory replaced."),
        rtouch_core::ReplResult::Aborted     => println!("Replacement aborted."),
        rtouch_core::ReplResult::NotRequired => println!("No directory to replace."),
    }
    std::fs::remove_file(&path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rtouch_core::ReplResult;

    /// Touching a normal (non-directory) path always returns `NotRequired`.
    #[test]
    fn non_directory_returns_not_required() {
        let path = std::env::temp_dir().join("rtouch_usage_repl_t1.txt");
        let _ = std::fs::remove_file(&path);

        let result = rtouch_core::touch(&path, false, None, false, false).unwrap();
        assert!(matches!(result, ReplResult::NotRequired));

        std::fs::remove_file(&path).unwrap();
    }
}
