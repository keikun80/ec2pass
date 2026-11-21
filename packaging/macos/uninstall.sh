#!/usr/bin/env bash
# macOS Uninstaller Script for InhausLab ec2pass
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}=== InhausLab ec2pass macOS Uninstaller ===${NC}"

# 1. Check Root Privileges
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This script must be run as root (sudo ./uninstall.sh)${NC}" >&2
    exit 1
fi

PURGE_DATA=false
if [[ "${1:-}" == "--purge-data" || "${1:-}" == "-p" ]]; then
    PURGE_DATA=true
fi

# 2. Stop and Unload Launchd Daemon
for PLIST_PATH in "/Library/LaunchDaemons/com.inhauslab.ec2passd.plist" "/Library/LaunchDaemons/com.amazonlinux.ec2passd.plist"; do
    if [ -f "$PLIST_PATH" ]; then
        echo -e "Unloading launchd daemon..."
        launchctl unload -w "$PLIST_PATH" 2>/dev/null || true
        rm -f "$PLIST_PATH"
        echo -e "${GREEN}Removed launchd service: $PLIST_PATH${NC}"
    fi
done

# 3. Kill any lingering daemon processes
pkill -9 -f ec2passd 2>/dev/null || true

# 4. Remove Binaries
INSTALL_DIR="/usr/local/bin"
rm -f "$INSTALL_DIR/ec2pass" "$INSTALL_DIR/ec2passd"
echo -e "${GREEN}Removed binaries from $INSTALL_DIR (ec2pass, ec2passd).${NC}"

# 5. Optional Data Purge
if [ "$PURGE_DATA" = true ]; then
    echo -e "${YELLOW}Purging VM instances and cache data...${NC}"
    rm -rf /var/root/Library/Application\ Support/ec2pass /var/run/ec2pass.sock
    echo -e "${GREEN}Data directories removed.${NC}"
else
    echo -e "VM instance data preserved. (Use --purge-data to remove completely)"
fi

echo -e "\n${GREEN}ec2pass uninstallation completed successfully on macOS!${NC}"
