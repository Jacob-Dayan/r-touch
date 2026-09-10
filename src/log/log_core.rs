// R-touch Library
// Copyright (c) 2026 Jacob Dayan
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 or the MIT License,
// at your option. You may not use this file except in compliance with
// one of these licenses.

use fs_err::{self as fs, OpenOptions};
use std::{
    fmt,
    io::{Result, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

/// logger bound to a specific log file path
///
/// [`LogCore`] is constructed once (typically as a [`std::sync::LazyLock`] static)
/// and reused for subsequent writes; log file path is resolved once at static access
///
/// # Examples
///
/// ```no_run
/// use std::sync::LazyLock;
/// use std::path::PathBuf;
/// use rtouch::log::log_core::LogCore;
///
/// static LOGGER: LazyLock<LogCore> = LazyLock::new(|| {
///     LogCore::new(PathBuf::from("/var/log/myapp/app.log"))
/// });
///
/// LOGGER.log(&format_args!("application started")).unwrap();
/// ```
pub struct LogCore {
    path: PathBuf,
}

impl LogCore {
    /// create a new [`LogCore`] bound to `path`
    ///
    /// path is stored as-is; parent directories are created on first
    /// [`log`](Self::log) call if they do not already exist
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// append a timestamped log entry to the bound file
    ///
    /// creates parent directory automatically if missing (equivalent to `mkdir -p`)
    ///
    /// # Errors
    ///
    /// returns an [`std::io::Error`] if directory cannot be created, file cannot
    /// be opened, or write fails
    pub fn log(&self, message: &fmt::Arguments) -> Result<()> {
        let path: &Path = &self.path;

        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
            #[cfg(target_family = "unix")]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = fs::metadata(parent) {
                    let mut perms = meta.permissions();
                    if perms.mode() & 0o777 != 0o755 {
                        perms.set_mode(0o755);
                        let _ = fs::set_permissions(parent, perms);
                    }
                }
            }
        }

        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = file.metadata() {
                let mut perms = meta.permissions();
                if perms.mode() & 0o777 != 0o644 {
                    perms.set_mode(0o644);
                    let _ = file.set_permissions(perms);
                }
            }
        }
        file.write_all(format!("{:?}: {}\n", SystemTime::now(), message).as_bytes())?;
        file.flush()
    }
}
