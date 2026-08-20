#!/usr/bin/env bash
# set_mtime.sh — Update only the modification time of a file.
#
# Usage:
#   ./set_mtime.sh
#
# Demonstrates:
#   rtouch -m -d <expr> <file>   — set mtime to a date expression
#   rtouch -m           <file>   — set mtime to now (atime preserved)
set -euo pipefail

TMPDIR_DEMO=$(mktemp -d)
trap 'rm -rf "$TMPDIR_DEMO"' EXIT
FILE="$TMPDIR_DEMO/archive.tar.gz"

rtouch "$FILE"

echo "=== Before: timestamps ==="
stat "$FILE"

echo ""
echo "=== Setting mtime to '3 days ago' (atime preserved) ==="
rtouch -m -d "3 days ago" "$FILE"
stat "$FILE"

echo ""
echo "Done."
