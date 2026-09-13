#!/usr/bin/env bash

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXAMPLES_LIB_DIR="${PROJECT_ROOT}/examples/lib"
EXAMPLES_CLI_DIR="${PROJECT_ROOT}/examples/cli"

if [ ! -d "${EXAMPLES_LIB_DIR}" ]; then
    echo "Error: Directory ${EXAMPLES_LIB_DIR} does not exist." >&2
    exit 1
fi

TEMP_DIR="$(mktemp -d -t rtouch_test_all_XXXXXX)"
cleanup() {
    rm -rf "${TEMP_DIR}"
}
trap cleanup EXIT INT TERM

# Ensure local debug binary is compiled and available on PATH for CLI examples
cargo build --manifest-path "${PROJECT_ROOT}/Cargo.toml" --quiet
export PATH="${PROJECT_ROOT}/target/debug:${PATH}"

TEST_PROJECT_DIR="${TEMP_DIR}/test-runner"
cargo new --bin "${TEST_PROJECT_DIR}" --quiet
cd "${TEST_PROJECT_DIR}"

cargo add rtouch --path "${PROJECT_ROOT}" --quiet || cargo add rtouch --quiet

printf "Running library example tests...\n\n"
while IFS= read -r -d '' file_path; do
    filename="$(basename "${file_path}")"
    printf "Testing: %s\n" "${file_path}"
    cp "${file_path}" "${TEST_PROJECT_DIR}/src/main.rs"
    cargo test --quiet
    printf "Test for %s passed successfully.\n\n" "${filename}"
done < <(find "${EXAMPLES_LIB_DIR}" -type f -name "*.rs" -print0)

if [ -d "${EXAMPLES_CLI_DIR}" ]; then
    printf "Running CLI example scripts...\n\n"
    while IFS= read -r -d '' file_path; do
        filename="$(basename "${file_path}")"
        printf "Testing CLI: %s\n" "${file_path}"
        chmod +x "${file_path}"
        bash "${file_path}"
        printf "CLI test for %s passed successfully.\n\n" "${filename}"
    done < <(find "${EXAMPLES_CLI_DIR}" -type f -name "*.sh" -print0)
fi

cd "${PROJECT_ROOT}"
printf "All example tests passed successfully.\n"
