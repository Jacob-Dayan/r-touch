// R-touch Library
// Copyright (c) 2026 Jacob Dayan
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 or the MIT License,
// at your option. You may not use this file except in compliance with
// one of these licenses.

use crate::conf::model::AppConfig;

/// parse a TOML string into an [`AppConfig`]
///
/// # Errors
///
/// returns a [`toml::de::Error`] if string cannot be parsed into an [`AppConfig`]
pub fn parse(s: &str) -> Result<AppConfig, toml::de::Error> {
    toml::from_str(s)
}

/// serialize an [`AppConfig`] into a TOML string
///
/// # Errors
///
/// returns a [`toml::ser::Error`] if configuration cannot be serialized
pub fn serialize(config: &AppConfig) -> Result<String, toml::ser::Error> {
    toml::to_string_pretty(config)
}
