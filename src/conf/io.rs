// R-touch Library
// Copyright (c) 2026 Jacob Dayan
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 or the MIT License,
// at your option. You may not use this file except in compliance with
// one of these licenses.

use crate::conf::{model::AppConfig, parse};
use fs_err as fs;
use std::{
    io,
    path::{Path, PathBuf},
};

/// application name used for default configuration directory resolution
pub const APP_NAME: &str = "R-touch";

/// resolve config path for a specific application name (`<config_dir>/<app_name>/config.toml`)
pub fn config_path_for(app_name: impl AsRef<str>) -> Option<PathBuf> {
    dirs_next::config_dir().map(|home_dir| home_dir.join(app_name.as_ref()).join("config.toml"))
}

/// resolve default config path (`<config_dir>/<APP_NAME>/config.toml`)
pub fn default_config_path() -> Option<PathBuf> {
    dirs_next::config_dir().map(|home_dir| home_dir.join(APP_NAME).join("config.toml"))
}

/// load [`AppConfig`] for a specific application name if it exists
///
/// # Errors
///
/// returns an [`io::Error`] if configuration file exists but cannot be read or parsed
pub fn load_default_for(app_name: impl AsRef<str>) -> io::Result<Option<AppConfig>> {
    let Some(path) = config_path_for(app_name) else {
        return Ok(None);
    };
    if !path.exists() {
        return Ok(None);
    }
    load_from(&path).map(Some)
}

/// load [`AppConfig`] from default path if it exists
///
/// # Errors
///
/// returns an [`io::Error`] if default configuration file exists but cannot be read or parsed
pub fn load_default() -> io::Result<Option<AppConfig>> {
    load_default_for(APP_NAME)
}

/// read and parse [`AppConfig`] from a file
///
/// # Errors
///
/// returns an [`io::Error`] if file cannot be read or parsed
pub fn load_from<P: AsRef<Path>>(path: P) -> io::Result<AppConfig> {
    let content = fs::read_to_string(path.as_ref())?;
    parse::parse(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// serialize and write [`AppConfig`] to a file
///
/// # Errors
///
/// returns an [`io::Error`] if parent directory cannot be created or file cannot be written
pub fn save_to<P: AsRef<Path>>(config: &AppConfig, path: P) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content =
        parse::serialize(config).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, content)
}

/// save [`AppConfig`] for a specific application name
///
/// # Errors
///
/// returns an [`io::Error`] if default config path cannot be determined,
/// parent directory cannot be created, or file cannot be written
pub fn save_default_for(config: &AppConfig, app_name: impl AsRef<str>) -> io::Result<()> {
    let Some(path) = config_path_for(app_name) else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine default configuration directory",
        ));
    };
    save_to(config, &path)
}

/// save [`AppConfig`] to default path
///
/// # Errors
///
/// returns an [`io::Error`] if default config path cannot be determined,
/// parent directory cannot be created, or file cannot be written
pub fn save_default(config: &AppConfig) -> io::Result<()> {
    save_default_for(config, APP_NAME)
}
