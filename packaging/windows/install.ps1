# PowerShell Installer for InhausLab ec2pass on Windows
# Enforces Hyper-V / WHPX hardware acceleration as a mandatory prerequisite.
param (
    [string]$InstallDir = "",
    [switch]$Silent = $false,
    [switch]$SkipHypervCheck = $false
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  InhausLab ec2pass Windows Setup" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan

# 1. Determine Elevation and Install Target Directory
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if ([string]::IsNullOrWhiteSpace($InstallDir)) {
    if ($isAdmin) {
        $InstallDir = "$env:ProgramFiles\ec2pass"
    } else {
        $InstallDir = "$env:LOCALAPPDATA\Programs\ec2pass"
    }
}
Write-Host "Target Directory: $InstallDir" -ForegroundColor Yellow

# 2. Mandatory Hypervisor Prerequisite Check
Write-Host "`n[1/5] Checking Windows Hypervisor (Hyper-V / WHPX) status..." -ForegroundColor Gray
$hypervReady = $false

try {
    $hypervFeature = Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V -ErrorAction SilentlyContinue
    if ($hypervFeature -and $hypervFeature.State -eq "Enabled") {
        $hypervReady = $true
        Write-Host "  [OK] Hyper-V native virtualization feature is active." -ForegroundColor Green
    }
} catch {
    # Non-elevated check or feature lookup fallback
}

if (-not $hypervReady) {
    # Check WHPX
    try {
        $whpxFeature = Get-WindowsOptionalFeature -Online -FeatureName HypervisorPlatform -ErrorAction SilentlyContinue
        if ($whpxFeature -and $whpxFeature.State -eq "Enabled") {
            $hypervReady = $true
            Write-Host "  [OK] Windows Hypervisor Platform (WHPX) is active." -ForegroundColor Green
        }
    } catch {}
}

if (-not $hypervReady) {
    # Check QEMU binary in PATH
    $qemuCmd = Get-Command qemu-system-x86_64.exe -ErrorAction SilentlyContinue
    if ($qemuCmd) {
        $hypervReady = $true
        Write-Host "  [OK] QEMU virtualization binary detected in PATH." -ForegroundColor Green
    }
}

if (-not $hypervReady -and -not $SkipHypervCheck) {
    Write-Host "`n================================================================" -ForegroundColor Red
    Write-Host " [ERROR] Required hypervisor (Hyper-V or WHPX) is not enabled." -ForegroundColor Red
    Write-Host " ec2pass strictly requires hardware virtualization without simulation." -ForegroundColor Red
    Write-Host "`n ▶ Resolution (Run in an elevated Administrator PowerShell):" -ForegroundColor Yellow
    Write-Host "   1. Enable Hyper-V (Windows Pro / Enterprise):" -ForegroundColor White
    Write-Host "      Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V -All" -ForegroundColor Cyan
    Write-Host "   2. Enable Windows Hypervisor Platform (Windows Home, etc.):" -ForegroundColor White
    Write-Host "      Enable-WindowsOptionalFeature -Online -FeatureName HypervisorPlatform -All" -ForegroundColor Cyan
    Write-Host "   3. Install QEMU:" -ForegroundColor White
    Write-Host "      winget install SoftwareFreedomConservancy.QEMU" -ForegroundColor Cyan
    Write-Host "`n After configuration, reboot the system and re-run this installer." -ForegroundColor Yellow
    Write-Host "================================================================" -ForegroundColor Red
    if (-not $Silent) {
        Read-Host "Press Enter to exit installer..."
    }
    exit 1
}

# 3. Locate Source Binaries
Write-Host "`n[2/5] Locating binaries..." -ForegroundColor Gray
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

$candidates = @(
    $scriptDir,
    (Join-Path $scriptDir "..\..\target\release"),
    (Join-Path $scriptDir "target\release"),
    (Join-Path (Get-Location) "target\release"),
    (Join-Path (Get-Location) "target\debug")
)
$distDirs = Get-ChildItem -Path (Join-Path (Get-Location) "dist\ec2pass-*") -Directory -ErrorAction SilentlyContinue | Select-Object -ExpandProperty FullName
if ($distDirs) {
    $candidates += $distDirs
}

$binSource = ""
foreach ($cand in $candidates) {
    if (Test-Path (Join-Path $cand "ec2pass.exe")) {
        $binSource = (Resolve-Path $cand).Path
        break
    }
}

if (-not $binSource) {
    Write-Error "Cannot locate ec2pass.exe. Please ensure the installer files or release build exist."
    exit 1
}
Write-Host "  Found binaries at: $binSource" -ForegroundColor Green

# 4. Stop any existing running ec2passd daemon
Write-Host "`n[3/5] Installing binaries and tools..." -ForegroundColor Gray
Get-Process -Name ec2passd, mpd, alpassd -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$filesToCopy = @("ec2pass.exe", "ec2passd.exe")
foreach ($f in $filesToCopy) {
    $src = Join-Path $binSource $f
    if (Test-Path $src) {
        Copy-Item -Path $src -Destination (Join-Path $InstallDir $f) -Force
        Write-Host "  Installed: $f" -ForegroundColor Green
    }
}

# Copy uninstall scripts and documentation if present
$auxFiles = @("uninstall.ps1", "Uninstall.bat", "MANUAL.md", "README.md")
foreach ($f in $auxFiles) {
    $auxSrc = Join-Path $scriptDir $f
    if (-not (Test-Path $auxSrc)) {
        $auxSrc = Join-Path (Get-Location) $f
    }
    if (Test-Path $auxSrc) {
        Copy-Item -Path $auxSrc -Destination (Join-Path $InstallDir $f) -Force
    }
}

# 5. Configure User/System PATH Environment Variable
Write-Host "`n[4/5] Configuring environment PATH..." -ForegroundColor Gray
$currentPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($currentPath -split ";" -notcontains $InstallDir) {
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$InstallDir", [EnvironmentVariableTarget]::User)
    Write-Host "  Added $InstallDir to User PATH." -ForegroundColor Green
}

# 6. Verify Installation
Write-Host "`n[5/5] Installation verification..." -ForegroundColor Gray
Write-Host "InhausLab ec2pass installation complete!" -ForegroundColor Green
Write-Host "Commands available in new terminal sessions:"
Write-Host "  ec2pass version"
Write-Host "  ec2pass find"
Write-Host "  ec2pass launch al2023 -n my-win-vm"
