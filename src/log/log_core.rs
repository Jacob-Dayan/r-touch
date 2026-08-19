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
    path::Path,
    time::SystemTime,
};

pub struct LogCore;

impl LogCore {
    // Append log entry to file
    pub fn log<P: AsRef<Path>>(file_path: P, message: &fmt::Arguments) -> Result<()> {
        let path = file_path.as_ref();

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
