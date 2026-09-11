// R-touch Library
// Copyright (c) 2026 Jacob Dayan
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 or the MIT License,
// at your option. You may not use this file except in compliance with
// one of these licenses.

use serde::{Deserialize, Serialize};
use std::{
    fmt, io,
    path::{Path, PathBuf},
    str::FromStr,
};

const fn default_true() -> bool {
    true
}

/// main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    /// whether shell completions are enabled or suggested on startup
    #[serde(default = "default_true")]
    pub completions: bool,

    /// whether logging to audit files is enabled by default
    #[serde(default = "default_true")]
    pub should_log: bool,

    /// custom directory for log files
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_dir: Option<PathBuf>,

    /// time modification behavior settings ([`TimeModifyConfig`])
    #[serde(default)]
    pub time_modify: TimeModifyConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            completions: default_true(),
            should_log: default_true(),
            log_dir: None,
            time_modify: TimeModifyConfig::default(),
        }
    }
}

impl AppConfig {
    /// load [`AppConfig`] from default configuration path if it exists
    ///
    /// # Errors
    ///
    /// returns an [`io::Error`] if configuration file exists but cannot be read or parsed
    pub fn load_default() -> io::Result<Option<Self>> {
        crate::conf::io::load_default()
    }

    /// load [`AppConfig`] for a specific application name if it exists
    ///
    /// # Errors
    ///
    /// returns an [`io::Error`] if configuration file exists but cannot be read or parsed
    pub fn load_default_for(app_name: impl AsRef<str>) -> io::Result<Option<Self>> {
        crate::conf::io::load_default_for(app_name)
    }

    /// read and parse [`AppConfig`] from a file path
    ///
    /// # Errors
    ///
    /// returns an [`io::Error`] if file cannot be read or parsed
    pub fn load_from<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        crate::conf::io::load_from(path)
    }

    /// serialize and save [`AppConfig`] to default configuration path
    ///
    /// # Errors
    ///
    /// returns an [`io::Error`] if default path cannot be resolved or file cannot be written
    pub fn save_default(&self) -> io::Result<()> {
        crate::conf::io::save_default(self)
    }

    /// serialize and save [`AppConfig`] for a specific application name
    ///
    /// # Errors
    ///
    /// returns an [`io::Error`] if configuration path cannot be determined,
    /// parent directory cannot be created, or file cannot be written
    pub fn save_default_for(&self, app_name: impl AsRef<str>) -> io::Result<()> {
        crate::conf::io::save_default_for(self, app_name)
    }

    /// serialize and save [`AppConfig`] to specified file path
    ///
    /// # Errors
    ///
    /// returns an [`io::Error`] if parent directory cannot be created or file cannot be written
    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        crate::conf::io::save_to(self, path)
    }
}

impl FromStr for AppConfig {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::conf::parse::parse(s)
    }
}

impl fmt::Display for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let serialized = crate::conf::parse::serialize(self).map_err(|_| fmt::Error)?;
        write!(f, "{serialized}")
    }
}

/// time modification behavior settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct TimeModifyConfig {
    /// update access time when updating modification time (`-m`)
    #[serde(default)]
    pub atime_on_mtime: bool,

    /// update modification time when updating access time (`-a`)
    #[serde(default)]
    pub mtime_on_atime: bool,
}

impl TimeModifyConfig {
    /// resolve effective atime and mtime flags based on configuration rules
    pub fn resolve_flags(&self, mut atime: bool, mut mtime: bool) -> (bool, bool) {
        if atime && !mtime && self.mtime_on_atime {
            mtime = true;
        }
        if mtime && !atime && self.atime_on_mtime {
            atime = true;
        }
        (atime, mtime)
    }
}
