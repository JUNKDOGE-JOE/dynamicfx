#!/bin/bash
# Installs only a verified, already packaged native macOS bundle.
set -euo pipefail
cd "$(dirname "$0")/.."
exec python3 scripts/macos.py install "$@"
