use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    #[serde(default = "default_completions")]
    pub completions: bool,

    #[serde(default = "default_should_log")]
    pub should_log: bool,

    /// custom directory for log files
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_dir: Option<PathBuf>,

    #[serde(default)]
    pub time_modify: TimeModifyConfig,
}

fn default_completions() -> bool {
    true
}

fn default_should_log() -> bool {
    true
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

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            completions: default_completions(),
            should_log: default_should_log(),
            log_dir: None,
            time_modify: TimeModifyConfig::default(),
        }
    }
}
