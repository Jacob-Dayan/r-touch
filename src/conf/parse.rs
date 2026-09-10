use crate::conf::model::AppConfig;

/// parse a TOML string into an [`AppConfig`]
pub fn parse(s: &str) -> Result<AppConfig, toml::de::Error> {
    toml::from_str(s)
}

/// serialize an [`AppConfig`] into a TOML string
pub fn serialize(config: &AppConfig) -> Result<String, toml::ser::Error> {
    toml::to_string_pretty(config)
}
