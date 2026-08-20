//! # Simply creating a file (original example)
//!
//! The simplest possible invocation of [`rtouch::touch`]:
//! create `foo.txt` in the current directory and immediately clean up.

macro_rules! foo {
    () => {
        rtouch::touch("foo.txt", false, None, false, false)
    };
}

fn main() {
    foo!().unwrap();
    std::fs::remove_file("foo.txt").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_foo() {
        foo!().unwrap();
        assert!(std::path::Path::new("foo.txt").exists());
        std::fs::remove_file("foo.txt").unwrap();
    }

    #[test]
    fn is_ok() {
        assert!(foo!().is_ok());
        let _ = std::fs::remove_file("foo.txt");
    }
}
