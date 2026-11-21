#!/usr/bin/env bash
# End-to-End Automated Test Suite for ec2pass on Linux and macOS
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

BIN="${EC2PASS_BIN:-./target/release/ec2pass}"
if [ ! -f "$BIN" ]; then
    if [ -f "./target/debug/ec2pass" ]; then
        BIN="./target/debug/ec2pass"
    elif command -v ec2pass >/dev/null 2>&1; then
        BIN="$(command -v ec2pass)"
    fi
fi

if [ ! -f "$BIN" ]; then
    echo -e "${RED}Error: ec2pass binary not found. Run 'cargo build' first.${NC}"
    exit 1
fi

echo -e "${CYAN}=== Running ec2pass End-to-End Verification Suite ===${NC}"

# Test 1: Version check
echo -n "[1/7] Testing version command... "
$BIN --version >/dev/null
echo -e "${GREEN}PASSED${NC}"

# Test 2: Find catalog check
echo -n "[2/7] Testing find command... "
$BIN find >/dev/null
$BIN find minimal --format json >/dev/null
echo -e "${GREEN}PASSED${NC}"

# Test 3: Networks list
echo -n "[3/7] Testing networks command... "
$BIN networks >/dev/null
echo -e "${GREEN}PASSED${NC}"

# Test 4: Launch instance
TEST_VM="e2e-ci-$(date +%s)"
echo -n "[4/7] Launching test instance '${TEST_VM}'... "
$BIN launch al2023 -n "$TEST_VM" --cpus 1 --memory 1G --disk 5G >/dev/null
echo -e "${GREEN}PASSED${NC}"

# Test 5: Verify list and info
echo -n "[5/7] Verifying list and info... "
$BIN list | grep -q "$TEST_VM"
$BIN info "$TEST_VM" | grep -q "Running"
echo -e "${GREEN}PASSED${NC}"

# Test 6: Execute commands inside VM
echo -n "[6/7] Executing commands (whoami, os-release)... "
USER_OUT=$($BIN exec "$TEST_VM" -- whoami | tr -d '\r\n')
if [ "$USER_OUT" != "ec2-user" ]; then
    echo -e "${RED}FAILED: expected 'ec2-user', got '$USER_OUT'${NC}"
    exit 1
fi
$BIN exec "$TEST_VM" -- cat /etc/os-release | grep -q "Amazon Linux"
echo -e "${GREEN}PASSED${NC}"

# Test 7: Stop, snapshot, delete and purge
echo -n "[7/7] Testing stop, snapshot, delete and purge lifecycle... "
$BIN stop "$TEST_VM" >/dev/null
$BIN snapshot "$TEST_VM" --name snap-e2e --comment "ci test" >/dev/null
$BIN list --snapshots | grep -q "snap-e2e"
$BIN delete "$TEST_VM" >/dev/null
$BIN list | grep -q "Deleted"
$BIN purge >/dev/null
if $BIN list | grep -q "$TEST_VM"; then
    echo -e "${RED}FAILED: instance was not purged${NC}"
    exit 1
fi
echo -e "${GREEN}PASSED${NC}"

echo -e "\n${GREEN}ALL 7 E2E INTEGRATION TESTS PASSED SUCCESSFULLY!${NC}"
