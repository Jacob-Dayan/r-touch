#!/usr/bin/env bash

set -euo pipefail

BASE_TMP_DIR="/dev/shm/test_all"
PROJECT_ROOT="$(pwd)"
USAGE_DIR="${PROJECT_ROOT}/usage/lib"

if [ ! -d "${USAGE_DIR}" ]; then
    echo "Error: Directory ${USAGE_DIR} does not exist."
    exit 1
fi

rm -rf "${BASE_TMP_DIR}"
mkdir -p "${BASE_TMP_DIR}"

cleanup() {
    rm -rf "${BASE_TMP_DIR}"
}
trap cleanup EXIT

find "${USAGE_DIR}" -type f -name "*.rs" | while read -r file_path; do
    filename=$(basename "${file_path}")
    project_name="${filename%.rs}"
    
    project_name=$(echo "${project_name}" | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9_-' '_')

    echo "=========================================="
    echo "Testing: ${file_path}"
    echo "Project name: ${project_name}"
    echo "=========================================="

    TEST_PROJECT_DIR="${BASE_TMP_DIR}/${project_name}"

    cd "${BASE_TMP_DIR}"
    cargo new --bin "${project_name}" --quiet
    cd "${TEST_PROJECT_DIR}"

    cargo add rtouch --path "${PROJECT_ROOT}" --quiet || cargo add rtouch --quiet

    cp "${file_path}" src/main.rs

    echo "Running cargo test..."
    cargo test

    cd "${PROJECT_ROOT}"
    rm -rf "${TEST_PROJECT_DIR}"

    echo -e "Test for ${filename} passed successfully!\n"
done

echo "All tests passed successfully!"
