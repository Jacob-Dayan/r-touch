# R-touch 🦀

A fast, modern, and slightly opinionated recreation of the classic Unix `touch` command, built from scratch in Rust.

Unlike the classic `touch` that silently fails or acts weirdly when encountering directories, `R-touch` actually talks to you, manages its own system logs safely, and ensures you don't accidentally trash your system layout.

> "Why did the developer use R-touch? Because standard touch was giving them some boundary issues." *(Sorry, we promised only semi-decent jokes).*

---

## Features

* **GNU Touch Parity (`-a`, `-m`, `-d`):** Selectively update access time (`-a`), modification time (`-m`), or both. Set custom timestamps using `-d` / `--date` with support for standard dates, GNU touch format, and human-readable relative expressions.
* **Flexible Date & Time Interpreter:** Supports ISO 8601, RFC 2822/3339, GNU touch syntax (`[[CC]YY]MMDDhhmm[.ss]`), and human expressions like `now`, `yesterday`, `tomorrow`, `2 days ago`, `+3 hours`, `-15 minutes`, `next tuesday`, and `today 14:30` — with clear and helpful error diagnostics if an invalid date is provided.
* **Smart Directory Handling:** If you try to create a file where a directory already exists, `R-touch` stops and asks you what to do instead of blowing up.
* **Parent Directory Creation (`-p` / `--parents`):** Need to touch `deep/nested/folder/file.txt`? Use `-p` or `--parents` and let `R-touch` build the directory tree for you.
* **Automatic Logging:** Logs successes and errors into your local OS data directory (`~/.local/share/rtouch` on Linux or `%LocalAppData%\rtouch` on Windows) so you always have an audit trail. Can be disabled with `--no-log`.
* **Platform-Friendly:** Built-in Windows path separator normalization (because backslashes shouldn't be your problem).

---

## CLI Options & Usage

```text
Usage: rtouch [OPTIONS] <PATHS>...

Arguments:
  <PATHS>...  File paths to touch or create

Options:
  -p, --parents                        Create parent directories if they do not exist
  -a, --atime, --access-time          Change only the access time
  -m, --mtime, --modification-time    Change only the modification time
  -d, --date <DATE>                    Parse date string expression and use it instead of current time
      --no-log                         Disable logging to log files
  -h, --help                           Print help
  -V, --version                        Print version
```

### Examples

#### 1. Basic File Touch / Creation
```bash
# Create a new file or update timestamps of an existing file
rtouch file.txt

# Touch multiple files at once
rtouch file1.txt file2.txt file3.rs
```

#### 2. Create Parent Directories (`-p`, `--parents`)
```bash
# Automatically create missing parent directories
rtouch -p src/components/button/index.tsx
```

#### 3. Change Only Access Time (`-a`, `--atime`, `--access-time`)
```bash
# Updates only atime; preserves mtime on existing files
rtouch -a document.pdf
```

#### 4. Change Only Modification Time (`-m`, `--mtime`, `--modification-time`)
```bash
# Updates only mtime; preserves atime on existing files
rtouch -m document.pdf
```

#### 5. Custom Timestamps with `-d` / `--date`
`R-touch` supports rich date/time expressions:

* **Relative Time Expressions:**
  ```bash
  rtouch -d "yesterday" file.txt
  rtouch -d "2 days ago" file.txt
  rtouch -d "tomorrow" file.txt
  rtouch -d "+3 hours" file.txt
  rtouch -d "-30 minutes" file.txt
  rtouch -d "next friday" file.txt
  rtouch -d "last month" file.txt
  rtouch -d "today 14:30" file.txt
  ```

* **Standard ISO / RFC Formats:**
  ```bash
  rtouch -d "2026-08-19 14:30:00" file.txt
  rtouch -d "2026-08-19T14:30:00Z" file.txt
  rtouch -d "2026-08-19" file.txt
  ```

* **GNU Touch Timestamp Syntax (`[[CC]YY]MMDDhhmm[.ss]`):**
  ```bash
  rtouch -d "202608191430.00" file.txt
  rtouch -d "2608191430" file.txt
  rtouch -d "08191430" file.txt
  ```

#### 6. Combining Flags
```bash
# Set only access time to yesterday
rtouch -a -d "yesterday" report.docx

# Set only modification time to a specific date
rtouch -m -d "2026-01-01 00:00:00" archive.tar.gz

# Create with parent directories, custom date, and disable logging
rtouch -p --no-log -d "2 days ago" logs/2026/08/old.log
```

---

## Installation

Make sure you have [Rust and Cargo](https://rustup.rs/) installed on your machine.

### Installing from crates.io (one command)
```bash
cargo install rtouch
```

### Building from source
1. Clone this repository:
   ```bash
   git clone https://github.com/Jacob-Dayan/r-touch.git
   cd r-touch
   ```

#### Unix/Linux
If you are on _Unix_ or _Unix-like_ (e.g. Linux, macOS) OS:
```bash
chmod +x ./build/build_unix.sh
./build/build-unix.sh
```

#### Windows
If you are on Windows (user-level installation):
```powershell
./build/build-user.ps1
```
And for machine-level Windows installation (makes the executable available to all users on the machine, recommended):
```powershell
./build/build-system.ps1
```

---

## Benchmarking

You can find the benchmarking results and get more information about the benchmarking process in the [benchmarks directory](https://github.com/Jacob-Dayan/r-touch/tree/main/Benchmarks).

---

## Compatibility

- Linux 🐧
- macOS 🍎💻
- Windows 🪟
- Windows Subsystem For Linux (I don't have an emoji for that)

---

### Note on Release Timestamps & Tags

> **Notice:** Due to a tag migration script refactoring (`R-touch-*` to `v*`), GitHub Release publication dates were reset. The underlying code history, original commit dates, and Git tags remain fully preserved in the repository tree.
