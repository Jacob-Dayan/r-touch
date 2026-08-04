#!/usr/bin/env bash

cd "$(dirname "$0")/.." || exit 1

cargo build --release
sudo cp ./target/release/rtouch /usr/local/bin/
rtouch -V
