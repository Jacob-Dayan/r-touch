use fs_err::{self as fs, File, OpenOptions};
use std::{fs::FileTimes, io, path::Path, time::SystemTime};

pub mod datetime;
pub mod log {
    pub mod logger;
    pub mod logmgr;
}
pub mod replace_dir;

pub use replace_dir::ReplResult;

/// Shortcut for `std::io::Error::new(std::io::ErrorKind::Other, e)`
#[macro_export]
macro_rules! new_io_error {
    ($e:expr) => {
        std::io::Error::new(std::io::ErrorKind::Other, $e)
    };
}

/// Core file creation and timestamp management logic.
pub fn create<P: AsRef<Path>>(
    path: P,
    create_parents: bool,
    access_time: Option<SystemTime>,
) -> io::Result<ReplResult> {
    let path_ref = path.as_ref();

    if create_parents
        && let Some(parent) = path_ref.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    if path_ref.is_dir() {
        let res = replace_dir::replace(path_ref)?;
        if let ReplResult::Completed = res
            && let Some(atime) = access_time
        {
            let file = OpenOptions::new().write(true).open(path_ref)?;
            let times = FileTimes::new().set_accessed(atime);
            file.set_times(times)?;
        }
        return Ok(res);
    }

    if !path_ref.exists() {
        let file = File::create(path_ref)?;
        if let Some(atime) = access_time {
            let times = FileTimes::new().set_accessed(atime);
            file.set_times(times)?;
        }
    } else {
        let file = OpenOptions::new().write(true).open(path_ref)?;
        let times = match access_time {
            Some(atime) => FileTimes::new().set_accessed(atime),
            None => {
                let now = SystemTime::now();
                FileTimes::new().set_accessed(now).set_modified(now)
            }
        };
        file.set_times(times)?;
    }

    Ok(ReplResult::NotRequired)
}

/// Explicitly update access time (`atime`) of a target path.
pub fn update_access_time<P: AsRef<Path>>(path: P, access_time: SystemTime) -> io::Result<()> {
    let path_ref = path.as_ref();
    let file = OpenOptions::new().write(true).open(path_ref)?;

    let times = FileTimes::new().set_accessed(access_time);
    file.set_times(times)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_create_new_file_without_access_time() {
        let temp_dir = std::env::temp_dir().join("rtouch_test_1");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test_file.txt");
        let res = create(&file_path, false, None).unwrap();
        assert!(matches!(res, ReplResult::NotRequired));
        assert!(file_path.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_create_new_file_with_access_time() {
        let temp_dir = std::env::temp_dir().join("rtouch_test_2");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test_file_atime.txt");
        let past_time = SystemTime::now() - Duration::from_secs(3600 * 24 * 10);
        let res = create(&file_path, false, Some(past_time)).unwrap();
        assert!(matches!(res, ReplResult::NotRequired));
        assert!(file_path.exists());

        let metadata = fs::metadata(&file_path).unwrap();
        let atime = metadata.accessed().unwrap();
        let mtime = metadata.modified().unwrap();

        let atime_diff = if atime > past_time {
            atime.duration_since(past_time).unwrap()
        } else {
            past_time.duration_since(atime).unwrap()
        };
        assert!(atime_diff < Duration::from_secs(2));

        let now = SystemTime::now();
        let mtime_diff = if now > mtime {
            now.duration_since(mtime).unwrap()
        } else {
            mtime.duration_since(now).unwrap()
        };
        assert!(mtime_diff < Duration::from_secs(5));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_existing_file_with_access_time_preserves_mtime() {
        let temp_dir = std::env::temp_dir().join("rtouch_test_3");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test_existing.txt");
        fs::write(&file_path, "initial content").unwrap();

        let old_mtime = SystemTime::now() - Duration::from_secs(3600 * 5);
        let old_atime = SystemTime::now() - Duration::from_secs(3600 * 10);
        let file = fs::OpenOptions::new().write(true).open(&file_path).unwrap();
        file.set_times(
            std::fs::FileTimes::new()
                .set_accessed(old_atime)
                .set_modified(old_mtime),
        )
        .unwrap();
        drop(file);

        let requested_atime = SystemTime::now() - Duration::from_secs(60);
        let res = create(&file_path, false, Some(requested_atime)).unwrap();
        assert!(matches!(res, ReplResult::NotRequired));

        let metadata = fs::metadata(&file_path).unwrap();
        let atime = metadata.accessed().unwrap();
        let mtime = metadata.modified().unwrap();

        let atime_diff = if atime > requested_atime {
            atime.duration_since(requested_atime).unwrap()
        } else {
            requested_atime.duration_since(atime).unwrap()
        };
        assert!(atime_diff < Duration::from_secs(2));

        let mtime_diff = if mtime > old_mtime {
            mtime.duration_since(old_mtime).unwrap()
        } else {
            old_mtime.duration_since(mtime).unwrap()
        };
        assert!(mtime_diff < Duration::from_secs(2));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_existing_file_without_access_time_updates_both_times() {
        let temp_dir = std::env::temp_dir().join("rtouch_test_4");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test_existing_both.txt");
        fs::write(&file_path, "initial content").unwrap();

        let old_mtime = SystemTime::now() - Duration::from_secs(3600 * 5);
        let old_atime = SystemTime::now() - Duration::from_secs(3600 * 10);
        let file = fs::OpenOptions::new().write(true).open(&file_path).unwrap();
        file.set_times(
            std::fs::FileTimes::new()
                .set_accessed(old_atime)
                .set_modified(old_mtime),
        )
        .unwrap();
        drop(file);

        let before_touch = SystemTime::now();
        let res = create(&file_path, false, None).unwrap();
        assert!(matches!(res, ReplResult::NotRequired));

        let metadata = fs::metadata(&file_path).unwrap();
        let atime = metadata.accessed().unwrap();
        let mtime = metadata.modified().unwrap();

        let atime_diff = if atime > before_touch {
            atime.duration_since(before_touch).unwrap()
        } else {
            before_touch.duration_since(atime).unwrap()
        };
        assert!(atime_diff < Duration::from_secs(5));

        let mtime_diff = if mtime > before_touch {
            mtime.duration_since(before_touch).unwrap()
        } else {
            before_touch.duration_since(mtime).unwrap()
        };
        assert!(mtime_diff < Duration::from_secs(5));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
