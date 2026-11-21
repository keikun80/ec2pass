#!/usr/bin/env bash
# Clean build artifacts, temporary caches, and runtime data before Git commit
set -euo pipefail

echo "=== Cleaning build artifacts and runtime data for Git commit ==="

# 1. Terminate background daemons
pkill -f "ec2passd" || true
pkill -f "mpd" || true
pkill -f "alpassd" || true

# 2. Cargo clean
if command -v cargo >/dev/null 2>&1; then
    cargo clean
fi

# 3. Remove build and runtime directories
rm -rf target dist data .data .ec2pass-data .ec2ps-data

# 4. Remove temporary packaging files and logs
rm -f packaging/windows/*_generated.sed
rm -f *.log *.tmp *.zip *.sha256 *.tar.gz

echo "Working directory is clean and ready for Git commit!"
