#!/usr/bin/env bash
# create_with_parents.sh — Touch a deeply nested file, creating parents.
#
# Usage:
#   ./create_with_parents.sh
#
# Demonstrates:
#   rtouch -p <path>   — create parent directories automatically
set -euo pipefail

TMPDIR_DEMO=$(mktemp -d)
trap 'rm -rf "$TMPDIR_DEMO"' EXIT

NESTED="$TMPDIR_DEMO/src/components/button/index.tsx"

echo "=== Creating nested file (with -p) ==="
rtouch -p "$NESTED"
echo "Created: $NESTED"
ls -lR "$TMPDIR_DEMO/src"

echo ""
echo "Done."
