#!/usr/bin/env bash
# Universal Installer Script for InhausLab ec2pass (multipass compatible) on Linux
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== InhausLab ec2pass Linux Installer ===${NC}"

# 1. Check Root Privileges
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This script must be run as root (sudo ./install.sh)${NC}" >&2
    exit 1
fi

# 2. Check Architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)  TARGET_ARCH="x86_64" ;;
    aarch64) TARGET_ARCH="aarch64" ;;
    *)
        echo -e "${RED}Error: Unsupported architecture $ARCH${NC}" >&2
        exit 1
        ;;
esac
echo -e "Detected Architecture: ${GREEN}${TARGET_ARCH}${NC}"

# 3. Check KVM Hypervisor Support
if [ ! -e /dev/kvm ]; then
    echo -e "${RED}Warning: /dev/kvm not found. Hardware acceleration (KVM) may be disabled in BIOS/UEFI.${NC}"
else
    echo -e "${GREEN}KVM Hardware Acceleration: Available (/dev/kvm)${NC}"
fi

# 4. Install Binaries
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

# 5. Install and Start Systemd Service
if command -v systemctl >/dev/null 2>&1; then
    SERVICE_FILE="/etc/systemd/system/ec2passd.service"
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    if [ -f "$SCRIPT_DIR/ec2passd.service" ]; then
        cp "$SCRIPT_DIR/ec2passd.service" "$SERVICE_FILE"
    else
        cat <<EOF > "$SERVICE_FILE"
[Unit]
Description=InhausLab ec2pass Virtual Machine Daemon (ec2passd)
After=network.target

[Service]
Type=simple
ExecStart=$INSTALL_DIR/ec2passd --address 127.0.0.1:50051
Restart=always
RestartSec=5
User=root

[Install]
WantedBy=multi-user.target
EOF
    fi

    systemctl daemon-reload
    systemctl enable --now ec2passd.service
    echo -e "${GREEN}Systemd service 'ec2passd.service' enabled and started.${NC}"
fi

echo -e "\n${GREEN}Installation complete! Try running:${NC}"
echo "  ec2pass version"
echo "  ec2pass find"
echo "  ec2pass launch al2023 -n test-vm"
