//! # Simply creating a file (original example)
//!
//! The simplest possible invocation of [`rtouch::touch`]:
//! create `foo.txt` in the current directory and immediately clean up.

macro_rules! foo {
    ($name:expr) => {
        rtouch::touch($name, false, None, false, false)
    };
}

fn main() {
    foo!("foo.txt").unwrap();
    std::fs::remove_file("foo.txt").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_foo() {
        foo!("foo_create.txt").unwrap();
        assert!(std::path::Path::new("foo_create.txt").exists());
        let _ = std::fs::remove_file("foo_create.txt");
    }

    #[test]
    fn is_ok() {
        assert!(foo!("foo_is_ok.txt").is_ok());
        let _ = std::fs::remove_file("foo_is_ok.txt");
    }
}
