# Configuration File Examples

This directory contains reference configuration files demonstrating different usage profiles for `rtouch`.

## Configuration Path

`R-touch` looks for its configuration file at the following standard paths:

- **Linux / Unix**: `~/.config/R-touch/config.toml`
- **Windows**: `%APPDATA%\R-touch\config.toml`

To apply one of these configurations, copy the desired file to your system's configuration path:

```bash
# Example for Linux/macOS
mkdir -p ~/.config/R-touch
cp examples/configuration/default.toml ~/.config/R-touch/config.toml
```

## Available Profiles

| File | Description |
| :--- | :--- |
| [`default.toml`](default.toml) | Standard default configuration generated automatically on first startup |
| [`custom_log_dir.toml`](custom_log_dir.toml) | Redirects all audit and crash logs to a custom user directory |
| [`silent_no_log.toml`](silent_no_log.toml) | Disables audit logging to eliminate disk I/O overhead and permission requirements |
| [`sync_timestamps.toml`](sync_timestamps.toml) | Synchronizes both access and modification times when either `-a` or `-m` is specified |
| [`ci_automation.toml`](ci_automation.toml) | Disables interactive prompts and logging for CI/CD runners and automated scripts |

## Schema Reference

```toml
# whether shell completion prompts are enabled on startup (default: true)
completions = true

# whether audit logging is enabled by default (default: true)
should-log = true

# custom directory for audit logs (optional)
log-dir = "/path/to/custom/logs"

[time-modify]
# update access time when modification time is updated with -m (default: false)
atime-on-mtime = false

# update modification time when access time is updated with -a (default: false)
mtime-on-atime = false
```
