#!/bin/bash
# Native Apple Silicon build. Honors RUSTUP_TOOLCHAIN; never changes it.
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "$(uname -s)" != Darwin || "$(uname -m)" != arm64 ]]; then
    echo 'This build requires native Apple Silicon macOS.' >&2
    exit 2
fi
cargo build --release --target aarch64-apple-darwin "$@"
python3 scripts/macos.py package "${CARGO_TARGET_DIR:-target}/aarch64-apple-darwin/release"
