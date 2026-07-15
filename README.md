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

Maka sure you have [Rust and Cargo](https://rustup.rs/) installed on your machine.

1. Clone this repository:
   ```bash
   git clone https://github.com/rust-glazer/R-touch.git
   cd R-touch

## Compatibility
Linux 🐧
MacOS 🍎💻
Windows 🪟
