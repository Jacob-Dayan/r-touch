#!/usr/bin/env bash
# no_log.sh — Run rtouch with logging disabled.
#
# Usage:
#   ./no_log.sh
#
# Demonstrates:
#   rtouch --no-log <file>   — suppress writing to the log files
#
# Useful in scripts or CI environments where you don't want rtouch
# to accumulate log entries in the local data directory.
set -euo pipefail

TMPDIR_DEMO=$(mktemp -d)
trap 'rm -rf "$TMPDIR_DEMO"' EXIT

echo "=== Touch without logging ==="
rtouch --no-log "$TMPDIR_DEMO/silent.txt"
echo "Created (no log written): $TMPDIR_DEMO/silent.txt"
ls -l "$TMPDIR_DEMO/silent.txt"

echo ""
echo "=== Combine --no-log with other flags ==="
rtouch --no-log -p -d "yesterday" "$TMPDIR_DEMO/deep/nested/old.txt"
echo "Created nested file with yesterday's timestamp, logging suppressed."
ls -l "$TMPDIR_DEMO/deep/nested/old.txt"

echo ""
echo "Done."
