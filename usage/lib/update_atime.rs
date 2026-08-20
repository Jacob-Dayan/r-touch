use std::{fs, io, path::Path, time::UNIX_EPOCH};

/// Sets atime (access time) of `foo.txt` to yesterday.
/// Checks if `foo.txt` exists, alerts the user and creates it with yesterday's access time if not.
/// If it does exist, updates its access time to yesterday.
///
/// Then for making sure the access time was updated correctly, it creates:
/// - `file_access_time` with fs::metadata of foo.txt, and applies the `.accessed()` method to get the access time.
/// - `expected_time` with the expected access time (yesterday), parsed from [`rtouch_core::datetime::parse_time_expression`].
///
/// Finally, it converts both to seconds, and asserts that they are equal.
fn main() -> io::Result<()> {
    println!("Setting the access time of foo.txt to yesterday!");
    if !Path::new("foo.txt").exists() {
        eprintln!("Oh, no! `foo.txt` not found!");
        println!("Creating foo.txt with access time of yesterday...");

        update_access_time_of_file("foo.txt", "yesterday")?;
    }
    update_access_time_of_file("foo.txt", "yesterday")?;

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

#[rustfmt::skip]
fn update_access_time_of_file(path: &str, atime: &str) -> Result<rtouch_core::ReplResult, io::Error> {
    rtouch_core::create(
        path,
        false, // Don't create parent directories
        Some(rtouch_core::datetime::parse_time_expression(atime).unwrap()),
        true, // Update access-time
        false, // Don't update modification-time
    )
}
