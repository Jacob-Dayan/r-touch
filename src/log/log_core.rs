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

/// A logger bound to a specific log file path.
///
/// `LogCore` is constructed once (typically as a [`std::sync::LazyLock`] static)
/// and reused for every subsequent write. The log-file path is therefore
/// resolved only once — at the time the static is first accessed — rather than
/// on every individual call.
///
/// # Examples
///
/// ```no_run
/// use std::sync::LazyLock;
/// use std::path::PathBuf;
/// use rtouch_core::log::log_core::LogCore;
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
    /// Creates a new `LogCore` bound to `path`.
    ///
    /// The path is stored as-is; parent directories are created on the first
    /// [`log`](Self::log) call if they do not already exist.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Appends a timestamped log entry to the bound file.
    ///
    /// If the parent directory of the log file does not exist, it is created
    /// automatically (equivalent to `mkdir -p`).
    ///
    /// # Errors
    ///
    /// Returns an [`std::io::Error`] if the directory cannot be created, the
    /// file cannot be opened, or the write fails.
    pub fn log(&self, message: &fmt::Arguments) -> Result<()> {
        let path: &Path = &self.path;

        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(format!("{:?}: {}\n", SystemTime::now(), message).as_bytes())?;
        file.flush()
    }
}
