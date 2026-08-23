// R-touch Library
// Copyright (c) 2026 Jacob Dayan
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 or the MIT License,
// at your option. You may not use this file except in compliance with
// one of these licenses.

use fs_err::{self as fs, File, OpenOptions};
#[cfg(target_os = "windows")]
use fs_err::os::windows::fs::OpenOptionsExt;
use std::{fs::FileTimes, io, path::Path, time::SystemTime};

/// Configuration for log file paths used by the library logging subsystem.
///
/// This struct allows the binary or callers to provide explicit log file
/// locations rather than relying on global statics inside the library.
///
/// Typical fields:
/// - `success_log`: general success/activity log (r-touch.log)
/// - `error_log`: crash/error log (crashes/...)
/// - `atime_log`: access-time update log (time_modifications/atime_modification.log)
/// - `mtime_log`: modification-time update log (time_modifications/mtime_modification.log)
pub struct LogConfig {
    /// Path to the general success log file.
    pub success_log: std::path::PathBuf,
    /// Path to the crash/error log file.
    pub error_log: std::path::PathBuf,
    /// Path to the access-time (atime) modifications log file.
    pub atime_log: std::path::PathBuf,
    /// Path to the modification-time (mtime) modifications log file.
    pub mtime_log: std::path::PathBuf,
}

impl LogConfig {
    /// Construct a new `LogConfig` from explicit paths.
    pub fn new(
        success_log: std::path::PathBuf,
        error_log: std::path::PathBuf,
        atime_log: std::path::PathBuf,
        mtime_log: std::path::PathBuf,
    ) -> Self {
        Self {
            success_log,
            error_log,
            atime_log,
            mtime_log,
        }
    }

    /// Build default log paths for a specific application name from the
    /// environment with sensible fallbacks and without relying on external
    /// crates. On Unix-like systems this prefers `$XDG_DATA_HOME` and falls back
    /// to `~/.local/share`. On Windows it uses `%LOCALAPPDATA%`.
    ///
    /// The application name is intentionally supplied by the caller so the
    /// library remains reusable as a general-purpose crate, while the binary can
    /// choose the concrete app directory name (for example, `R-touch`).
    pub fn from_env_defaults_for(app_name: impl AsRef<str>) -> Self {
        use std::env;
        use std::path::PathBuf;

        #[cfg(target_family = "windows")]
        let base: PathBuf = env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or(PathBuf::from(r"C:\Users\Public\AppData\Local"));

        #[cfg(target_family = "unix")]
        let base: PathBuf = env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or(env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
            .unwrap_or(PathBuf::from("/tmp"));

        let app_root = base.join(app_name.as_ref());
        let success_log = app_root.join("logs").join("r-touch.log");
        let error_log = app_root
            .join("logs")
            .join("crashes")
            .join("file_creations.log");

        let time_dir = app_root.join("logs").join("time_modifications");
        let atime_log = time_dir.join("atime_modification.log");
        let mtime_log = time_dir.join("mtime_modification.log");

        Self::new(success_log, error_log, atime_log, mtime_log)
    }
}

pub mod datetime;
pub mod log {
    pub mod log_core;
    pub mod logmgr;
}
pub mod replace_dir;

pub use replace_dir::ReplResult;

/// Core file creation and timestamp management logic.
///
/// If `create_parents` is true, parent directories are created if they do not exist.
/// Existing directories are treated as valid touch targets and receive timestamp
/// updates by default; directory replacement only happens through explicit
/// replacement logic in the CLI layer.
///
/// When updating timestamps:
/// - `time` is the timestamp to apply (if `None`, `SystemTime::now()` is used).
/// - `atime`: if `true` and `mtime` is `false`, only the access time is updated.
/// - `mtime`: if `true` and `atime` is `false`, only the modification time is updated.
/// - If both `atime` and `mtime` are `false` (or both `true`), both access and modification times are updated.
pub fn touch<P: AsRef<Path>>(
    path: P,
    create_parents: bool,
    time: Option<SystemTime>,
    atime: bool,
    mtime: bool,
) -> io::Result<ReplResult> {
    let path_ref = path.as_ref();

    if create_parents
        && let Some(parent) = path_ref.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let target_time = time.unwrap_or_else(SystemTime::now);
    let (set_atime, set_mtime) = match (atime, mtime) {
        (true, false) => (true, false),
        (false, true) => (false, true),
        _ => (true, true),
    };

    if path_ref.is_dir() {
        // Opening directories for writing can fail with `IsADirectory` on many
        // Unix-like systems. Opening read-only is sufficient for setting
        // times via the file descriptor; on Windows a custom flag is still
        // required to open a directory handle.
        #[cfg(target_os = "windows")]
        let file = OpenOptions::new()
            .access_mode(0x0180) // FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES
            .custom_flags(0x0200_0000) // FILE_FLAG_BACKUP_SEMANTICS
            .open(path_ref)?;
        #[cfg(not(target_os = "windows"))]
        let file = OpenOptions::new().read(true).open(path_ref)?;

        let mut times = FileTimes::new();
        if set_atime {
            times = times.set_accessed(target_time);
        }
        if set_mtime {
            times = times.set_modified(target_time);
        }
        file.set_times(times)?;
        return Ok(ReplResult::NotRequired);
    }

    if !path_ref.exists() {
        let file = File::create(path_ref)?;
        let mut times = FileTimes::new();
        if set_atime {
            times = times.set_accessed(target_time);
        }
        if set_mtime {
            times = times.set_modified(target_time);
        }
        file.set_times(times)?;
    } else {
        let file = OpenOptions::new().write(true).open(path_ref)?;
        let mut times = FileTimes::new();
        if set_atime {
            times = times.set_accessed(target_time);
        }
        if set_mtime {
            times = times.set_modified(target_time);
        }
        file.set_times(times)?;
    }

    Ok(ReplResult::NotRequired)
}

/// Explicitly set access time (`atime`) of a target path.
/// gets `path` and `access_time`.
/// - `path`: The path of the file to update.
/// - `access_time`: The new access time to set.
pub fn set_access_time<P: AsRef<Path>>(path: P, access_time: SystemTime) -> io::Result<()> {
    let path_ref = path.as_ref();
    let file = OpenOptions::new().write(true).open(path_ref)?;

    let times = FileTimes::new().set_accessed(access_time);
    file.set_times(times)?;
    Ok(())
}

/// Explicitly set modification time (`mtime`) of a target path.
/// gets `path` and `modification_time`.
/// - `path`: The path of the file to update.
/// - `modification_time`: The new modification time to set.
pub fn set_modification_time<P: AsRef<Path>>(
    path: P,
    modification_time: SystemTime,
) -> io::Result<()> {
    let path_ref = path.as_ref();
    let file = OpenOptions::new().write(true).open(path_ref)?;

    let times = FileTimes::new().set_modified(modification_time);
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
        let res = touch(&file_path, false, None, false, false).unwrap();
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
        let res = touch(&file_path, false, Some(past_time), true, false).unwrap();
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
        let res = touch(&file_path, false, Some(requested_atime), true, false).unwrap();
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
    fn test_existing_file_with_mtime_preserves_atime() {
        let temp_dir = std::env::temp_dir().join("rtouch_test_mtime");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test_existing_mtime.txt");
        fs::write(&file_path, "initial content").unwrap();

        let old_mtime = SystemTime::now() - Duration::from_secs(3600 * 10);
        let old_atime = SystemTime::now() - Duration::from_secs(3600 * 5);
        let file = fs::OpenOptions::new().write(true).open(&file_path).unwrap();
        file.set_times(
            std::fs::FileTimes::new()
                .set_accessed(old_atime)
                .set_modified(old_mtime),
        )
        .unwrap();
        drop(file);

        let requested_mtime = SystemTime::now() - Duration::from_secs(60);
        let res = touch(&file_path, false, Some(requested_mtime), false, true).unwrap();
        assert!(matches!(res, ReplResult::NotRequired));

        let metadata = fs::metadata(&file_path).unwrap();
        let atime = metadata.accessed().unwrap();
        let mtime = metadata.modified().unwrap();

        let mtime_diff = if mtime > requested_mtime {
            mtime.duration_since(requested_mtime).unwrap()
        } else {
            requested_mtime.duration_since(mtime).unwrap()
        };
        assert!(mtime_diff < Duration::from_secs(2));

        let atime_diff = if atime > old_atime {
            atime.duration_since(old_atime).unwrap()
        } else {
            old_atime.duration_since(atime).unwrap()
        };
        assert!(atime_diff < Duration::from_secs(2));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_existing_file_without_flags_updates_both_times() {
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
        let res = touch(&file_path, false, None, false, false).unwrap();
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

    #[test]
    fn test_touch_directory_updates_metadata_without_replacing() {
        let temp_dir = std::env::temp_dir().join("rtouch_dir_metadata_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(temp_dir.join("nested.txt"), "hello").unwrap();

        let target_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let res = touch(&temp_dir, false, Some(target_time), true, false).unwrap();
        assert!(matches!(res, ReplResult::NotRequired));

        let metadata = fs::metadata(&temp_dir).unwrap();
        let atime = metadata.accessed().unwrap();
        let diff = if atime > target_time {
            atime.duration_since(target_time).unwrap()
        } else {
            target_time.duration_since(atime).unwrap()
        };
        assert!(diff < Duration::from_secs(2));
        assert!(temp_dir.is_dir());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
