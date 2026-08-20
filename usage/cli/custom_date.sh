#!/usr/bin/env bash
# custom_date.sh — Use various date expression formats with -d / --date.
#
# Usage:
#   ./custom_date.sh
#
# Demonstrates:
#   rtouch -d "yesterday"          — relative keyword
#   rtouch -d "2 days ago"        — relative offset
#   rtouch -d "+3 hours"          — future offset
#   rtouch -d "next monday"       — next weekday
#   rtouch -d "2026-08-14 14:30"  — ISO 8601 datetime
#   rtouch -d "202608141430"      — GNU touch format
set -euo pipefail

TMPDIR_DEMO=$(mktemp -d)
trap 'rm -rf "$TMPDIR_DEMO"' EXIT

touch_and_stat() {
    local label="$1"; shift
    rtouch "$@"
    echo "[$label] → $(stat -c '%y / %x' "${@: -1}")"
}

echo "=== Relative keywords ==="
touch_and_stat "yesterday"    -d "yesterday"    "$TMPDIR_DEMO/f1.txt"
touch_and_stat "tomorrow"     -d "tomorrow"     "$TMPDIR_DEMO/f2.txt"

echo ""
echo "=== Relative offsets ==="
touch_and_stat "2 days ago"   -d "2 days ago"   "$TMPDIR_DEMO/f3.txt"
touch_and_stat "+3 hours"     -d "+3 hours"     "$TMPDIR_DEMO/f4.txt"
touch_and_stat "-15 minutes"  -d "-15 minutes"  "$TMPDIR_DEMO/f5.txt"

echo ""
echo "=== Next weekday ==="
touch_and_stat "next monday"  -d "next monday"  "$TMPDIR_DEMO/f6.txt"

echo ""
echo "=== ISO 8601 datetime ==="
touch_and_stat "ISO datetime" -d "2026-08-14 14:30" "$TMPDIR_DEMO/f7.txt"

echo ""
echo "=== GNU touch format (CCYYMMDDhhmm) ==="
touch_and_stat "GNU format"   -d "202608141430"  "$TMPDIR_DEMO/f8.txt"

echo ""
echo "Done."
