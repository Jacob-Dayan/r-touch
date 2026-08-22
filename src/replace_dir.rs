// R-touch Library
// Copyright (c) 2026 Jacob Dayan
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 or the MIT License,
// at your option. You may not use this file except in compliance with
// one of these licenses.

use fs_err::{self as fs, File};
use std::io;
use std::path::Path;

/// Outcome of a directory-to-file replacement attempt.
///
/// Returned by [`replace`] and propagated up to [`crate::touch`] so that
/// callers (e.g. the CLI binary) can decide how to log or report the result.
pub enum ReplResult {
    /// The directory was successfully removed and replaced with an empty file.
    Completed,
    /// The user declined the replacement prompt; no changes were made.
    Aborted,
    /// No replacement was necessary (the path was not a directory).
    NotRequired,
}

/// User-input decision for a directory-replacement prompt.
///
/// Constructed from stdin via [`Action::new`] and consumed by [`replace`].
pub enum Action {
    /// The user confirmed the replacement (`y` / `yes` / empty input).
    Accept,
    /// The user declined the replacement or stdin could not be read.
    Abort,
}

impl Action {
    /// Prompts the user interactively and returns their decision.
    ///
    /// Prints a confirmation message to stderr, then reads a single line from
    /// stdin. Any input that is not `y`, `yes`, or an empty line is treated as
    /// [`Action::Abort`].  A read error also yields [`Action::Abort`].
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        eprintln!(
            "'{p}' is a directory. Do you want to delete directory and replace it with the file? (y/n)",
            p = path.as_ref().display()
        );
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return Action::Abort;
        }
        match input.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" | "" => Action::Accept,
            _ => Action::Abort,
        }
    }
}

/// Attempts to replace the directory at `path` with an empty file.
///
/// Prompts the user interactively via [`Action::new`]. If the user confirms,
/// the directory tree is removed with [`fs_err::remove_dir_all`] and an empty
/// file is created in its place.
///
/// Logging is intentionally **not** performed here; the caller is responsible
/// for logging the returned [`ReplResult`].
///
/// # Errors
///
/// Returns an [`std::io::Error`] if the directory cannot be removed or the
/// replacement file cannot be created.
pub fn replace<P: AsRef<Path>>(path: P) -> io::Result<ReplResult> {
    replace_with_force(path, false)
}

/// Attempts to replace the directory at `path` with an empty file.
///
/// Empty directories are replaced immediately. Non-empty directories only prompt
/// when `force` is false; if `force` is true, the directory is deleted without
/// prompting.
pub fn replace_with_force<P: AsRef<Path>>(path: P, force: bool) -> io::Result<ReplResult> {
    let path_ref = path.as_ref();
    if !path_ref.is_dir() {
        return Ok(ReplResult::NotRequired);
    }

    let is_empty = fs::read_dir(path_ref)
        .map(|mut dir| dir.next().is_none())
        .unwrap_or(false);

    if is_empty {
        fs::remove_dir(path_ref)?;
        File::create(path_ref)?;
        return Ok(ReplResult::Completed);
    }

    if force {
        fs::remove_dir_all(path_ref)?;
        File::create(path_ref)?;
        return Ok(ReplResult::Completed);
    }

    match Action::new(path_ref) {
        Action::Accept => {
            fs::remove_dir_all(path_ref)?;
            File::create(path_ref)?;
            Ok(ReplResult::Completed)
        }
        Action::Abort => {
            eprintln!("Abort");
            Ok(ReplResult::Aborted)
        }
    }
}
