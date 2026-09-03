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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
/// Constructed from a decision closure via [`Action::new`] and consumed by [`replace`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// The user confirmed the replacement.
    Accept,
    /// The user declined the replacement.
    Abort,
}

impl From<bool> for Action {
    fn from(value: bool) -> Self {
        if value {
            Self::Accept
        } else {
            Self::Abort
        }
    }
}

impl Action {
    /// Evaluates the decision closure and returns the corresponding [`Action`].
    ///
    /// If the closure returns `true`, returns [`Action::Accept`];
    /// otherwise returns [`Action::Abort`].
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> bool,
    {
        Self::from(f())
    }
}

/// Attempts to replace the directory at `path` with an empty file.
///
/// Prompts the caller via the `confirm` closure when replacement is required.
/// If confirmed, the directory tree is removed with [`fs_err::remove_dir_all`]
/// and an empty file is created in its place.
///
/// Logging is intentionally **not** performed here; the caller is responsible
/// for logging the returned [`ReplResult`].
///
/// # Errors
///
/// Returns an [`std::io::Error`] if the directory cannot be removed or the
/// replacement file cannot be created.
pub fn replace<P, F>(path: P, confirm: F) -> io::Result<ReplResult>
where
    P: AsRef<Path>,
    F: FnOnce() -> bool,
{
    replace_with_force(path, false, confirm)
}

/// Attempts to replace the directory at `path` with an empty file.
///
/// Empty directories are replaced immediately without prompting. Non-empty
/// directories only evaluate `confirm` when `force` is false; if `force` is true,
/// the directory is deleted without prompting.
pub fn replace_with_force<P, F>(path: P, force: bool, confirm: F) -> io::Result<ReplResult>
where
    P: AsRef<Path>,
    F: FnOnce() -> bool,
{
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

    match Action::new(confirm) {
        Action::Accept => {
            fs::remove_dir_all(path_ref)?;
            File::create(path_ref)?;
            Ok(ReplResult::Completed)
        }
        Action::Abort => Ok(ReplResult::Aborted),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_new_accept() {
        let action = Action::new(|| true);
        assert_eq!(action, Action::Accept);
    }

    #[test]
    fn test_action_new_abort() {
        let action = Action::new(|| false);
        assert_eq!(action, Action::Abort);
    }

    #[test]
    fn test_action_from_bool() {
        assert_eq!(Action::from(true), Action::Accept);
        assert_eq!(Action::from(false), Action::Abort);
    }

    #[test]
    fn test_replace_with_force_not_a_directory() {
        let file_path = std::env::temp_dir().join("rtouch_test_not_dir.txt");
        let _ = std::fs::remove_file(&file_path);
        let _ = std::fs::remove_dir_all(&file_path);

        std::fs::File::create(&file_path).unwrap();
        let res = replace_with_force(&file_path, false, || panic!("should not be called")).unwrap();
        assert_eq!(res, ReplResult::NotRequired);

        std::fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_replace_with_force_empty_dir() {
        let dir_path = std::env::temp_dir().join("rtouch_test_empty_dir");
        let _ = std::fs::remove_dir_all(&dir_path);
        let _ = std::fs::remove_file(&dir_path);

        std::fs::create_dir(&dir_path).unwrap();
        let res = replace_with_force(&dir_path, false, || panic!("should not be called")).unwrap();
        assert_eq!(res, ReplResult::Completed);
        assert!(dir_path.is_file());

        std::fs::remove_file(&dir_path).unwrap();
    }

    #[test]
    fn test_replace_with_force_non_empty_force() {
        let dir_path = std::env::temp_dir().join("rtouch_test_force_dir");
        let _ = std::fs::remove_dir_all(&dir_path);
        let _ = std::fs::remove_file(&dir_path);

        std::fs::create_dir(&dir_path).unwrap();
        std::fs::write(dir_path.join("file.txt"), "hello").unwrap();

        let res = replace_with_force(&dir_path, true, || panic!("should not be called")).unwrap();
        assert_eq!(res, ReplResult::Completed);
        assert!(dir_path.is_file());

        std::fs::remove_file(&dir_path).unwrap();
    }

    #[test]
    fn test_replace_with_force_non_empty_confirm_accept() {
        let dir_path = std::env::temp_dir().join("rtouch_test_confirm_accept_dir");
        let _ = std::fs::remove_dir_all(&dir_path);
        let _ = std::fs::remove_file(&dir_path);

        std::fs::create_dir(&dir_path).unwrap();
        std::fs::write(dir_path.join("file.txt"), "hello").unwrap();

        let mut called = false;
        let res = replace_with_force(&dir_path, false, || {
            called = true;
            true
        })
        .unwrap();

        assert!(called);
        assert_eq!(res, ReplResult::Completed);
        assert!(dir_path.is_file());

        std::fs::remove_file(&dir_path).unwrap();
    }

    #[test]
    fn test_replace_with_force_non_empty_confirm_abort() {
        let dir_path = std::env::temp_dir().join("rtouch_test_confirm_abort_dir");
        let _ = std::fs::remove_dir_all(&dir_path);
        let _ = std::fs::remove_file(&dir_path);

        std::fs::create_dir(&dir_path).unwrap();
        std::fs::write(dir_path.join("file.txt"), "hello").unwrap();

        let mut called = false;
        let res = replace_with_force(&dir_path, false, || {
            called = true;
            false
        })
        .unwrap();

        assert!(called);
        assert_eq!(res, ReplResult::Aborted);
        assert!(dir_path.is_dir());

        std::fs::remove_dir_all(&dir_path).unwrap();
    }
}
