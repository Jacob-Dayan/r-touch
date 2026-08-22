#!/usr/bin/env bash
# set_atime.sh — Update only the access time of a file.
#
# Usage:
#   ./set_atime.sh
#
# Demonstrates:
#   rtouch -a -d <expr> <file>   — set atime to a date expression
#   rtouch -a           <file>   — set atime to now (mtime preserved)
set -euo pipefail

TMPDIR_DEMO=$(mktemp -d)
trap 'rm -rf "$TMPDIR_DEMO"' EXIT
FILE="$TMPDIR_DEMO/report.pdf"

# Seed the file with an old timestamp so the change is visible.
rtouch "$FILE"
touch -m -d "1970-01-01 00:00:00" "$FILE"   # set mtime to epoch via system touch

echo "=== Before: timestamps ==="
stat "$FILE"

echo ""
echo "=== Setting atime to 'yesterday' (mtime preserved) ==="
rtouch -a -d "yesterday" "$FILE"
stat "$FILE"

echo ""
echo "Done."
