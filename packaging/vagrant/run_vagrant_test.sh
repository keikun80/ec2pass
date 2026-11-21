#!/usr/bin/env bash
# Host Automation Runner for Vagrant ec2pass Linux Build, Test, and Packaging
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
echo -e "${BLUE}  InhausLab ec2pass Vagrant Linux Build, Test & Packaging       ${NC}"
echo -e "${BLUE}================================================================${NC}"

cd "$PROJECT_ROOT"

# 1. Check Vagrant executable
if ! command -v vagrant >/dev/null 2>&1; then
    echo -e "${RED}Error: vagrant command not found in PATH.${NC}" >&2
    exit 1
fi

echo -e "Detected Vagrant: ${GREEN}$(vagrant --version)${NC}"

# 2. Boot Vagrant VM
echo -e "\n${YELLOW}[1/4] Booting Vagrant VM (Amazon Linux 2023)...${NC}"
vagrant up --provider virtualbox

# 3. Upload source code to guest VM via vagrant upload
echo -e "\n${YELLOW}[2/4] Packaging and uploading source code to guest VM...${NC}"
TEMP_TAR="/tmp/ec2pass_src.tar.gz"
rm -f "$TEMP_TAR"

tar --exclude "target" --exclude "dist" --exclude ".git" --exclude ".vagrant" -czf "$TEMP_TAR" .
vagrant upload "$TEMP_TAR" "/tmp/ec2pass_src.tar.gz"
rm -f "$TEMP_TAR"

vagrant ssh -c "sudo mkdir -p /vagrant && sudo chown -R vagrant:vagrant /vagrant && tar -xzf /tmp/ec2pass_src.tar.gz -C /vagrant && rm -f /tmp/ec2pass_src.tar.gz"
echo -e "  ${GREEN}✓ Source code uploaded and unpacked to /vagrant in guest VM.${NC}"

# 4. Execute Build, Test & Packaging inside guest VM
echo -e "\n${YELLOW}[3/4] Running build, E2E tests, and release packaging inside Linux VM...${NC}"
vagrant ssh -c "bash /vagrant/packaging/vagrant/build_and_test.sh"

# 5. Retrieve Linux release artifacts from guest VM
echo -e "\n${YELLOW}[4/4] Retrieving Linux release package from guest VM to host dist/...${NC}"
mkdir -p "$PROJECT_ROOT/dist"
SSH_KEY="$(vagrant ssh-config | grep IdentityFile | awk '{print $2}')"
SSH_PORT="$(vagrant ssh-config | grep Port | awk '{print $2}')"
scp -i "$SSH_KEY" -P "$SSH_PORT" -o StrictHostKeyChecking=no 'vagrant@127.0.0.1:/vagrant/dist/ec2pass-*-linux-*' "$PROJECT_ROOT/dist/"

echo -e "\n${GREEN}================================================================${NC}"
echo -e "${GREEN}  SUCCESS: Linux build, E2E tests, and release packaging PASSED!${NC}"
echo -e "${GREEN}================================================================${NC}"
ls -lh "$PROJECT_ROOT/dist/ec2pass"*linux* || true
