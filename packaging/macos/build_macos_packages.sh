#!/usr/bin/env bash
# macOS Multi-Architecture Packaging Script
# Builds Universal Binary (Apple Silicon + Intel) and architecture-specific .tar.gz packages
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

VERSION="1.0.0"
DIST_DIR="dist"
mkdir -p "$DIST_DIR" target/universal

echo -e "${BLUE}================================================================${NC}"
echo -e "${BLUE}       InhausLab ec2pass macOS Distribution Packager            ${NC}"
echo -e "${BLUE}================================================================${NC}"

# 1. Compile both x86_64 and aarch64
echo -e "\n${CYAN}[1/4] Ensuring release binaries for both architectures...${NC}"
cargo build --workspace --release --target x86_64-apple-darwin
cargo build --workspace --release --target aarch64-apple-darwin

# 2. Combine into Universal 2 Binaries via lipo
echo -e "\n${CYAN}[2/4] Creating Universal 2 Binaries via lipo...${NC}"
lipo -create -output target/universal/ec2pass \
    target/x86_64-apple-darwin/release/ec2pass \
    target/aarch64-apple-darwin/release/ec2pass

lipo -create -output target/universal/ec2passd \
    target/x86_64-apple-darwin/release/ec2passd \
    target/aarch64-apple-darwin/release/ec2passd

lipo -create -output target/universal/multipass \
    target/x86_64-apple-darwin/release/multipass \
    target/aarch64-apple-darwin/release/multipass

echo -e "  ${GREEN}✓ Universal binaries created in target/universal/${NC}"
file target/universal/ec2pass target/universal/ec2passd target/universal/multipass

# 3. Helper function to create .tar.gz package
package_target() {
    local ARCH="$1"
    local BIN_DIR="$2"
    local PKG_NAME="ec2pass-v${VERSION}-darwin-${ARCH}"
    local STAGE="${DIST_DIR}/${PKG_NAME}"
    local ARCHIVE="${DIST_DIR}/${PKG_NAME}.tar.gz"

    echo -e "\n${CYAN}[3/4] Staging and packaging ${PKG_NAME}.tar.gz...${NC}"
    rm -rf "$STAGE" "$ARCHIVE"
    mkdir -p "$STAGE"

    cp "$BIN_DIR/ec2pass" "$STAGE/"
    cp "$BIN_DIR/ec2passd" "$STAGE/"
    cp "$BIN_DIR/multipass" "$STAGE/"

    cp packaging/macos/com.inhauslab.ec2passd.plist "$STAGE/"
    cp packaging/macos/install.sh "$STAGE/"
    cp packaging/macos/uninstall.sh "$STAGE/"
    cp README.md MANUAL.md BUILD_AND_RELEASE.md platform.md USERDATA.md "$STAGE/"

    chmod 755 "$STAGE/install.sh" "$STAGE/uninstall.sh" "$STAGE/ec2pass" "$STAGE/ec2passd" "$STAGE/multipass"

    tar -czvf "$ARCHIVE" -C "$DIST_DIR" "$PKG_NAME"

    shasum -a 256 "$ARCHIVE" > "${ARCHIVE}.sha256"
    echo -e "  ${GREEN}✓ Created: ${ARCHIVE}${NC}"
    echo -e "  ${GREEN}  Checksum:${NC} $(cat "${ARCHIVE}.sha256")"
}

# 4. Create all 3 package variants
# A. Universal (Recommended - runs natively on both M-series and Intel)
package_target "universal" "target/universal"

# B. Apple Silicon Native
package_target "aarch64" "target/aarch64-apple-darwin/release"

# C. Intel Mac Native
package_target "x86_64" "target/x86_64-apple-darwin/release"

echo -e "\n${BLUE}================================================================${NC}"
echo -e "${GREEN}All macOS release packages successfully created in ${DIST_DIR}/:${NC}"
ls -lh ${DIST_DIR}/ec2pass-v${VERSION}-darwin-*.tar.gz
echo -e "${BLUE}================================================================${NC}"
