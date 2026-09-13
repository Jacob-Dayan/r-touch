# Scripts and Automation Directory

This directory contains automation, build, configuration, and testing scripts for the project:

- `build/build-unix.sh`: Builds the release binary and installs it to `/usr/local/bin` on Unix/Linux/macOS systems.
- `build/build-user.ps1`: Builds the release binary and installs it to the current user's local application directory on Windows.
- `build/build-system.ps1`: Builds the release binary with administrator privileges and installs it to Program Files for all users on Windows.
- `grant-logging-permissions.sh`: Initializes the `/var/log/R-touch` audit log directory and assigns ownership to the current user.
- `test_all_examples.sh`: Comprehensive test runner that validates all library examples (`examples/lib/`) and CLI scenarios (`examples/cli/`).
