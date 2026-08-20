#!/usr/bin/env bash
sudo -v || { echo "Could not complete build: not enough permissions."; exit 1; }
cd "$(dirname "$0")/.." || exit 1

cargo build --release
sudo cp ./target/release/rtouch /usr/local/bin/
rtouch -V
