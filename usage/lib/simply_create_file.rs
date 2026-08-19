macro_rules! foo {
    () => {
        rtouch_core::touch("foo.txt", false, None, false, false)
    };
}

fn main() {
    foo!().unwrap();
    std::fs::remove_file("foo.txt").unwrap();
}

#[cfg(test)]
#[test]
fn create_foo() {
    foo!().unwrap();
    assert!(std::path::Path::new("foo.txt").exists());
    std::fs::remove_file("foo.txt").unwrap();
}

#[test]
fn is_ok() {
    assert!(foo!().is_ok())
}
