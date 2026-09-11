//! demo program parsing TOML configuration

fn main() {
    let toml = "completions = true\nshould-log = false";
    let cfg = rtouch::conf::parse(toml).unwrap();
    println!("completions: {}, should-log: {}", cfg.completions, cfg.should_log);
}

#[cfg(test)]
mod tests {
    /// parse simple TOML string into config
    #[test]
    fn parse_valid_toml() {
        let cfg = rtouch::conf::parse("should-log = false").unwrap();
        assert!(!cfg.should_log);
    }
}
