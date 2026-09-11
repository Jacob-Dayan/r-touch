# Quick Start Guide for R-touch 🦀

Welcome to `R-touch`! This guide gets you up and running in minutes with common commands, configuration tips, and logging options.

---

## 1. Installation & First Run

### Building from Source
Ensure Rust 2024 edition is installed (`rustup update`), then:

```bash
git clone https://github.com/Jacob-Dayan/r-touch.git
cd r-touch
cargo build --release
```

The compiled binary will be located at `target/release/rtouch` (or `target/release/rtouch.exe` on Windows).

### Automated Build Scripts
- **Unix / Linux / macOS**:
  ```bash
  chmod +x ./build/build-unix.sh
  ./build/build-unix.sh
  ```
- **Windows (User-level)**:
  ```powershell
  .\build\build-user.ps1
  ```
- **Windows (System-wide)**:
  ```powershell
  .\build\build-system.ps1
  ```

### Shell Completions
On your first run, `R-touch` will ask if you want shell completions installed. Hit `Enter` to confirm. You can also install or re-install completions anytime:

```bash
# Auto-detect shell and install into standard user completion directory
rtouch --install-completion

# Explicit shell target (bash, zsh, fish, powershell, pwsh, elvish)
rtouch --completion zsh
rtouch --completion fish
rtouch --completion pwsh
```

---

## 2. Common Usage Examples

### Create Files & Touch Timestamps
```bash
# Create a new empty file or update timestamps of an existing file
rtouch hello.txt

# Touch multiple files at once
rtouch file1.txt file2.txt file3.rs
```

### Create Parent Directories (`-p` / `--parents`)
Equivalent to `mkdir -p` followed by file creation:
```bash
rtouch -p src/components/button/index.tsx
```

### Update Only Access Time (`-a`) or Modification Time (`-m`)
```bash
# Update only atime (preserves mtime on existing files)
rtouch -a document.pdf

# Update only mtime (preserves atime on existing files)
rtouch -m document.pdf
```

### Custom Date & Relative Time Expressions (`-d` / `--date`)
`R-touch` includes an expressive human-readable date parser:

```bash
# Relative keywords & offsets
rtouch -d "yesterday" report.txt
rtouch -d "yesterday 14:30" report.txt
rtouch -d "2 days ago" report.txt
rtouch -d "tomorrow 09:00" reminder.txt
rtouch -d "+3 hours" file.txt
rtouch -d "-30 minutes" file.txt
rtouch -d "next monday" file.txt
rtouch -d "today 16:45" file.txt

# Standard ISO 8601 & GNU touch formats
rtouch -d "2026-08-14T14:30:00Z" file.txt
rtouch -d "2026-08-14 14:30:00" file.txt
rtouch -d "202608141430" file.txt
```

### Chaining Options & Compact Flags
```bash
# Combine flags and attach date values directly
rtouch -amd "yesterday" file.txt
rtouch -dyesterday file.txt
rtouch -pad"2 days ago" deep/nested/dir/log.txt
```

### Safe Directory Replacement (`-r` / `-f`)
If a target path is an existing directory:
```bash
# Prompts for confirmation before deleting directory and creating empty file
rtouch -r existing_folder

# Force deletion without prompting (even if directory is not empty)
rtouch -rf existing_folder
```

---

## 3. Logging System

`R-touch` automatically records an audit log of created files, timestamp modifications, and errors:

### Default Log Locations
- **Linux / Unix**: `/var/log/R-touch/`
  - Success log: `/var/log/R-touch/r-touch.log`
  - Crashes & errors: `/var/log/R-touch/crashes/file_creations.log`
  - Access time updates: `/var/log/R-touch/time_modifications/atime_modification.log`
  - Modification time updates: `/var/log/R-touch/time_modifications/mtime_modification.log`
- **Windows**: `%LOCALAPPDATA%\R-touch\logs\`

> [!NOTE]
> On Unix systems, `/var/log` requires root privileges to create new top-level directories. To initialize user-level access, run:
> ```bash
> sudo mkdir -p /var/log/R-touch && sudo chown -R $USER:$USER /var/log/R-touch
> ```

### Custom Log Directories
You can customize the log directory with any of the following methods (in order of priority):
1. **CLI Flag**: `rtouch --log-dir /path/to/my/logs file.txt`
2. **Config file**: `log-dir = "/path/to/my/logs"` in `config.toml`
3. **Environment variable**: `export R_TOUCH_LOG_DIR="/path/to/my/logs"` (or `RTOUCH_LOG_DIR`)

### Controlling Logging
- Disable logging for an invocation: `rtouch --no-log file.txt`
- Force logging (overriding `config.toml`): `rtouch --log file.txt`

---

## 4. Configuration File (`config.toml`)

`R-touch` stores its configuration in:
- **Unix**: `~/.config/R-touch/config.toml`
- **Windows**: `%APPDATA%\R-touch\config.toml`

### Example `config.toml`
```toml
# Enable or disable automated shell completion prompt
completions = true

# Default logging behavior (override with --log or --no-log)
should-log = true

# Optional custom log directory
# log-dir = "/home/user/logs/R-touch"

[time-modify]
# If true, updating access time (-a) will also update modification time
mtime-on-atime = false

# If true, updating modification time (-m) will also update access time
atime-on-mtime = false
```

---

## 5. Running the Test Suite

Both Bash and PowerShell test suites are provided to validate all usage examples and CLI commands end-to-end:

### Unix / Linux / macOS (Bash)
```bash
./test_all_examples.sh
```

### Windows (PowerShell)
```powershell
pwsh -ExecutionPolicy Bypass -File .\test_all_examples.ps1
```
