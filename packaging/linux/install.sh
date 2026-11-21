#!/usr/bin/env bash
# Universal Installer Script for InhausLab ec2pass on Linux
# Enforces Linux KVM hardware acceleration as a mandatory prerequisite.
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}================================================================${NC}"
echo -e "${BLUE}       InhausLab ec2pass Linux System Installer                 ${NC}"
echo -e "${BLUE}================================================================${NC}"

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

# 3. Mandatory Hypervisor (KVM) Pre-flight Check
echo -e "\n${CYAN}[1/4] Checking Linux KVM Hypervisor prerequisites...${NC}"
KVM_READY=true

if [ ! -e /dev/kvm ]; then
    echo -e "${RED}[X] Hardware acceleration device /dev/kvm was not found.${NC}"
    KVM_READY=false
else
    echo -e "  ${GREEN}✓ /dev/kvm device node is active.${NC}"
fi

QEMU_READY=false
if command -v qemu-system-x86_64 >/dev/null 2>&1 || command -v qemu-system-aarch64 >/dev/null 2>&1 || command -v qemu-kvm >/dev/null 2>&1; then
    QEMU_READY=true
    echo -e "  ${GREEN}✓ QEMU virtualization binary is installed.${NC}"
else
    echo -e "${RED}[X] QEMU system emulator binary was not found.${NC}"
fi

if [ "$KVM_READY" = false ] || [ "$QEMU_READY" = false ]; then
    echo -e "\n${RED}================================================================${NC}"
    echo -e "${RED}[ERROR] Required hypervisor (KVM) prerequisites are not met.${NC}"
    echo -e "ec2pass strictly requires hardware virtualization without simulation."
    echo -e "\n${YELLOW}▶ Resolution (install KVM and QEMU using commands below):${NC}"
    if command -v apt-get >/dev/null 2>&1; then
        echo -e "  sudo apt-get update && sudo apt-get install -y qemu-system-x86 qemu-utils libvirt-daemon-system"
        echo -e "  sudo usermod -aG kvm \$SUDO_USER && newgrp kvm"
    elif command -v dnf >/dev/null 2>&1; then
        echo -e "  sudo dnf install -y qemu-kvm qemu-img"
        echo -e "  sudo usermod -aG kvm \$SUDO_USER && newgrp kvm"
    elif command -v pacman >/dev/null 2>&1; then
        echo -e "  sudo pacman -S qemu-full"
        echo -e "  sudo usermod -aG kvm \$SUDO_USER && newgrp kvm"
    else
        echo -e "  Install qemu-kvm and qemu-img from your package manager and grant access to /dev/kvm."
    fi
    echo -e "\nAfter installation, re-run this script (sudo ./install.sh)."
    echo -e "${RED}================================================================${NC}"
    exit 1
fi
echo -e "${GREEN}Hypervisor status: OK (KVM acceleration ready)${NC}"

# 4. Install Binaries
echo -e "\n${CYAN}[2/4] Installing binaries to /usr/local/bin...${NC}"
INSTALL_DIR="/usr/local/bin"
mkdir -p "$INSTALL_DIR"

if [ -f "./target/release/ec2pass" ]; then
    cp "./target/release/ec2pass" "$INSTALL_DIR/ec2pass"
    cp "./target/release/ec2passd" "$INSTALL_DIR/ec2passd"
elif [ -f "./ec2pass" ]; then
    cp "./ec2pass" "$INSTALL_DIR/ec2pass"
    cp "./ec2passd" "$INSTALL_DIR/ec2passd"
else
    echo -e "${RED}Error: ec2pass binaries not found in current directory or target/release.${NC}" >&2
    exit 1
fi

chmod 755 "$INSTALL_DIR/ec2pass" "$INSTALL_DIR/ec2passd"
echo -e "  ${GREEN}✓ Installed ec2pass, ec2passd to $INSTALL_DIR${NC}"

# 5. Install and Start Systemd Service
echo -e "\n${CYAN}[3/4] Configuring ec2passd systemd background service...${NC}"
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
    echo -e "  ${GREEN}✓ Systemd service 'ec2passd.service' registered and running.${NC}"
fi

# 6. Verify Installation
echo -e "\n${CYAN}[4/4] Verifying installation...${NC}"
echo -e "${GREEN}Installation successfully completed!${NC}"
echo -e "Available commands:"
echo "  ec2pass version"
echo "  ec2pass find"
echo "  ec2pass launch al2023 -n my-vm"
