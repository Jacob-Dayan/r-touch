#!/usr/bin/env bash
# basic_touch.sh — Create a file or update its timestamps.
#
# Usage:
#   ./basic_touch.sh
#
# Demonstrates:
#   rtouch <file>            — create / update a single file
#   rtouch <f1> <f2> <f3>   — touch multiple files in one command
set -euo pipefail

TMPDIR_DEMO=$(mktemp -d)
trap 'rm -rf "$TMPDIR_DEMO"' EXIT

echo "=== Basic single-file touch ==="
rtouch "$TMPDIR_DEMO/hello.txt"
echo "Created: $TMPDIR_DEMO/hello.txt"
ls -l "$TMPDIR_DEMO/hello.txt"

echo ""
echo "=== Touch multiple files at once ==="
rtouch "$TMPDIR_DEMO/a.txt" "$TMPDIR_DEMO/b.txt" "$TMPDIR_DEMO/c.rs"
echo "Created: a.txt, b.txt, c.rs"
ls -l "$TMPDIR_DEMO/"

echo ""
echo "Done."
