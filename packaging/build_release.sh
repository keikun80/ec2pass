#!/usr/bin/env bash
# Local Release Packager for Linux and macOS (No GitHub Actions required)
set -euo pipefail

VERSION="1.14.0"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

echo "=== Building ec2pass release for ${OS}-${ARCH} ==="

# 1. Compile release binaries
cargo build --workspace --release

# 2. Run test verification
./tests/e2e_test.sh

# 3. Create dist archive
DIST_DIR="dist/ec2pass-v${VERSION}-${OS}-${ARCH}"
ARCHIVE_NAME="dist/ec2pass-v${VERSION}-${OS}-${ARCH}.tar.gz"

rm -rf "$DIST_DIR" "$ARCHIVE_NAME"
mkdir -p "$DIST_DIR"

cp target/release/ec2pass "$DIST_DIR/"
cp target/release/ec2passd "$DIST_DIR/"
cp target/release/multipass "$DIST_DIR/"

if [ "$OS" = "linux" ]; then
    cp packaging/linux/ec2passd.service "$DIST_DIR/"
    cp packaging/linux/install.sh "$DIST_DIR/"
elif [ "$OS" = "darwin" ]; then
    cp packaging/macos/com.amazonlinux.ec2passd.plist "$DIST_DIR/"
    cp packaging/macos/install.sh "$DIST_DIR/"
fi

cp README.md MANUAL.md "$DIST_DIR/"

tar -czvf "$ARCHIVE_NAME" -C "dist" "ec2pass-v${VERSION}-${OS}-${ARCH}"

# 4. Generate SHA-256 Checksum
if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$ARCHIVE_NAME" > "${ARCHIVE_NAME}.sha256"
else
    shasum -a 256 "$ARCHIVE_NAME" > "${ARCHIVE_NAME}.sha256"
fi

echo "Created release package: ${ARCHIVE_NAME}"
echo "Checksum: $(cat "${ARCHIVE_NAME}.sha256")"
