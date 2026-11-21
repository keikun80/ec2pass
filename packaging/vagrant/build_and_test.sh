#!/usr/bin/env bash
# Build, Unit Test, and Linux E2E Verification Script for ec2pass
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${CYAN}================================================================${NC}"
echo -e "${CYAN}  InhausLab ec2pass Linux Automated Build & Test Pipeline       ${NC}"
echo -e "${CYAN}================================================================${NC}"

# Source rust environment
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
elif [ -f "/home/vagrant/.cargo/env" ]; then
    source "/home/vagrant/.cargo/env"
fi

PROJECT_DIR="/vagrant"
if [ ! -d "$PROJECT_DIR" ]; then
    PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fi

# Use native fast ext4 directory for target artifacts to avoid shared folder lock/latency issues
export CARGO_TARGET_DIR="${HOME}/target_cache"
mkdir -p "$CARGO_TARGET_DIR"

echo -e "\n${YELLOW}[Step 1/6] Verifying environment & tools...${NC}"
echo "Rustc:    $(rustc --version)"
echo "Cargo:    $(cargo --version)"
echo "Kernel:   $(uname -r) ($(uname -m))"
if [ -e /dev/kvm ]; then
    echo -e "KVM Accel: ${GREEN}Available (/dev/kvm)${NC}"
else
    echo -e "KVM Accel: ${YELLOW}Not detected in this VM${NC}"
fi

cd "$PROJECT_DIR"

echo -e "\n${YELLOW}[Step 2/6] Compiling release binaries with cargo build --release...${NC}"
cargo build --workspace --release

echo -e "\n${YELLOW}[Step 3/6] Running Rust unit & component tests...${NC}"
cargo test --workspace

echo -e "\n${YELLOW}[Step 4/6] Staging binaries to /usr/local/bin...${NC}"
sudo cp "$CARGO_TARGET_DIR/release/ec2pass" "/usr/local/bin/ec2pass"
sudo cp "$CARGO_TARGET_DIR/release/ec2passd" "/usr/local/bin/ec2passd"
sudo chmod 755 /usr/local/bin/ec2pass /usr/local/bin/ec2passd

echo -e "Installed binaries:"
ls -lh /usr/local/bin/ec2pass*

echo -e "\n${YELLOW}[Step 5/6] Executing End-to-End (E2E) Test Suite...${NC}"
export EC2PASS_BIN="/usr/local/bin/ec2pass"
export EC2PASSD_BIN="/usr/local/bin/ec2passd"
bash "$PROJECT_DIR/tests/e2e_test.sh"

echo -e "\n${YELLOW}[Step 6/6] Packaging Linux Release Archive...${NC}"
bash "$PROJECT_DIR/packaging/build_release.sh"

echo -e "\n${GREEN}================================================================${NC}"
echo -e "${GREEN}  ALL BUILD, E2E TESTS, AND LINUX PACKAGING PASSED IN LINUX VM! ${NC}"
echo -e "${GREEN}================================================================${NC}"
