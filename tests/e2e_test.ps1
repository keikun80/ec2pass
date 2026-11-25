# End-to-End Automated Test Suite for ec2pass on Windows (PowerShell)
$ErrorActionPreference = "Stop"

$bin = ".\target\release\ec2pass.exe"
if (-not (Test-Path $bin)) {
    $bin = ".\target\debug\ec2pass.exe"
}
$daemonBin = ".\target\release\ec2passd.exe"
if (-not (Test-Path $daemonBin)) {
    $daemonBin = ".\target\debug\ec2passd.exe"
}

if (-not (Test-Path $bin)) {
    Write-Error "ec2pass.exe not found. Run 'cargo build' first."
}

# Unaccelerated (TCG) hosts, e.g. nested VMs without WHPX, need far longer to boot.
$launchTimeout = if ($env:E2E_LAUNCH_TIMEOUT) { $env:E2E_LAUNCH_TIMEOUT } else { "300" }

Write-Host "=== Running ec2pass End-to-End Windows Verification Suite ===" -ForegroundColor Cyan

# Terminate any stale daemons
Get-Process -Name ec2passd, mpd, alpassd -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# Start ec2passd daemon in same session context (Rule 8 compliant)
$dataDir = ".\data"
New-Item -ItemType Directory -Path $dataDir -Force | Out-Null
$daemonProc = Start-Process -FilePath $daemonBin -ArgumentList "--address", "127.0.0.1:50051", "--data-dir", $dataDir -PassThru -NoNewWindow
Start-Sleep -Seconds 1

try {
    # Test 1: Version check
    Write-Host -NoNewline "[1/8] Testing version command... "
    & $bin --version | Out-Null
    Write-Host "PASSED" -ForegroundColor Green

    # Test 2: Find catalog check
    Write-Host -NoNewline "[2/8] Testing find command and AWS CDN version listing... "
    & $bin find | Out-Null
    & $bin find minimal --format json | Out-Null
    & $bin find --show-versions | Out-Null
    & $bin find 2023.12 --show-versions --format json | Out-Null
    Write-Host "PASSED" -ForegroundColor Green

    # Test 3: Networks list
    Write-Host -NoNewline "[3/8] Testing networks command... "
    & $bin networks | Out-Null
    Write-Host "PASSED" -ForegroundColor Green

    # Test 4: Launch instance
    $testVm = "e2e-win-$(Get-Random)"
    Write-Host -NoNewline "[4/8] Launching test instance '$testVm'... "
    & $bin launch al2023 -n $testVm --cpus 1 --memory 1G --disk 25G --timeout $launchTimeout | Out-Null
    Write-Host "PASSED" -ForegroundColor Green

    # Test 5: Verify list and info
    Write-Host -NoNewline "[5/8] Verifying list and info... "
    $listOut = (& $bin list | Out-String)
    if ($listOut -notmatch $testVm) { throw "Instance not found in list" }
    $infoOut = (& $bin info $testVm | Out-String)
    if ($infoOut -notmatch "Running") { throw "Instance state is not Running" }
    Write-Host "PASSED" -ForegroundColor Green

    # Test 6: Execute commands inside VM
    Write-Host -NoNewline "[6/8] Executing commands (whoami, os-release)... "
    $userOut = (& $bin exec $testVm -- whoami | Out-String).Trim()
    if ($userOut -ne "ec2-user") { throw "Expected 'ec2-user', got '$userOut'" }
    $osRelease = (& $bin exec $testVm -- cat /etc/os-release | Out-String)
    if ($osRelease -notmatch "Amazon Linux") { throw "OS release did not match Amazon Linux" }
    Write-Host "PASSED" -ForegroundColor Green

    # Test 7: user-data debugging. A failing script must not fail the launch; the report,
    # trace, guest log and bundle must still be produced.
    Write-Host -NoNewline "[7/8] Testing user-data debug report, --trace and logs... "
    $dbgVm = "$testVm-dbg"
    $udFile = [System.IO.Path]::GetTempFileName()
    # LF line endings: the guest runs this with bash.
    [System.IO.File]::WriteAllText($udFile, "#!/bin/bash`nstep() { echo `"e2e-step-`$1`"; }`nstep one`nexit 3`n")
    $launchOut = (& $bin launch al2023 -n $dbgVm --cpus 1 --memory 1G --disk 25G --timeout $launchTimeout --cloud-init $udFile --trace 2>&1 | Out-String)
    if ($LASTEXITCODE -ne 0) { throw "launch must succeed when user-data fails: $launchOut" }
    Remove-Item $udFile -Force
    if ($launchOut -notmatch "exited with code 3") { throw "launch report lacks the exit code: $launchOut" }
    if ($launchOut -notmatch "debug bundle:") { throw "launch did not save a debug bundle" }
    $out = (& $bin logs $dbgVm --cloud-init 2>&1 | Out-String)
    if ($out -notmatch "e2e-step-one") { throw "logs --cloud-init lacks script output: $out" }
    $out = (& $bin logs $dbgVm --trace 2>&1 | Out-String)
    if ($out -notmatch "step one") { throw "logs --trace lacks the traced line: $out" }
    $out = (& $bin logs $dbgVm --save 2>&1 | Out-String)
    if ($out -notmatch "Debug bundle saved") { throw "logs --save failed: $out" }
    & $bin logs $dbgVm | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "logs (console) failed" }
    & $bin delete $dbgVm --purge | Out-Null
    Write-Host "PASSED" -ForegroundColor Green

    # Test 8: Stop, snapshot, delete, purge
    Write-Host -NoNewline "[8/8] Testing stop, snapshot, delete and purge lifecycle... "
    & $bin stop $testVm | Out-Null
    & $bin snapshot $testVm --name snap-e2e --comment "win test" | Out-Null
    $snapList = (& $bin list --snapshots | Out-String)
    if ($snapList -notmatch "snap-e2e") { throw "Snapshot not found in list" }
    & $bin delete $testVm | Out-Null
    $delList = (& $bin list | Out-String)
    if ($delList -notmatch "Deleted") { throw "Instance state is not Deleted" }
    & $bin purge | Out-Null
    $finalList = (& $bin list | Out-String)
    if ($finalList -match $testVm) { throw "Instance was not purged" }
    Write-Host "PASSED" -ForegroundColor Green

    Write-Host "`nALL 8 E2E INTEGRATION TESTS PASSED SUCCESSFULLY!" -ForegroundColor Green
} finally {
    if ($daemonProc -and -not $daemonProc.HasExited) {
        Stop-Process -Id $daemonProc.Id -Force -ErrorAction SilentlyContinue
    }
}
