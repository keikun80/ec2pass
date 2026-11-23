#!/usr/bin/env bash
# ==============================================================================
# InhausLab ec2pass - Comprehensive Subcommand & Option Matrix Test Runner
# Tests all subcommands, flags, and format combinations with output capture.
# ==============================================================================
set -u

FAILED_TESTS=0

BIN="${EC2PASS_BIN:-target/release/ec2pass}"
REPORT_FILE="${REPORT_FILE:-/tmp/ec2pass_matrix_capture.md}"
PLATFORM_NAME="${PLATFORM_NAME:-macOS Darwin (x86_64, HVF)}"

if [ ! -e /dev/kvm ]; then
    export EC2PASS_ALLOW_UNACCELERATED=1
fi

mkdir -p "$(dirname "$REPORT_FILE")"
mkdir -p /tmp/matrix_share
echo "ec2pass-matrix-shared-file-v1.0" > /tmp/matrix_share/shared_test.txt
echo "ec2pass-payload-transfer-content" > /tmp/matrix_payload.txt

cat << 'EOF' > /tmp/matrix_cloud_init.yaml
#cloud-config
write_files:
  - path: /etc/matrix-ci-test.conf
    content: |
      [matrix]
      status=validated
      platform=auto
    permissions: '0644'
runcmd:
  - echo "Matrix cloud-init test executed" > /tmp/ci-run.log
EOF

exec > >(tee -a "$REPORT_FILE") 2>&1

echo "# ec2pass Subcommand & Option Test Matrix Capture: ${PLATFORM_NAME}"
echo "Execution Timestamp: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo "Binary: \`$($BIN --version 2>/dev/null || echo "$BIN")\`"
echo ""

run_test() {
    local TITLE="$1"
    local CMD="$2"
    echo "### ${TITLE}"
    echo "**Command**: \`${CMD}\`"
    echo '```text'
    eval "${CMD}"
    local STATUS=$?
    echo '```'
    echo "**Exit Code**: \`${STATUS}\`"
    echo ""

    if [ "$STATUS" -ne 0 ]; then
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

echo "## 1. System & Discovery Commands"
echo ""
run_test "1.1 Client & Daemon Version" "$BIN version"
run_test "1.2 Available Network Interfaces" "$BIN networks"
run_test "1.3 Find Images (Default Table)" "$BIN find"
run_test "1.4 Find Specific Image (al2)" "$BIN find al2"
run_test "1.5 Find with All AWS CDN Versions" "$BIN find --show-versions"
run_test "1.6 Find Output Format: JSON" "$BIN find --format json"
run_test "1.7 Find Output Format: YAML" "$BIN find --format yaml"
run_test "1.8 Find Output Format: CSV" "$BIN find --format csv"
run_test "1.9 Find Versions Filtered by 2023.12 (JSON)" "$BIN find 2023.12 --show-versions --format json"

echo "## 2. Launching & Creation Combinations"
echo ""
run_test "2.1 Launch Standard Instance (Auto-Name Petname)" "$BIN launch al2023 -n matrix-std"
run_test "2.2 Launch with Custom Specs (CPUs, Memory, Disk)" "$BIN launch al2023 -n matrix-specs -c 2 -m 1G -d 26G"
run_test "2.3 Launch with Cloud-Init User-Data YAML" "$BIN launch al2023 -n matrix-ci --cloud-init /tmp/matrix_cloud_init.yaml"
run_test "2.4 Launch with Compatibility Alias (al2023-k6.12)" "$BIN launch al2023-k6.12 -n matrix-compat"

echo "## 3. Listing & Detailed Instance Information"
echo ""
run_test "3.1 List Instances (Default Table)" "$BIN list"
run_test "3.2 List Instances (JSON Format)" "$BIN list --format json"
run_test "3.3 List Instances (YAML Format)" "$BIN list --format yaml"
run_test "3.4 List Instances (CSV Format)" "$BIN list --format csv"
run_test "3.5 Info Single Instance" "$BIN info matrix-std"
run_test "3.6 Info Multiple Instances" "$BIN info matrix-std matrix-specs"
run_test "3.7 Info All Instances (--all)" "$BIN info --all"
run_test "3.8 Info Format JSON" "$BIN info matrix-std --format json"
run_test "3.9 Info Format YAML" "$BIN info matrix-std --format yaml"
run_test "3.10 Info Format CSV" "$BIN info matrix-std --format csv"

echo "## 4. Execution, File Transfer & Directory Mounting"
echo ""
run_test "4.1 Exec Simple Command (whoami)" "$BIN exec matrix-std -- whoami"
run_test "4.2 Exec OS Release & Kernel Architecture" "$BIN exec matrix-std -- uname -a"
run_test "4.3 Exec with Custom Working Directory" "$BIN exec matrix-std --working-directory /etc -- pwd"
run_test "4.4 Exec Verifying Cloud-Init Injected File" "$BIN exec matrix-ci -- cat /etc/matrix-ci-test.conf"
run_test "4.5 Transfer Local File to Instance" "$BIN transfer /tmp/matrix_payload.txt matrix-std:/tmp/payload.txt"
run_test "4.6 Verify Transferred File on Instance" "$BIN exec matrix-std -- cat /tmp/payload.txt"
run_test "4.7 Transfer File from Instance to Host" "$BIN transfer matrix-std:/etc/os-release /tmp/guest_os_release.txt && head -n 5 /tmp/guest_os_release.txt"
run_test "4.8 Mount Host Directory into Instance" "$BIN mount /tmp/matrix_share matrix-std:/mnt/matrix_share"
run_test "4.9 Verify Mounted Directory in Guest" "$BIN exec matrix-std -- ls -la /mnt/matrix_share"
run_test "4.10 Unmount Directory from Instance" "$BIN umount matrix-std:/mnt/matrix_share"

echo "## 5. Snapshot, Restore, Cloning & Lifecycle Control"
echo ""
run_test "5.1 Create Named Snapshot with Comment" "$BIN snapshot matrix-std -n baseline-snap -c 'Baseline clean snapshot'"
run_test "5.2 List Snapshots (--snapshots)" "$BIN list --snapshots"
run_test "5.3 Info Snapshots (--snapshots)" "$BIN info matrix-std --snapshots"
run_test "5.4 Modify Guest State Before Restore" "$BIN exec matrix-std -- sh -c 'echo \"canary\" > /tmp/canary_file.txt && ls -l /tmp/canary_file.txt'"
run_test "5.5 Restore Snapshot" "$BIN restore matrix-std -s baseline-snap"
run_test "5.6 Verify Snapshot Restored State" "$BIN exec matrix-std -- test ! -f /tmp/canary_file.txt && echo 'Canary successfully reverted!'"
run_test "5.7 Stop Single Instance" "$BIN stop matrix-specs"
run_test "5.8 Stop Instance with Timeout (-t 10)" "$BIN stop matrix-ci -t 10"
run_test "5.9 Clone Stopped Instance" "$BIN clone matrix-specs -n matrix-specs-clone"
run_test "5.10 Verify Cloned Instance in List" "$BIN list"
run_test "5.11 Start Stopped Instance" "$BIN start matrix-specs"
run_test "5.12 Restart Running Instance" "$BIN restart matrix-std && sleep 2"
run_test "5.13 Suspend Running Instance" "$BIN suspend matrix-std"
run_test "5.14 Verify Suspended State in List" "$BIN list"
run_test "5.15 Resume Suspended Instance via Start" "$BIN start matrix-std"
run_test "5.16 Verify Resumed State in List" "$BIN list"

echo "## 6. Deletion, Recovery, Purging & Configuration Settings"
echo ""
run_test "6.1 Create Command Alias on Instance" "$BIN alias matrix-std:uptime host-uptime"
run_test "6.2 List Aliases (Table Format)" "$BIN aliases"
run_test "6.3 List Aliases (JSON Format)" "$BIN aliases --format json"
run_test "6.4 Execute Aliased Command" "$BIN host-uptime"
run_test "6.5 Remove Alias (unalias)" "$BIN unalias host-uptime"
run_test "6.6 Verify Empty Aliases List" "$BIN aliases"
run_test "6.7 Soft Delete Instance" "$BIN delete matrix-specs-clone"
run_test "6.8 Verify Deleted State in List" "$BIN list"
run_test "6.9 Recover Soft-Deleted Instance" "$BIN recover matrix-specs-clone"
run_test "6.10 Verify Recovered State in List" "$BIN list"
run_test "6.11 Immediate Purge on Delete (-p / --purge)" "$BIN delete matrix-specs-clone -p"
run_test "6.12 Delete All Remaining Test Instances" "$BIN delete matrix-std matrix-specs matrix-ci matrix-compat"
run_test "6.13 Purge All Deleted Instances Permanently" "$BIN purge"
run_test "6.14 Verify Empty Instance List" "$BIN list"
run_test "6.15 Get All Configuration Settings" "$BIN get"
run_test "6.16 Set Configuration Setting" "$BIN set client.primary-name=primary-test"
run_test "6.17 Get Specific Configuration Setting" "$BIN get client.primary-name"
run_test "6.18 Client Authentication" "$BIN authenticate ''"

echo ""
if [ "$FAILED_TESTS" -ne 0 ]; then
    echo "=== Test Matrix Run Failed: ${FAILED_TESTS} test(s) failed ==="
    exit 1
fi

echo "=== Test Matrix Run Finished Successfully ==="
