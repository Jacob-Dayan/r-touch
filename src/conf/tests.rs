// R-touch Library
// Copyright (c) 2026 Jacob Dayan
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 or the MIT License,
// at your option. You may not use this file except in compliance with
// one of these licenses.

use super::*;
use fs_err as fs;
use std::path::PathBuf;

#[test]
fn test_default_config_values() {
    let cfg = AppConfig::default();
    assert!(cfg.completions);
    assert!(cfg.should_log);
    assert_eq!(cfg.log_dir, None);
    assert!(!cfg.time_modify.atime_on_mtime);
    assert!(!cfg.time_modify.mtime_on_atime);
}

#[test]
fn test_deserialize_custom_toml() {
    let toml_str = r#"
completions = false
should-log = true
log-dir = "/tmp/custom_logs"

[time-modify]
atime-on-mtime = true
mtime-on-atime = false
"#;
    let cfg = parse(toml_str).unwrap();
    assert!(!cfg.completions);
    assert!(cfg.should_log);
    assert_eq!(cfg.log_dir, Some(PathBuf::from("/tmp/custom_logs")));
    assert!(cfg.time_modify.atime_on_mtime);
    assert!(!cfg.time_modify.mtime_on_atime);

    let serialized = serialize(&cfg).unwrap();
    assert!(serialized.contains("completions = false"));
    assert!(serialized.contains("should-log = true"));
    assert!(serialized.contains(r#"log-dir = "/tmp/custom_logs""#));
    assert!(serialized.contains("atime-on-mtime = true"));
    assert!(serialized.contains("mtime-on-atime = false"));
}

#[test]
fn test_deserialize_empty_toml_uses_defaults() {
    let cfg = parse("").unwrap();
    assert_eq!(cfg, AppConfig::default());
}

#[test]
fn test_deserialize_partial_toml_only_time_modify() {
    let toml_str = r#"
[time-modify]
mtime-on-atime = true
"#;
    let cfg = parse(toml_str).unwrap();
    assert!(cfg.completions);
    assert!(cfg.should_log);
    assert_eq!(cfg.log_dir, None);
    assert!(!cfg.time_modify.atime_on_mtime);
    assert!(cfg.time_modify.mtime_on_atime);
}

#[test]
fn test_deserialize_partial_toml_only_log_dir() {
    let toml_str = r#"log-dir = "/var/log/my_app""#;
    let cfg = parse(toml_str).unwrap();
    assert!(cfg.completions);
    assert!(cfg.should_log);
    assert_eq!(cfg.log_dir, Some(PathBuf::from("/var/log/my_app")));
    assert!(!cfg.time_modify.atime_on_mtime);
    assert!(!cfg.time_modify.mtime_on_atime);
}

#[test]
fn test_deserialize_explicit_false_values() {
    let toml_str = r#"
completions = false
should-log = false
"#;
    let cfg = parse(toml_str).unwrap();
    assert!(!cfg.completions);
    assert!(!cfg.should_log);
    assert_eq!(cfg.log_dir, None);
}

#[test]
fn test_parse_invalid_toml_syntax_returns_error() {
    let bad_toml = "completions = [unclosed bracket";
    assert!(parse(bad_toml).is_err());
}

#[test]
fn test_load_from_nonexistent_file_returns_error() {
    let path = PathBuf::from("/path/that/definitely/does/not/exist/conf.toml");
    assert!(load_from(&path).is_err());
}

#[test]
fn test_load_from_invalid_content_returns_error() {
    let temp_dir = std::env::temp_dir().join("rtouch_test_conf_invalid");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();
    let path = temp_dir.join("config.toml");
    fs::write(&path, b"not valid = toml == syntax").unwrap();

    assert!(load_from(&path).is_err());
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_save_and_load_roundtrip() {
    let temp_dir = std::env::temp_dir().join("rtouch_test_conf_roundtrip");
    let _ = fs::remove_dir_all(&temp_dir);
    let path = temp_dir.join("config.toml");

    let original = AppConfig {
        completions: false,
        should_log: true,
        log_dir: Some(PathBuf::from("/var/log/custom")),
        time_modify: TimeModifyConfig {
            atime_on_mtime: true,
            mtime_on_atime: true,
        },
    };

    save_to(&original, &path).unwrap();
    assert!(path.exists());

    let loaded = load_from(&path).unwrap();
    assert_eq!(original, loaded);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_time_modify_resolve_flags() {
    let mut tm = TimeModifyConfig::default();
    assert_eq!(tm.resolve_flags(true, false), (true, false));
    assert_eq!(tm.resolve_flags(false, true), (false, true));
    assert_eq!(tm.resolve_flags(true, true), (true, true));
    assert_eq!(tm.resolve_flags(false, false), (false, false));

    tm.mtime_on_atime = true;
    assert_eq!(tm.resolve_flags(true, false), (true, true));
    assert_eq!(tm.resolve_flags(false, true), (false, true));

    tm.mtime_on_atime = false;
    tm.atime_on_mtime = true;
    assert_eq!(tm.resolve_flags(false, true), (true, true));
    assert_eq!(tm.resolve_flags(true, false), (true, false));
}

#[test]
fn test_app_config_from_str_and_display() {
    use std::str::FromStr;

    let toml_str = "completions = false\nshould-log = true\n";
    let cfg = AppConfig::from_str(toml_str).unwrap();
    assert!(!cfg.completions);
    assert!(cfg.should_log);

    let display_str = cfg.to_string();
    assert!(display_str.contains("completions = false"));
    assert!(display_str.contains("should-log = true"));
}

#[test]
fn test_app_config_associated_methods() {
    let temp_dir = std::env::temp_dir().join("rtouch_test_conf_assoc");
    let _ = fs::remove_dir_all(&temp_dir);
    let path = temp_dir.join("assoc_config.toml");

    let original = AppConfig {
        completions: true,
        should_log: false,
        log_dir: Some(PathBuf::from("/var/log/assoc")),
        time_modify: TimeModifyConfig::default(),
    };

    original.save_to(&path).unwrap();
    assert!(path.exists());

    let loaded = AppConfig::load_from(&path).unwrap();
    assert_eq!(original, loaded);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_usage_configuration_examples_parse_valid() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_examples_dir = manifest_dir.join("usage").join("configuration");
    assert!(config_examples_dir.is_dir());

    let entries = fs::read_dir(&config_examples_dir).unwrap();
    let mut toml_count = 0;

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            let content = fs::read_to_string(&path).unwrap();
            let parsed = parse(&content);
            assert!(
                parsed.is_ok(),
                "Failed to parse example config {}: {:?}",
                path.display(),
                parsed.err()
            );
            toml_count += 1;
        }
    }

    assert!(toml_count >= 5);
}

#[test]
fn test_app_name_and_config_path_for() {
    assert_eq!(APP_NAME, "R-touch");
    let default_path = default_config_path();
    let app_path = config_path_for(APP_NAME);
    assert_eq!(default_path, app_path);

    let custom_path = config_path_for("other_app").unwrap();
    assert!(custom_path.ends_with(PathBuf::from("other_app").join("config.toml")));
}


