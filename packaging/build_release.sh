#!/usr/bin/env bash
# Multi-Platform Release Packager for Linux and macOS (.tar.gz)
# Builds production binaries, runs validation, and packages archives.
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

VERSION="1.0.0"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

echo -e "${BLUE}================================================================${NC}"
echo -e "${BLUE}  InhausLab ec2pass Packaging - ${OS} (${ARCH})                 ${NC}"
echo -e "${BLUE}  Output Format: .tar.gz archive with installer & daemon        ${NC}"
echo -e "${BLUE}================================================================${NC}"

# 1. Compile Release Binaries
echo -e "\n${CYAN}[1/4] Compiling release binaries via cargo build --workspace --release...${NC}"
cargo build --workspace --release

# 2. Run Workspace Tests
echo -e "\n${CYAN}[2/4] Running test validation suite...${NC}"
cargo test --workspace

# 3. Create Distribution Staging Directory
echo -e "\n${CYAN}[3/4] Staging release artifacts...${NC}"
PACKAGE_NAME="ec2pass-v${VERSION}-${OS}-${ARCH}"
DIST_ROOT="dist"
STAGE_DIR="${DIST_ROOT}/${PACKAGE_NAME}"
ARCHIVE_PATH="${DIST_ROOT}/${PACKAGE_NAME}.tar.gz"

mkdir -p "$STAGE_DIR"

TARGET_RELEASE_DIR="${CARGO_TARGET_DIR:-target}/release"
if [ ! -f "$TARGET_RELEASE_DIR/ec2pass" ] && [ -f "target/release/ec2pass" ]; then
    TARGET_RELEASE_DIR="target/release"
fi
cp "$TARGET_RELEASE_DIR/ec2pass" "$STAGE_DIR/"
cp "$TARGET_RELEASE_DIR/ec2passd" "$STAGE_DIR/"

if [ "$OS" = "linux" ]; then
    cp packaging/linux/ec2passd.service "$STAGE_DIR/"
    cp packaging/linux/install.sh "$STAGE_DIR/"
    cp packaging/linux/uninstall.sh "$STAGE_DIR/"
elif [ "$OS" = "darwin" ]; then
    cp packaging/macos/com.inhauslab.ec2passd.plist "$STAGE_DIR/"
    cp packaging/macos/install.sh "$STAGE_DIR/"
    cp packaging/macos/uninstall.sh "$STAGE_DIR/"
fi

cp README.md MANUAL.md BUILD_AND_RELEASE.md platform.md USERDATA.md "$STAGE_DIR/"
chmod 755 "$STAGE_DIR/install.sh" "$STAGE_DIR/uninstall.sh"

# 4. Create .tar.gz Archive & SHA-256 Checksum
echo -e "\n${CYAN}[4/4] Creating ${ARCHIVE_PATH}...${NC}"
tar -czvf "$ARCHIVE_PATH" -C "$DIST_ROOT" "$PACKAGE_NAME"

if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$ARCHIVE_PATH" > "${ARCHIVE_PATH}.sha256"
else
    shasum -a 256 "$ARCHIVE_PATH" > "${ARCHIVE_PATH}.sha256"
fi

echo -e "\n${GREEN}Successfully created release package:${NC} ${ARCHIVE_PATH}"
echo -e "${GREEN}SHA-256 Checksum:${NC} $(cat "${ARCHIVE_PATH}.sha256")"
