// R-touch Library
// Copyright (c) 2026 Jacob Dayan
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 or the MIT License,
// at your option. You may not use this file except in compliance with
// one of these licenses.

use crate::log::log_core::LogCore;
use std::{fmt, io};

/// Internal helper: write a message to `path` using `LogCore`.
fn write_log(path: &std::path::Path, message: &fmt::Arguments) -> io::Result<()> {
    LogCore::new(path.to_path_buf()).log(message).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Cannot write log to {}: {e}", path.display()),
        )
    })
}

/// Appends an entry to the general success log file configured in `cfg`.
///
/// # Errors
///
/// Returns an error if the log file cannot be written.
pub fn success_log(cfg: &crate::LogConfig, message: &fmt::Arguments) -> io::Result<()> {
    write_log(&cfg.success_log, message)
}

/// Appends an entry to the error/crash log file configured in `cfg`.
///
/// # Errors
///
/// Returns an error if the log file cannot be written.
pub fn error_log(cfg: &crate::LogConfig, message: &fmt::Arguments) -> io::Result<()> {
    write_log(&cfg.error_log, message)
}

/// Appends an entry for a successful access-time (atime) update.
pub fn atime_modification_success(
    cfg: &crate::LogConfig,
    message: &fmt::Arguments,
) -> io::Result<()> {
    write_log(&cfg.atime_log, message)
}

/// Appends an entry for a successful modification-time (mtime) update.
pub fn mtime_modification_success(
    cfg: &crate::LogConfig,
    message: &fmt::Arguments,
) -> io::Result<()> {
    write_log(&cfg.mtime_log, message)
}

/// Appends an entry describing a failure related to time modification (parsing
/// or update failures). Errors are still written under the `crashes/` log in
/// the configured `error_log` path.
pub fn time_modification_failure(cfg: &crate::LogConfig, message: &fmt::Arguments) -> io::Result<()> {
    // Keep failures under the configured `error_log` (crashes/...) to preserve
    // the previous behaviour of writing errors to the crash logs.
    write_log(&cfg.error_log, message)
}
