#!/usr/bin/env bash
# macOS Installer Script for InhausLab ec2pass
# Enforces Hypervisor.framework and QEMU as mandatory prerequisites.
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}================================================================${NC}"
echo -e "${BLUE}       InhausLab ec2pass macOS System Installer                 ${NC}"
echo -e "${BLUE}================================================================${NC}"

# 1. Check Root Privileges
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This installer must be run as root (sudo ./install.sh)${NC}" >&2
    exit 1
fi

# 2. Check Architecture
ARCH=$(uname -m)
echo -e "Detected Architecture: ${GREEN}${ARCH}${NC}"

# 3. Mandatory Hypervisor (HVF + QEMU) Pre-flight Check
echo -e "\n${CYAN}[1/4] Checking macOS Hypervisor prerequisites...${NC}"
HV_SUPPORT=$(sysctl -n kern.hv_support 2>/dev/null || echo "1")
if [ "$HV_SUPPORT" != "1" ]; then
    echo -e "${RED}[ERROR] macOS Hypervisor.framework is not supported on this CPU.${NC}"
    exit 1
fi
echo -e "  ${GREEN}✓ Apple Hypervisor.framework (HVF) is supported by the kernel.${NC}"

QEMU_INSTALLED=false
if command -v qemu-system-x86_64 >/dev/null 2>&1 || command -v qemu-system-aarch64 >/dev/null 2>&1 || [ -f "/usr/local/bin/qemu-system-x86_64" ] || [ -f "/opt/homebrew/bin/qemu-system-aarch64" ]; then
    QEMU_INSTALLED=true
fi

if [ "$QEMU_INSTALLED" = false ]; then
    echo -e "\n${RED}================================================================${NC}"
    echo -e "${RED}[ERROR] 필수 하이퍼바이저 엔진(QEMU)이 감지되지 않았습니다.${NC}"
    echo -e "ec2pass는 macOS에서 커널 HVF 가속을 연동하기 위해 QEMU가 필수입니다."
    echo -e "\n${YELLOW}▶ 해결 방법:${NC}"
    echo -e "  터미널에서 Homebrew를 통해 QEMU를 설치해 주세요:"
    echo -e "  ${GREEN}brew install qemu${NC}"
    echo -e "\n설치 완료 후 본 설치 스크립트(sudo ./install.sh)를 다시 실행해 주세요."
    echo -e "${RED}================================================================${NC}"
    exit 1
fi
echo -e "  ${GREEN}✓ QEMU virtualization engine is installed.${NC}"

# 4. Install Binaries
echo -e "\n${CYAN}[2/4] Installing binaries to /usr/local/bin...${NC}"
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
echo -e "  ${GREEN}✓ Installed ec2pass, multipass, ec2passd to $INSTALL_DIR${NC}"

# 5. Install Launchd Daemon
echo -e "\n${CYAN}[3/4] Configuring launchd background daemon...${NC}"
PLIST_SRC="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/com.inhauslab.ec2passd.plist"
PLIST_DST="/Library/LaunchDaemons/com.inhauslab.ec2passd.plist"

if [ -f "$PLIST_SRC" ]; then
    cp "$PLIST_SRC" "$PLIST_DST"
    chown root:wheel "$PLIST_DST"
    chmod 644 "$PLIST_DST"
    launchctl unload "$PLIST_DST" 2>/dev/null || true
    launchctl load -w "$PLIST_DST"
    echo -e "  ${GREEN}✓ Launchd daemon registered and active (${PLIST_DST})${NC}"
fi

# 6. Verify Installation
echo -e "\n${CYAN}[4/4] Verifying installation...${NC}"
echo -e "${GREEN}macOS installation successfully complete!${NC}"
echo -e "Available commands:"
echo "  ec2pass version"
echo "  ec2pass find"
echo "  ec2pass launch al2023 -n my-mac-vm"
