#!/usr/bin/env bash
# Universal Uninstaller Script for InhausLab ec2pass on Linux
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}=== InhausLab ec2pass Linux Uninstaller ===${NC}"

# 1. Check Root Privileges
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This script must be run as root (sudo ./uninstall.sh)${NC}" >&2
    exit 1
fi

PURGE_DATA=false
if [[ "${1:-}" == "--purge-data" || "${1:-}" == "-p" ]]; then
    PURGE_DATA=true
fi

# 2. Stop and disable Systemd Service
if command -v systemctl >/dev/null 2>&1; then
    echo -e "Stopping and disabling ec2passd systemd service..."
    systemctl stop ec2passd.service 2>/dev/null || true
    systemctl disable ec2passd.service 2>/dev/null || true
    rm -f /etc/systemd/system/ec2passd.service
    systemctl daemon-reload
    echo -e "${GREEN}Removed ec2passd.service.${NC}"
fi

# 3. Kill any remaining processes
pkill -9 -f ec2passd 2>/dev/null || true

# 4. Remove Binaries
INSTALL_DIR="/usr/local/bin"
rm -f "$INSTALL_DIR/ec2pass" "$INSTALL_DIR/ec2passd" "$INSTALL_DIR/multipass"
echo -e "${GREEN}Removed binaries from $INSTALL_DIR (ec2pass, ec2passd, multipass).${NC}"

# 5. Optional Data Purge
if [ "$PURGE_DATA" = true ]; then
    echo -e "${YELLOW}Purging VM instances and cache data...${NC}"
    rm -rf /var/lib/ec2pass /var/run/ec2pass.sock /etc/ec2pass
    echo -e "${GREEN}Data directories removed.${NC}"
else
    echo -e "VM instance data in /var/lib/ec2pass preserved. (Use --purge-data to remove completely)"
fi

echo -e "\n${GREEN}ec2pass uninstallation completed successfully!${NC}"
