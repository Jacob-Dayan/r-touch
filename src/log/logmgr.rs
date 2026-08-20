// R-touch Library
// Copyright (c) 2026 Jacob Dayan
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 or the MIT License,
// at your option. You may not use this file except in compliance with
// one of these licenses.

use crate::log::log_core::LogCore;
use std::{fmt, io, path::PathBuf, sync::LazyLock};

// not really OS_ROOT but the root directory for log files
#[cfg(target_family = "windows")]
const OS_ROOT: Option<&str> = Some("C:\\Users\\Public\\AppData\\Local");
#[cfg(target_family = "unix")]
const OS_ROOT: Option<&str> = Some("/var/log");
#[cfg(not(any(target_family = "windows", target_family = "unix")))]
const OS_ROOT: Option<&str> = None;

/// Resolves the base data-local directory, falling back to the OS-specific
/// root when [`dirs_next::data_local_dir`] returns `None`.
///
/// Returns `None` only on platforms where neither source is available.
fn base_dir() -> Option<PathBuf> {
    dirs_next::data_local_dir().or_else(|| OS_ROOT.map(PathBuf::from))
}

/// Constructs a log-file path from the base directory and the given sub-path
/// segments, panicking at static-init time if no base directory is available.
///
/// Because this is only called from `LazyLock` initialisers it panics instead
/// of returning an error; a missing log directory is an unrecoverable
/// configuration problem for this binary.
fn make_log_path(segments: &[&str]) -> PathBuf {
    let mut path = base_dir().expect("Cannot determine log directory");
    for seg in segments {
        path = path.join(seg);
    }
    path
}

/// Logger for general successful operations.
///
/// Writes to `<data_local_dir>/R-touch/logs/r-touch.log`.
static SUCCESS: LazyLock<LogCore> =
    LazyLock::new(|| LogCore::new(make_log_path(&["R-touch", "logs", "r-touch.log"])));

/// Logger for file-creation errors and crash events.
///
/// Writes to `<data_local_dir>/R-touch/logs/crashes/file_creations.log`.
static ERROR: LazyLock<LogCore> = LazyLock::new(|| {
    LogCore::new(make_log_path(&[
        "R-touch",
        "logs",
        "crashes",
        "file_creations.log",
    ]))
});

/// Logger for successful access-time updates.
///
/// Writes to `<data_local_dir>/R-touch/logs/access_time/access-time_success.log`.
static ACCESS_TIME_SUCCESS: LazyLock<LogCore> = LazyLock::new(|| {
    LogCore::new(make_log_path(&[
        "R-touch",
        "logs",
        "access_time",
        "access-time_success.log",
    ]))
});

/// Logger for failed access-time updates or date-expression parsing errors.
///
/// Writes to `<data_local_dir>/R-touch/logs/crashes/access-time_failure.log`.
static ACCESS_TIME_FAILURE: LazyLock<LogCore> = LazyLock::new(|| {
    LogCore::new(make_log_path(&[
        "R-touch",
        "logs",
        "crashes",
        "access-time_failure.log",
    ]))
});

/// Logs a successful file operation.
///
/// Appends `message` to the general success log
/// (`<data_local_dir>/R-touch/logs/r-touch.log`).
///
/// # Errors
///
/// Returns an error if the log file cannot be written.
pub fn success_log(message: &fmt::Arguments) -> io::Result<()> {
    SUCCESS.log(message).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Cannot log actions: {e}"),
        )
    })
}

/// Logs a file-creation error or unexpected crash event.
///
/// Appends `message` to the error/crash log
/// (`<data_local_dir>/R-touch/logs/crashes/file_creations.log`).
///
/// # Errors
///
/// Returns an error if the log file cannot be written.
pub fn error_log(message: &fmt::Arguments) -> io::Result<()> {
    ERROR.log(message).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("Cannot log error: {e}"))
    })
}

/// Logs a successful access-time or modification-time update.
///
/// Appends `message` to the access-time success log
/// (`<data_local_dir>/R-touch/logs/access_time/access-time_success.log`).
///
/// # Errors
///
/// Returns an error if the log file cannot be written.
pub fn access_time_success(message: &fmt::Arguments) -> io::Result<()> {
    ACCESS_TIME_SUCCESS.log(message).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Cannot log access time success: {e}"),
        )
    })
}

/// Logs a failed access-time update or date-expression parsing failure.
///
/// Appends `message` to the access-time failure log
/// (`<data_local_dir>/R-touch/logs/crashes/access-time_failure.log`).
///
/// # Errors
///
/// Returns an error if the log file cannot be written.
pub fn access_time_failure(message: &fmt::Arguments) -> io::Result<()> {
    ACCESS_TIME_FAILURE.log(message).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Cannot log access time failure: {e}"),
        )
    })
}
