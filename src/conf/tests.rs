use super::*;
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
    let _ = fs_err::remove_dir_all(&temp_dir);
    fs_err::create_dir_all(&temp_dir).unwrap();
    let path = temp_dir.join("config.toml");
    fs_err::write(&path, b"not valid = toml == syntax").unwrap();

    assert!(load_from(&path).is_err());
    let _ = fs_err::remove_dir_all(&temp_dir);
}

#[test]
fn test_save_and_load_roundtrip() {
    let temp_dir = std::env::temp_dir().join("rtouch_test_conf_roundtrip");
    let _ = fs_err::remove_dir_all(&temp_dir);
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

    let _ = fs_err::remove_dir_all(&temp_dir);
}
