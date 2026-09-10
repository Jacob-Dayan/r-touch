// R-touch Library
// Copyright (c) 2026 Jacob Dayan
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 or the MIT License,
// at your option. You may not use this file except in compliance with
// one of these licenses.

use crate::{LogConfig, ReplResult, touch};
use fs_err as fs;
use std::time::{Duration, SystemTime};

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

#[test]
fn test_log_config_from_env_defaults_for() {
    #[cfg(target_family = "unix")]
    use std::path::PathBuf;

    let cfg = LogConfig::from_env_defaults_for("test-app");
    #[cfg(target_family = "unix")]
    {
        assert_eq!(
            cfg.success_log,
            PathBuf::from("/var/log/test-app/r-touch.log")
        );
        assert_eq!(
            cfg.error_log,
            PathBuf::from("/var/log/test-app/crashes/file_creations.log")
        );
        assert_eq!(
            cfg.atime_log,
            PathBuf::from("/var/log/test-app/time_modifications/atime_modification.log")
        );
        assert_eq!(
            cfg.mtime_log,
            PathBuf::from("/var/log/test-app/time_modifications/mtime_modification.log")
        );
    }
    #[cfg(target_family = "windows")]
    {
        assert!(cfg.success_log.ends_with(r"test-app\logs\r-touch.log"));
        assert!(
            cfg.error_log
                .ends_with(r"test-app\logs\crashes\file_creations.log")
        );
        assert!(
            cfg.atime_log
                .ends_with(r"test-app\logs\time_modifications\atime_modification.log")
        );
        assert!(
            cfg.mtime_log
                .ends_with(r"test-app\logs\time_modifications\mtime_modification.log")
        );
    }
}

#[test]
fn test_log_config_from_log_dir() {
    use std::path::PathBuf;

    let base = PathBuf::from("/custom/logging/base");
    let cfg = LogConfig::from_log_dir(&base);

    assert_eq!(cfg.success_log, base.join("r-touch.log"));
    assert_eq!(
        cfg.error_log,
        base.join("crashes").join("file_creations.log")
    );
    assert_eq!(
        cfg.atime_log,
        base.join("time_modifications").join("atime_modification.log")
    );
    assert_eq!(
        cfg.mtime_log,
        base.join("time_modifications").join("mtime_modification.log")
    );
}

#[test]
fn test_log_config_env_override() {
    use std::path::PathBuf;

    unsafe {
        std::env::set_var("TEST_CUSTOM_APP_LOG_DIR", "/tmp/test_custom_env_dir");
    }
    let cfg = LogConfig::from_env_defaults_for("test-custom-app");
    assert_eq!(
        cfg.success_log,
        PathBuf::from("/tmp/test_custom_env_dir/r-touch.log")
    );
    assert_eq!(
        cfg.error_log,
        PathBuf::from("/tmp/test_custom_env_dir/crashes/file_creations.log")
    );
    unsafe {
        std::env::remove_var("TEST_CUSTOM_APP_LOG_DIR");
    }
}
