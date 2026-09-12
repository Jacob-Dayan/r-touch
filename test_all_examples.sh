#!/usr/bin/env bash

set -euo pipefail

BASE_TMP_DIR="/tmp/test_all"
PROJECT_ROOT="$(pwd)"
EXAMPLES_DIR="${PROJECT_ROOT}/examples/lib"

debug() {
    echo "DEBUG: $1"
}

if [ ! -d "${EXAMPLES_DIR}" ]; then
    echo "Error: Directory ${EXAMPLES_DIR} does not exist."
    exit 1
fi

debug "setting up temp directory"

rm -rf "${BASE_TMP_DIR}"
mkdir -p "${BASE_TMP_DIR}"

cleanup() {
    rm -rf "${BASE_TMP_DIR}"
}
trap cleanup EXIT

TEST_PROJECT_DIR="${BASE_TMP_DIR}/test-runner"
cargo new --bin "${TEST_PROJECT_DIR}" --quiet
debug "finished setup"
cd "${TEST_PROJECT_DIR}"

debug "adding R-touch to dependencies"

cargo add rtouch --path "${PROJECT_ROOT}" --quiet || cargo add rtouch --quiet

printf "DONE.\nStarting tests...\n\n"
find "${EXAMPLES_DIR}" -type f -name "*.rs" | while read -r file_path; do
    filename=$(basename "${file_path}")

echo "=============================================================================================="
    echo "Testing: ${file_path}"
echo "=============================================================================================="

    cp "${file_path}" "${TEST_PROJECT_DIR}/src/main.rs"

    echo "Running cargo test..."
    cargo test

    echo -e "Test for ${filename} passed successfully!\n"
done

find "${PROJECT_ROOT}/examples/cli" -type f -name "*.sh" | while read -r file_path; do
    filename=$(basename "${file_path}")

echo "==============================================================================================="
    echo "Testing CLI: ${file_path}"
echo "==============================================================================================="

    chmod +x "${file_path}"
    bash "${file_path}"

    echo -e "Test for ${filename} passed successfully!\n"
done

cd "${PROJECT_ROOT}"
echo "All tests passed successfully!"
