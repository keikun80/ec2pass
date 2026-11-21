#!/usr/bin/env bash
# macOS Installer Script for InhausLab ec2pass (multipass compatible)
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== InhausLab ec2pass macOS Installer ===${NC}"

# 1. Check Root Privileges
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This installer must be run as root (sudo ./install.sh)${NC}" >&2
    exit 1
fi

# 2. Check Architecture
ARCH=$(uname -m)
echo -e "Detected Architecture: ${GREEN}${ARCH}${NC} (HVF Hypervisor.framework Native Acceleration)"

# 3. Install Binaries
INSTALL_DIR="/usr/local/bin"
mkdir -p "$INSTALL_DIR"

if [ -f "./target/release/ec2pass" ]; then
    cp "./target/release/ec2pass" "$INSTALL_DIR/ec2pass"
    cp "./target/release/ec2passd" "$INSTALL_DIR/ec2passd"
    cp "./target/release/multipass" "$INSTALL_DIR/multipass"
elif [ -f "./ec2pass" ]; then
    cp "./ec2pass" "$INSTALL_DIR/ec2pass"
    cp "./ec2passd" "$INSTALL_DIR/ec2passd"
    if [ -f "./multipass" ]; then
        cp "./multipass" "$INSTALL_DIR/multipass"
    else
        ln -sf "$INSTALL_DIR/ec2pass" "$INSTALL_DIR/multipass"
    fi
else
    echo -e "${RED}Error: ec2pass binaries not found in current directory or target/release.${NC}" >&2
    exit 1
fi

chmod 755 "$INSTALL_DIR/ec2pass" "$INSTALL_DIR/ec2passd" "$INSTALL_DIR/multipass"
echo -e "${GREEN}Installed binaries to $INSTALL_DIR (ec2pass, multipass, ec2passd)${NC}"

# 4. Install Launchd Daemon
PLIST_SRC="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/com.inhauslab.ec2passd.plist"
PLIST_DST="/Library/LaunchDaemons/com.inhauslab.ec2passd.plist"

if [ -f "$PLIST_SRC" ]; then
    cp "$PLIST_SRC" "$PLIST_DST"
    chown root:wheel "$PLIST_DST"
    chmod 644 "$PLIST_DST"
    launchctl unload "$PLIST_DST" 2>/dev/null || true
    launchctl load -w "$PLIST_DST"
    echo -e "${GREEN}Launchd daemon registered and started (${PLIST_DST})${NC}"
fi

echo -e "\n${GREEN}macOS installation complete! Try running:${NC}"
echo "  ec2pass version"
echo "  ec2pass find"
echo "  ec2pass launch al2023 -n test-mac"
