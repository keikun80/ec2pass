# ==============================================================================
# InhausLab ec2pass - Comprehensive Subcommand & Option Matrix Test Runner (PowerShell)
# Tests all subcommands, flags, and format combinations with output capture.
# ==============================================================================
param(
    [string]$Bin = $env:EC2PASS_BIN,
    [string]$ReportFile = $env:REPORT_FILE,
    [string]$PlatformName = $env:PLATFORM_NAME
)

$ErrorActionPreference = "Continue"
$failedTests = 0

if (-not $Bin) {
    if (Test-Path ".\target\release\ec2pass.exe") {
        $Bin = ".\target\release\ec2pass.exe"
    } elseif (Test-Path ".\target\release\ec2pass") {
        $Bin = ".\target\release\ec2pass"
    } elseif (Test-Path "C:\ec2pass_target\release\ec2pass.exe") {
        $Bin = "C:\ec2pass_target\release\ec2pass.exe"
    } else {
        $Bin = "ec2pass"
    }
}

if (-not $ReportFile) {
    $ReportFile = "packaging\test_results_windows.md"
}

if (-not $PlatformName) {
    $PlatformName = "Windows 11 Enterprise (x86_64, Hyper-V / Named Pipe)"
}

$reportDir = Split-Path -Parent $ReportFile
if ($reportDir -and -not (Test-Path $reportDir)) {
    New-Item -ItemType Directory -Path $reportDir -Force | Out-Null
}

# Temporary directories and files
$tmpDir = if ($env:TEMP) { $env:TEMP } else { "/tmp" }
$matrixShare = Join-Path $tmpDir "matrix_share"
New-Item -ItemType Directory -Path $matrixShare -Force | Out-Null
"ec2pass-matrix-shared-file-v1.0" | Out-File -FilePath (Join-Path $matrixShare "shared_test.txt") -Encoding utf8
"ec2pass-payload-transfer-content" | Out-File -FilePath (Join-Path $tmpDir "matrix_payload.txt") -Encoding utf8

$cloudInitContent = @"
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
"@
$cloudInitPath = Join-Path $tmpDir "matrix_cloud_init.yaml"
$cloudInitContent | Out-File -FilePath $cloudInitPath -Encoding utf8

# Start report
$nowIso = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
$versionStr = (& $Bin --version 2>&1 | Out-String).Trim()

$lines = [System.Collections.Generic.List[string]]::new()
$lines.Add("# ec2pass Subcommand & Option Test Matrix Capture: $PlatformName")
$lines.Add("Execution Timestamp: $nowIso")
$lines.Add("Binary: " + '`' + $versionStr + '`')
$lines.Add("")

function Run-Test {
    param(
        [string]$Title,
        [string]$CmdStr
    )
    $lines.Add("### $Title")
    $lines.Add("**Command**: " + '`' + $CmdStr + '`')
    $lines.Add('```text')
    
    $out = Invoke-Expression $CmdStr 2>&1 | Out-String
    $statusCode = $LASTEXITCODE
    if ($null -eq $statusCode) { $statusCode = 0 }
    
    $cleanOut = $out.TrimEnd("`r`n")
    if ($cleanOut) {
        $lines.Add($cleanOut)
    }
    $lines.Add('```')
    $lines.Add("**Exit Code**: " + '`' + $statusCode + '`')
    $lines.Add("")
    
    Write-Host "[$statusCode] $Title"
    if ($statusCode -ne 0) {
        $script:failedTests++
    }
}

# Section 1
$lines.Add("## 1. System & Discovery Commands")
$lines.Add("")
Run-Test "1.1 Client & Daemon Version" "$Bin version"
Run-Test "1.2 Available Network Interfaces" "$Bin networks"
Run-Test "1.3 Find Images (Default Table)" "$Bin find"
Run-Test "1.4 Find Specific Image (al2)" "$Bin find al2"
Run-Test "1.5 Find with All AWS CDN Versions" "$Bin find --show-versions"
Run-Test "1.6 Find Output Format: JSON" "$Bin find --format json"
Run-Test "1.7 Find Output Format: YAML" "$Bin find --format yaml"
Run-Test "1.8 Find Output Format: CSV" "$Bin find --format csv"
Run-Test "1.9 Find Versions Filtered by 2023.12 (JSON)" "$Bin find 2023.12 --show-versions --format json"

# Section 2
$lines.Add("## 2. Launching & Creation Combinations")
$lines.Add("")
Run-Test "2.1 Launch Standard Instance (Auto-Name Petname)" "$Bin launch al2023 -n matrix-std"
Run-Test "2.2 Launch with Custom Specs (CPUs, Memory, Disk)" "$Bin launch al2023 -n matrix-specs -c 2 -m 1G -d 26G"
Run-Test "2.3 Launch with Cloud-Init User-Data YAML" "$Bin launch al2023 -n matrix-ci --cloud-init $cloudInitPath"
Run-Test "2.4 Launch with Compatibility Alias (al2023-k6.12)" "$Bin launch al2023-k6.12 -n matrix-compat"

# Section 3
$lines.Add("## 3. Listing & Detailed Instance Information")
$lines.Add("")
Run-Test "3.1 List Instances (Default Table)" "$Bin list"
Run-Test "3.2 List Instances (JSON Format)" "$Bin list --format json"
Run-Test "3.3 List Instances (YAML Format)" "$Bin list --format yaml"
Run-Test "3.4 List Instances (CSV Format)" "$Bin list --format csv"
Run-Test "3.5 Info Single Instance" "$Bin info matrix-std"
Run-Test "3.6 Info Multiple Instances" "$Bin info matrix-std matrix-specs"
Run-Test "3.7 Info All Instances (--all)" "$Bin info --all"
Run-Test "3.8 Info Format JSON" "$Bin info matrix-std --format json"
Run-Test "3.9 Info Format YAML" "$Bin info matrix-std --format yaml"
Run-Test "3.10 Info Format CSV" "$Bin info matrix-std --format csv"

# Section 4
$lines.Add("## 4. Execution, File Transfer & Directory Mounting")
$lines.Add("")
Run-Test "4.1 Exec Simple Command (whoami)" "$Bin exec matrix-std -- whoami"
Run-Test "4.2 Exec OS Release & Kernel Architecture" "$Bin exec matrix-std -- uname -a"
Run-Test "4.3 Exec with Custom Working Directory" "$Bin exec matrix-std --working-directory /etc -- pwd"
Run-Test "4.4 Exec Verifying Cloud-Init Injected File" "$Bin exec matrix-ci -- cat /etc/matrix-ci-test.conf"
Run-Test "4.5 Transfer Local File to Instance" "$Bin transfer (Join-Path $tmpDir 'matrix_payload.txt') matrix-std:/tmp/payload.txt"
Run-Test "4.6 Verify Transferred File on Instance" "$Bin exec matrix-std -- cat /tmp/payload.txt"
Run-Test "4.7 Transfer File from Instance to Host" "$Bin transfer matrix-std:/etc/os-release (Join-Path $tmpDir 'guest_os_release.txt')"
Run-Test "4.8 Mount Host Directory into Instance" "$Bin mount $matrixShare matrix-std:/mnt/matrix_share"
Run-Test "4.9 Verify Mounted Directory in Guest" "$Bin exec matrix-std -- ls -la /mnt/matrix_share"
Run-Test "4.10 Unmount Directory from Instance" "$Bin umount matrix-std:/mnt/matrix_share"

# Section 5
$lines.Add("## 5. Snapshot, Restore, Cloning & Lifecycle Control")
$lines.Add("")
Run-Test "5.1 Create Named Snapshot with Comment" "$Bin snapshot matrix-std -n baseline-snap -c 'Baseline clean snapshot'"
Run-Test "5.2 List Snapshots (--snapshots)" "$Bin list --snapshots"
Run-Test "5.3 Info Snapshots (--snapshots)" "$Bin info matrix-std --snapshots"
Run-Test "5.4 Modify Guest State Before Restore" "$Bin exec matrix-std -- sh -c 'echo canary > /tmp/canary_file.txt && ls -l /tmp/canary_file.txt'"
Run-Test "5.5 Restore Snapshot" "$Bin restore matrix-std -s baseline-snap"
Run-Test "5.6 Verify Snapshot Restored State" "$Bin exec matrix-std -- ls -l /tmp/canary_file.txt"
Run-Test "5.7 Stop Single Instance" "$Bin stop matrix-specs"
Run-Test "5.8 Stop Instance with Timeout (-t 10)" "$Bin stop matrix-ci -t 10"
Run-Test "5.9 Clone Stopped Instance" "$Bin clone matrix-specs -n matrix-specs-clone"
Run-Test "5.10 Verify Cloned Instance in List" "$Bin list"
Run-Test "5.11 Start Stopped Instance" "$Bin start matrix-specs"
Run-Test "5.12 Restart Running Instance" "$Bin restart matrix-std; Start-Sleep -Seconds 2"
Run-Test "5.13 Suspend Running Instance" "$Bin suspend matrix-std"
Run-Test "5.14 Verify Suspended State in List" "$Bin list"
Run-Test "5.15 Resume Suspended Instance via Start" "$Bin start matrix-std"
Run-Test "5.16 Verify Resumed State in List" "$Bin list"

# Section 6
$lines.Add("## 6. Deletion, Recovery, Purging & Configuration Settings")
$lines.Add("")
Run-Test "6.1 Create Command Alias on Instance" "$Bin alias matrix-std:uptime host-uptime"
Run-Test "6.2 List Aliases (Table Format)" "$Bin aliases"
Run-Test "6.3 List Aliases (JSON Format)" "$Bin aliases --format json"
Run-Test "6.4 Execute Aliased Command" "$Bin host-uptime"
Run-Test "6.5 Remove Alias (unalias)" "$Bin unalias host-uptime"
Run-Test "6.6 Verify Empty Aliases List" "$Bin aliases"
Run-Test "6.7 Soft Delete Instance" "$Bin delete matrix-specs-clone"
Run-Test "6.8 Verify Deleted State in List" "$Bin list"
Run-Test "6.9 Recover Soft-Deleted Instance" "$Bin recover matrix-specs-clone"
Run-Test "6.10 Verify Recovered State in List" "$Bin list"
Run-Test "6.11 Immediate Purge on Delete (-p / --purge)" "$Bin delete matrix-specs-clone -p"
Run-Test "6.12 Delete All Remaining Test Instances" "$Bin delete matrix-std matrix-specs matrix-ci matrix-compat"
Run-Test "6.13 Purge All Deleted Instances Permanently" "$Bin purge"
Run-Test "6.14 Verify Empty Instance List" "$Bin list"
Run-Test "6.15 Get All Configuration Settings" "$Bin get"
Run-Test "6.16 Set Configuration Setting" "$Bin set client.primary-name=primary-test"
Run-Test "6.17 Get Specific Configuration Setting" "$Bin get client.primary-name"
Run-Test "6.18 Client Authentication" "$Bin authenticate ''"

$lines.Add("")
if ($failedTests -ne 0) {
    $lines.Add("=== Test Matrix Run Failed: $failedTests test(s) failed ===")
} else {
    $lines.Add("=== Test Matrix Run Finished Successfully ===")
}

$lines | Out-File -FilePath $ReportFile -Encoding utf8
Write-Host "`nTest matrix report written to: $ReportFile" -ForegroundColor Green

if ($failedTests -ne 0) {
    Write-Error "Test matrix failed: $failedTests test(s) failed."
    exit 1
}
