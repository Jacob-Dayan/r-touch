#!/usr/bin/env bash
set -euo pipefail

sudo -v || { echo "Could not complete build: insufficient permissions." >&2; exit 1; }

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${REPO_ROOT}"

cargo build --release
sudo install -m 755 ./target/release/rtouch /usr/local/bin/rtouch

/usr/local/bin/rtouch -V
