# R-touch 🦀

A fast, modern, and slightly opinionated recreation of the classic Unix `touch` command, built from scratch in Rust.

Unlike the classic `touch` that silently fails or acts weirdly when encountering directories, `R-touch` actually talks to you, manages its own system logs safely, and ensures you don't accidentally trash your system layout.

> "Why did the developer use R-touch? Because standard touch was giving them some boundary issues." *(Sorry, we promised only semi-decent jokes).*

---

## Features

* **Smart Directory Handling:** If you try to create a file where a directory already exists, `R-touch` stops and asks you what to do instead of blowing up.
* **Parent Directory Creation:** Need to touch `deep/nested/folder/file.txt`? Use `-p` or `--parents` and let us build the path for you.
* **Automatic Logging:** Logs successes and errors into your local OS data directory (`~/.local/share` on Linux or `AppData` on Windows) so you always have an audit trail.
* **Platform-Friendly:** Built-in Windows path separator normalization (because backslashes shouldn't be your problem).

---

## Installation

Make sure you have [Rust and Cargo](https://rustup.rs/) installed on your machine.

1. Clone this repository:
   ```bash
   git clone https://github.com/Jacob-Dayan/R-touch.git
   cd R-touch
   ```
---

## Building from source

---

### Unix/Linux
   if you are on _Unix_ or _Unix-Like_(e.g, Linux) OS:
   ```bash
         git clone https://github.com/Jacob-Dayan/r-touch.git
         cd r-touch
         ./build/build-unix.sh
   ```

### Windows
   if you're in Windows(10+, user-level installation):
   ```powershell
       git clone https://github.com/Jacob-Dayan/r-touch.git
       cd r-touch
       ./build/build-user.ps1
   ```
   and for machin-level Windows installation (makes the executable output file available to all the users on the machin, recommended):
   ```powershell
      git clone https://github.com/Jacob-Dayan/r-touch
      cd r-touch
      ./build/build-system.ps1      
   ```

---

## Benchmarking
---
You can find the benchmarking results and get more information about the benchmarking process in the [benchmarks directory](https://github.com/Jacob-Dayan/r-touch/tree/main/Benchmarks).


## Compatibility
Linux 🐧

MacOS 🍎💻

Windows 🪟

Windows Subsystem For Linux (I don't have an emoji for that)



---
### Note on Release Timestamps & Tags
---
> **Notice:** Due to a tag migration script refactoring (`R-touch-*` to `v*`), GitHub Release publication dates were reset. The underlying code history, original commit dates, and Git tags remain fully preserved in the repository tree.
