use crate::conf::{model::AppConfig, parse};
use std::{
    io,
    path::{Path, PathBuf},
};

/// resolve default config path (`<config_dir>/rtouch/config.toml`)
pub fn default_config_path() -> Option<PathBuf> {
    dirs_next::config_dir().map(|dir| dir.join("rtouch").join("config.toml"))
}

/// load [`AppConfig`] from default path if it exists
pub fn load_default() -> io::Result<Option<AppConfig>> {
    let Some(path) = default_config_path() else {
        return Ok(None);
    };
    if !path.exists() {
        return Ok(None);
    }
    load_from(&path).map(Some)
}

/// read and parse [`AppConfig`] from a file
pub fn load_from(path: &Path) -> io::Result<AppConfig> {
    let content = fs_err::read_to_string(path)?;
    parse::parse(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// serialize and write [`AppConfig`] to a file
pub fn save_to(config: &AppConfig, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs_err::create_dir_all(parent)?;
    }
    let content = parse::serialize(config)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs_err::write(path, content)
}

/// save [`AppConfig`] to default path
pub fn save_default(config: &AppConfig) -> io::Result<()> {
    if let Some(path) = default_config_path() {
        save_to(config, &path)?;
    }
    Ok(())
}
