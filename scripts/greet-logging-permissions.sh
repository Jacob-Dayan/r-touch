#!/usr/bin/env bash
set -euo pipefail

LOG_DIR="/var/log/R-touch"
TARGET_USER="${SUDO_USER:-${USER:-$(id -un)}}"
TARGET_GROUP="$(id -gn "${TARGET_USER}" 2>/dev/null || echo "${TARGET_USER}")"

sudo mkdir -p "${LOG_DIR}"
sudo chown -R "${TARGET_USER}:${TARGET_GROUP}" "${LOG_DIR}"
sudo chmod -R u+rwX "${LOG_DIR}"

printf "Configured %s with ownership for %s:%s\n" "${LOG_DIR}" "${TARGET_USER}" "${TARGET_GROUP}"
