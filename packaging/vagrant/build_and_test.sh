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

echo -e "\n${YELLOW}[Step 1/5] Verifying environment & tools...${NC}"
echo "Rustc:    $(rustc --version)"
echo "Cargo:    $(cargo --version)"
echo "Kernel:   $(uname -r) ($(uname -m))"
if [ -e /dev/kvm ]; then
    echo -e "KVM Accel: ${GREEN}Available (/dev/kvm)${NC}"
else
    echo -e "KVM Accel: ${YELLOW}Not detected (Fallback simulation enabled)${NC}"
fi

cd "$PROJECT_DIR"

echo -e "\n${YELLOW}[Step 2/5] Compiling release binaries with cargo build --release...${NC}"
cargo build --workspace --release

echo -e "\n${YELLOW}[Step 3/5] Running Rust unit & component tests...${NC}"
cargo test --workspace

echo -e "\n${YELLOW}[Step 4/5] Staging binaries to /usr/local/bin...${NC}"
sudo cp "$CARGO_TARGET_DIR/release/ec2pass" "/usr/local/bin/ec2pass"
sudo cp "$CARGO_TARGET_DIR/release/ec2passd" "/usr/local/bin/ec2passd"
sudo cp "$CARGO_TARGET_DIR/release/multipass" "/usr/local/bin/multipass"
sudo chmod 755 /usr/local/bin/ec2pass /usr/local/bin/ec2passd /usr/local/bin/multipass

echo -e "Installed binaries:"
ls -lh /usr/local/bin/ec2pass* /usr/local/bin/multipass

echo -e "\n${YELLOW}[Step 5/5] Executing 7-Stage End-to-End (E2E) Test Suite...${NC}"
export EC2PASS_BIN="/usr/local/bin/ec2pass"
bash "$PROJECT_DIR/tests/e2e_test.sh"

echo -e "\n${GREEN}================================================================${NC}"
echo -e "${GREEN}  ALL BUILD AND E2E INTEGRATION TESTS PASSED IN LINUX VM!       ${NC}"
echo -e "${GREEN}================================================================${NC}"
