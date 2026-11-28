#!/usr/bin/env bash
# End-to-End Automated Test Suite for ec2pass on Linux and macOS
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
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

DAEMON_BIN="${EC2PASSD_BIN:-$(dirname "$BIN")/ec2passd}"
if [ ! -f "$DAEMON_BIN" ] && command -v ec2passd >/dev/null 2>&1; then
    DAEMON_BIN="$(command -v ec2passd)"
fi

# Unaccelerated (TCG) hosts, e.g. nested VMs without VT-x, need far longer to boot.
LAUNCH_TIMEOUT="${E2E_LAUNCH_TIMEOUT:-300}"

# Ensure background daemon is running
DAEMON_PID=""
cleanup() {
    if [ -n "$DAEMON_PID" ]; then
        kill "$DAEMON_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT

if ! $BIN networks >/dev/null 2>&1; then
    echo -e "Starting background daemon (${CYAN}$DAEMON_BIN${NC})..."
    export EC2PASS_ALLOW_UNACCELERATED=1
    "$DAEMON_BIN" >/tmp/ec2passd_e2e.log 2>&1 &
    DAEMON_PID=$!
    sleep 2
fi

echo -e "${CYAN}=== Running ec2pass End-to-End Verification Suite ===${NC}"

# Test 1: Version check
echo -n "[1/8] Testing version command... "
$BIN --version >/dev/null
echo -e "${GREEN}PASSED${NC}"

# Test 2: Find catalog check (standard catalog and AWS CDN version listing)
echo -n "[2/8] Testing find command and AWS CDN version listing... "
$BIN find >/dev/null
$BIN find minimal --format json >/dev/null
$BIN find --show-versions >/dev/null
$BIN find 2023.12 --show-versions --format json >/dev/null
echo -e "${GREEN}PASSED${NC}"

# Test 3: Networks list
echo -n "[3/8] Testing networks command... "
$BIN networks >/dev/null
echo -e "${GREEN}PASSED${NC}"

# Test 4: Launch instance
TEST_VM="e2e-ci-$(date +%s)"
if command -v qemu-system-x86_64 >/dev/null 2>&1 || command -v qemu-system-aarch64 >/dev/null 2>&1 || command -v qemu-kvm >/dev/null 2>&1 || [ -x /usr/libexec/qemu-kvm ]; then
    echo -n "[4/8] Launching test instance '${TEST_VM}'... "
    $BIN launch al2023 -n "$TEST_VM" --cpus 1 --memory 1G --disk 25G --timeout "$LAUNCH_TIMEOUT" >/dev/null
    echo -e "${GREEN}PASSED${NC}"

    # Test 5: Verify list and info
    echo -n "[5/8] Verifying list and info... "
    $BIN list | grep -q "$TEST_VM"
    $BIN info "$TEST_VM" | grep -q "Running"
    echo -e "${GREEN}PASSED${NC}"

    # Test 6: Execute commands inside VM
    echo -n "[6/8] Executing commands (whoami, os-release)... "
    USER_OUT=$($BIN exec "$TEST_VM" -- whoami | tr -d '\r\n')
    if [ "$USER_OUT" != "ec2-user" ]; then
        echo -e "${RED}FAILED: expected 'ec2-user', got '$USER_OUT'${NC}"
        exit 1
    fi
    $BIN exec "$TEST_VM" -- cat /etc/os-release | grep -q "Amazon Linux"
    echo -e "${GREEN}PASSED${NC}"

    # Test 7: user-data debugging. A failing script must not fail the launch; the report,
    # trace, guest log and bundle must still be produced.
    echo -n "[7/8] Testing user-data debug report, --trace and logs... "
    DBG_VM="${TEST_VM}-dbg"
    UD_FILE="$(mktemp)"
    printf '#!/bin/bash\nstep() { echo "e2e-step-$1"; }\nstep one\nexit 3\n' >"$UD_FILE"
    e2e_fail() { echo -e "${RED}FAILED: $1${NC}"; exit 1; }
    LAUNCH_OUT=$($BIN launch al2023 -n "$DBG_VM" --cpus 1 --memory 1G --disk 25G --timeout "$LAUNCH_TIMEOUT" \
        --cloud-init "$UD_FILE" --trace 2>&1) || e2e_fail "launch must succeed when user-data fails"
    rm -f "$UD_FILE"
    [[ "$LAUNCH_OUT" == *"exited with code 3"* ]] || e2e_fail "launch report lacks the exit code: $LAUNCH_OUT"
    [[ "$LAUNCH_OUT" == *"debug bundle:"* ]] || e2e_fail "launch did not save a debug bundle"
    OUT=$($BIN logs "$DBG_VM" --cloud-init 2>&1)
    [[ "$OUT" == *"e2e-step-one"* ]] || e2e_fail "logs --cloud-init lacks script output: $OUT"
    OUT=$($BIN logs "$DBG_VM" --trace 2>&1)
    [[ "$OUT" == *"step one"* ]] || e2e_fail "logs --trace lacks the traced line: $OUT"
    OUT=$($BIN logs "$DBG_VM" --save 2>&1)
    [[ "$OUT" == *"Debug bundle saved"* ]] || e2e_fail "logs --save failed: $OUT"
    $BIN logs "$DBG_VM" >/dev/null || e2e_fail "logs (console) failed"
    $BIN delete "$DBG_VM" --purge >/dev/null
    echo -e "${GREEN}PASSED${NC}"

    # Test 8: Stop, snapshot, delete and purge
    echo -n "[8/8] Testing stop, snapshot, delete and purge lifecycle... "
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

    echo -e "\n${GREEN}ALL 8 E2E INTEGRATION TESTS PASSED SUCCESSFULLY!${NC}"
else
    echo -e "[4-7/8] ${YELLOW}Note: qemu-system-x86_64 not present in this lightweight build environment; full VM launch lifecycle skipped.${NC}"
    echo -e "        ${GREEN}CLI commands, remote AWS catalog, network queries, and protocol RPCs verified successfully!${NC}"
fi
