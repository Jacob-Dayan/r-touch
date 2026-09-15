// R-touch Library
// Copyright (c) 2026 Jacob Dayan
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 or the MIT License,
// at your option. You may not use this file except in compliance with
// one of these licenses.

#[cfg(target_os = "windows")]
use fs_err::os::windows::fs::OpenOptionsExt;
use fs_err::{self as fs, File, OpenOptions};
use std::{fs::FileTimes, io, path::Path, time::SystemTime};

/// configuration for log file paths used by the library logging subsystem
///
/// allows binary or callers to provide explicit log file locations
/// rather than relying on global statics inside the library
pub struct LogConfig {
    /// path to the general success log file
    pub success_log: std::path::PathBuf,
    /// path to the crash/error log file
    pub error_log: std::path::PathBuf,
    /// path to the access-time (atime) modifications log file
    pub atime_log: std::path::PathBuf,
    /// path to the modification-time (mtime) modifications log file
    pub mtime_log: std::path::PathBuf,
}

impl LogConfig {
    /// construct a new [`LogConfig`] from explicit paths
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

    /// construct a [`LogConfig`] with all standard log subpaths relative to `log_dir`
    /// for a specific application name
    pub fn from_log_dir_for<P: AsRef<Path>>(log_dir: P, app_name: impl AsRef<str>) -> Self {
        let dir = log_dir.as_ref();
        let app_str = app_name.as_ref();
        let log_name = format!("{}.log", app_str.to_lowercase());
        let success_log = dir.join(log_name);
        let error_log = dir.join("crashes").join("file_creations.log");

        let time_dir = dir.join("time_modifications");
        let atime_log = time_dir.join("atime_modification.log");
        let mtime_log = time_dir.join("mtime_modification.log");

        Self::new(success_log, error_log, atime_log, mtime_log)
    }

    /// construct a [`LogConfig`] with all standard log subpaths relative to `log_dir`
    pub fn from_log_dir<P: AsRef<Path>>(log_dir: P) -> Self {
        Self::from_log_dir_for(log_dir, APP_NAME)
    }

    /// build default log paths for default application (`APP_NAME`) from environment
    pub fn from_env_defaults() -> Self {
        Self::from_env_defaults_for(APP_NAME)
    }

    /// build default log paths for a specific application name from environment
    /// with sensible fallbacks and without relying on external crates
    /// (checks `<APP_NAME>_LOG_DIR` and `RTOUCH_LOG_DIR` environment variables first)
    pub fn from_env_defaults_for(app_name: impl AsRef<str>) -> Self {
        use std::path::PathBuf;

        let app_str = app_name.as_ref();
        let env_var_name = format!("{}_LOG_DIR", app_str.to_uppercase().replace('-', "_"));
        if let Some(dir) =
            std::env::var_os(&env_var_name).or_else(|| std::env::var_os("RTOUCH_LOG_DIR"))
        {
            return Self::from_log_dir_for(dir, app_str);
        }

        #[cfg(target_family = "windows")]
        let log_dir = {
            use std::env;
            let base: PathBuf = env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or(PathBuf::from(r"C:\Users\Public\AppData\Local"));
            base.join(app_str).join("logs")
        };

        #[cfg(target_family = "unix")]
        let log_dir = {
            let is_root = unsafe { libc::getuid() == 0 };
            if is_root {
                PathBuf::from("/var/log").join(app_str)
            } else if let Some(val) = std::env::var_os("XDG_STATE_HOME").filter(|s| !s.is_empty()) {
                PathBuf::from(val).join(app_str)
            } else if let Some(home) =
                dirs_next::home_dir().or_else(|| std::env::var_os("HOME").map(PathBuf::from))
            {
                home.join(".local").join("state").join(app_str)
            } else {
                PathBuf::from("/tmp").join(app_str).join("logs")
            }
        };

        Self::from_log_dir_for(log_dir, app_str)
    }

    /// ensure log dirs and files have standard permissions
    pub fn ensure_permissions(&self) {
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;

            for path in [
                &self.success_log,
                &self.error_log,
                &self.atime_log,
                &self.mtime_log,
            ] {
                if let Some(parent) = path.parent() {
                    let _ = fs::create_dir_all(parent);
                    if let Ok(meta) = fs::metadata(parent) {
                        let mut perms = meta.permissions();
                        let target_mode = if unsafe { libc::getuid() == 0 } {
                            0o755
                        } else {
                            0o700
                        };
                        if perms.mode() & 0o777 != target_mode {
                            perms.set_mode(target_mode);
                            let _ = fs::set_permissions(parent, perms);
                        }
                    }
                }
                if path.exists()
                    && let Ok(meta) = fs::metadata(path)
                {
                    let mut perms = meta.permissions();
                    let target_mode = if unsafe { libc::getuid() == 0 } {
                        0o644
                    } else {
                        0o600
                    };
                    if perms.mode() & 0o777 != target_mode {
                        perms.set_mode(target_mode);
                        let _ = fs::set_permissions(path, perms);
                    }
                }
            }
        }
    }
}

pub mod conf;
pub mod datetime;
pub mod log {
    pub mod log_core;
    pub mod logmgr;
}
pub mod replace_dir;

pub use conf::{APP_NAME, AppConfig, TimeModifyConfig};
pub use replace_dir::ReplResult;

/// core file creation and timestamp management logic
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
        // opening directories for writing fails with `IsADirectory` on Unix;
        // read-only is sufficient, while Windows needs backup semantics flag
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

    let file = if !path_ref.exists() {
        File::create(path_ref)?
    } else {
        OpenOptions::new().write(true).open(path_ref)?
    };

    let mut times = FileTimes::new();
    if set_atime {
        times = times.set_accessed(target_time);
    }
    if set_mtime {
        times = times.set_modified(target_time);
    }
    file.set_times(times)?;

    Ok(ReplResult::NotRequired)
}

/// set access time (`atime`) of a target path
pub fn set_access_time<P: AsRef<Path>>(path: P, access_time: SystemTime) -> io::Result<()> {
    let path_ref = path.as_ref();
    let file = OpenOptions::new().write(true).open(path_ref)?;

    let times = FileTimes::new().set_accessed(access_time);
    file.set_times(times)?;
    Ok(())
}

/// set modification time (`mtime`) of a target path
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
