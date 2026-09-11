// R-touch Library
// Copyright (c) 2026 Jacob Dayan
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 or the MIT License,
// at your option. You may not use this file except in compliance with
// one of these licenses.

//! configuration management and persistence for `rtouch`

pub mod io;
pub mod model;
pub mod parse;

pub use io::{
    config_path_for, default_config_path, load_default, load_default_for, load_from, save_default,
    save_default_for, save_to, APP_NAME,
};
pub use model::{AppConfig, TimeModifyConfig};
pub use parse::{parse, serialize};

#[cfg(test)]
mod tests;
