#!/usr/bin/env bash
# Host Automation Runner for Vagrant ec2pass Windows 11 Build, Test, and Packaging
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo -e "${BLUE}================================================================${NC}"
echo -e "${BLUE}  InhausLab ec2pass Windows 11 Vagrant Build, Test & Packaging  ${NC}"
echo -e "${BLUE}================================================================${NC}"

cd "$PROJECT_ROOT"

# 1. Check Vagrant executable
if ! command -v vagrant >/dev/null 2>&1; then
    echo -e "${RED}Error: vagrant command not found in PATH.${NC}" >&2
    exit 1
fi
echo -e "Detected Vagrant: ${GREEN}$(vagrant --version)${NC}"

# 2. Check VirtualBox provider
if ! command -v VBoxManage >/dev/null 2>&1; then
    echo -e "${RED}Error: VirtualBox VBoxManage not found in PATH.${NC}" >&2
    exit 1
fi
echo -e "Detected VirtualBox: ${GREEN}$(VBoxManage --version)${NC}"

# 3. Check Available Disk Space
AVAIL_GB=$(df -g / | awk 'NR==2 {print $4}')
echo -e "Host Available Disk Space: ${CYAN}${AVAIL_GB} GB${NC}"
if [ "$AVAIL_GB" -lt 30 ]; then
    echo -e "${YELLOW}Warning: Windows 11 box download and virtual disk typically require ~30 GB of free space.${NC}"
fi

# 4. Check if box is already cached
BOX_NAME="${VAGRANT_WIN11_BOX:-gusztavvargadr/windows-11}"
if vagrant box list | grep -q "$BOX_NAME"; then
    echo -e "Found cached Windows 11 box: ${GREEN}$BOX_NAME${NC}"
else
    echo -e "${YELLOW}Note: '$BOX_NAME' not found locally. Vagrant will download it (~13.7 GB). This may take several minutes.${NC}"
fi

# 5. Boot Windows 11 Vagrant VM
echo -e "\n${YELLOW}[1/4] Booting Windows 11 Vagrant VM (win11)...${NC}"
vagrant up win11 --provider virtualbox

# 6. Upload source code to guest VM
echo -e "\n${YELLOW}[2/4] Packaging and uploading source code to Windows 11 guest...${NC}"
TEMP_ZIP="/tmp/ec2pass_src.zip"
rm -f "$TEMP_ZIP"

# Create clean zip excluding build artifacts
zip -rq "$TEMP_ZIP" . -x "target/*" "dist/*" ".git/*" ".vagrant/*"
vagrant upload "$TEMP_ZIP" "C:\\ec2pass_src.zip" win11
rm -f "$TEMP_ZIP"

vagrant winrm -e -c 'powershell -ExecutionPolicy Bypass -Command "Expand-Archive -Force -Path C:\ec2pass_src.zip -DestinationPath C:\vagrant; Remove-Item -Force C:\ec2pass_src.zip"' win11
echo -e "  ${GREEN}✓ Source code unpacked to C:\\vagrant in Windows 11 VM.${NC}"

# 7. Run Build, Test & Packaging inside guest VM
echo -e "\n${YELLOW}[3/4] Running build, E2E tests, and release packaging inside Windows 11 VM...${NC}"
vagrant winrm -e -c 'powershell -ExecutionPolicy Bypass -File C:\vagrant\packaging\vagrant\build_and_test_win11.ps1' win11

# 8. Retrieve Windows 11 release artifacts to host dist/
echo -e "\n${YELLOW}[4/4] Retrieving Windows 11 release package from guest VM to host dist/...${NC}"
mkdir -p "$PROJECT_ROOT/dist"

# Download release package using powershell base64 or vagrant winrm
vagrant winrm -e -c 'powershell -ExecutionPolicy Bypass -Command "[Convert]::ToBase64String([IO.File]::ReadAllBytes(\"C:\vagrant\dist\ec2pass-v0.1.0-windows-x86_64.zip\"))"' win11 | tr -d '\r\n' | base64 --decode > "$PROJECT_ROOT/dist/ec2pass-v0.1.0-windows-x86_64.zip"
vagrant winrm -e -c 'powershell -ExecutionPolicy Bypass -Command "Get-Content C:\vagrant\dist\ec2pass-v0.1.0-windows-x86_64.zip.sha256"' win11 > "$PROJECT_ROOT/dist/ec2pass-v0.1.0-windows-x86_64.zip.sha256"

echo -e "\n${GREEN}================================================================${NC}"
echo -e "${GREEN}  SUCCESS: Windows 11 build, E2E tests, and release packaging PASSED!${NC}"
echo -e "${GREEN}================================================================${NC}"
ls -lh "$PROJECT_ROOT/dist/ec2pass"*windows* || true
