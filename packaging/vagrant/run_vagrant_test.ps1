# PowerShell Host Automation Runner for Vagrant ec2pass Build & Test
param (
    [switch]$DestroyOnFinish = $false,
    [switch]$KeepRunning = $false,
    [switch]$ForceProvision = $false,
    [string]$Provider = ""
)

$ErrorActionPreference = "Stop"

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  InhausLab ec2pass Vagrant EC2/AL2023 Automation Runner" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan

# Terminate any hung background ruby/vagrant instances to release lock
Get-Process -Name ruby, vagrant -ErrorAction SilentlyContinue | Where-Object { $_.Id -ne $PID } | Stop-Process -Force -ErrorAction SilentlyContinue

# 1. Check Vagrant executable
if (Test-Path "C:\Program Files\Vagrant\bin\vagrant.exe") {
    $env:Path = "C:\Program Files\Vagrant\bin;$env:Path"
}

$vagrantCmd = Get-Command "vagrant" -ErrorAction SilentlyContinue
if (-not $vagrantCmd) {
    Write-Warning "Vagrant is not installed or not in your PATH."
    Write-Host "`nTo install Vagrant on Windows via winget, run:" -ForegroundColor Yellow
    Write-Host "  winget install HashiCorp.Vagrant" -ForegroundColor White
    Write-Host "`nAfter installation, restart PowerShell and re-run this script." -ForegroundColor Yellow
    exit 1
}

Write-Host "Detected Vagrant: $(vagrant --version)" -ForegroundColor Green

# 2. Determine Provider & Elevation Check
$providerArg = @()
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if ($Provider) {
    $providerArg = @("--provider", $Provider)
} elseif ((Get-Command "VBoxManage" -ErrorAction SilentlyContinue) -or (Test-Path "$env:ProgramFiles\Oracle\VirtualBox\VBoxManage.exe")) {
    $providerArg = @("--provider", "virtualbox")
    Write-Host "Target Provider: VirtualBox" -ForegroundColor Gray
} elseif ((Get-Command "vmconnect" -ErrorAction SilentlyContinue) -or (Test-Path "$env:SystemRoot\System32\vmconnect.exe") -or (Get-Service vmms -ErrorAction SilentlyContinue)) {
    $providerArg = @("--provider", "hyperv")
    Write-Host "Target Provider: Hyper-V (Windows Native)" -ForegroundColor Green
    
    if (-not $isAdmin) {
        Write-Warning "Hyper-V virtualization management requires Windows Administrator privileges."
        Write-Host "`n[Resolution Option 1 - Using Hyper-V (Recommended)]" -ForegroundColor Yellow
        Write-Host "  Right-click 'PowerShell' in the Start Menu, select 'Run as Administrator', then run:" -ForegroundColor White
        Write-Host "    cd c:\ec2pass" -ForegroundColor Cyan
        Write-Host "    powershell -ExecutionPolicy Bypass -File packaging\vagrant\run_vagrant_test.ps1" -ForegroundColor Cyan
        Write-Host "`n[Resolution Option 2 - Running as standard user]" -ForegroundColor Yellow
        Write-Host "  Install VirtualBox to run from a standard user terminal:" -ForegroundColor White
        Write-Host "    winget install Oracle.VirtualBox" -ForegroundColor White
        exit 1
    }
}

# 3. Boot Vagrant VM
Write-Host "`n[1/4] Booting and provisioning Vagrant VM..." -ForegroundColor Yellow
$upArgs = @("up") + $providerArg
if ($ForceProvision) {
    $upArgs += "--provision"
}

& vagrant $upArgs
if ($LASTEXITCODE -ne 0) {
    Write-Error "Vagrant up failed with exit code $LASTEXITCODE."
    exit $LASTEXITCODE
}

# 4. Sync source code to guest VM via vagrant upload (No Windows password or SMB required!)
Write-Host "`n[2/4] Uploading source code to guest VM via SSH (passwordless)..." -ForegroundColor Yellow
$tempTar = Join-Path $env:TEMP "ec2pass_src.tar.gz"
if (Test-Path $tempTar) { Remove-Item -Path $tempTar -Force }

& tar --exclude "target" --exclude "dist" --exclude ".git" --exclude ".vagrant" -czf $tempTar .
& vagrant upload $tempTar "/tmp/ec2pass_src.tar.gz"
Remove-Item -Path $tempTar -Force -ErrorAction SilentlyContinue

& vagrant ssh -c "sudo mkdir -p /vagrant && sudo chown -R vagrant:vagrant /vagrant && tar -xzf /tmp/ec2pass_src.tar.gz -C /vagrant && rm -f /tmp/ec2pass_src.tar.gz"
Write-Host "  Source code uploaded and unpacked to /vagrant successfully." -ForegroundColor Green

# 5. Execute Build & E2E Test Suite inside Guest
Write-Host "`n[3/4] Executing build_and_test.sh inside guest VM..." -ForegroundColor Yellow
& vagrant ssh -c "bash /vagrant/packaging/vagrant/build_and_test.sh"
$testExitCode = $LASTEXITCODE

# 6. Teardown or State Management
Write-Host "`n[4/4] Managing VM state..." -ForegroundColor Yellow
if ($DestroyOnFinish) {
    Write-Host "Destroying ephemeral test VM..." -ForegroundColor DarkGray
    & vagrant destroy -f
} elseif (-not $KeepRunning) {
    Write-Host "Halting test VM to conserve host resources..." -ForegroundColor DarkGray
    & vagrant halt
} else {
    Write-Host "Leaving VM running for interactive SSH debugging (use: vagrant ssh)." -ForegroundColor Green
}

Write-Host "`n================================================================" -ForegroundColor Cyan
if ($testExitCode -eq 0) {
    Write-Host "  SUCCESS: All ec2pass Linux builds and E2E tests PASSED!" -ForegroundColor Green
} else {
    Write-Host "  FAILED: Build or tests encountered errors (Code: $testExitCode)." -ForegroundColor Red
}
Write-Host "================================================================" -ForegroundColor Cyan

exit $testExitCode
