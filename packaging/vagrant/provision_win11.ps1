# PowerShell Provisioning Script for ec2pass on Windows 11 Vagrant Guest
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  InhausLab ec2pass Windows 11 Provisioning Script              " -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan

# 1. Enable Unrestricted Script Execution
Write-Host "`n[1/4] Configuring PowerShell Execution Policy..." -ForegroundColor Yellow
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope LocalMachine -Force

# 2. Check / Install Git
Write-Host "`n[2/4] Verifying Git..." -ForegroundColor Yellow
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Git via winget..." -ForegroundColor Gray
    try {
        winget install --id Git.Git -e --source winget --accept-package-agreements --accept-source-agreements --silent
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
    } catch {
        Write-Warning "winget Git install skipped or unavailable: $($_.Exception.Message)"
    }
}
if (Get-Command git -ErrorAction SilentlyContinue) {
    Write-Host "  [OK] Git detected: $(git --version)" -ForegroundColor Green
}

# 3. Check / Install Rust Toolchain
Write-Host "`n[3/4] Verifying Rust toolchain..." -ForegroundColor Yellow
$cargoBin = "$env:USERPROFILE\.cargo\bin"
if (Test-Path $cargoBin) {
    $env:Path = "$cargoBin;$env:Path"
}

if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "Downloading and installing rustup..." -ForegroundColor Gray
    $rustupInit = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupInit
    & $rustupInit -y --default-toolchain stable-x86_64-pc-windows-msvc
    Remove-Item -Force $rustupInit -ErrorAction SilentlyContinue
    $env:Path = "$cargoBin;$env:Path"
}

if (Get-Command rustc -ErrorAction SilentlyContinue) {
    Write-Host "  [OK] Rustc: $(rustc --version)" -ForegroundColor Green
    Write-Host "  [OK] Cargo: $(cargo --version)" -ForegroundColor Green
} else {
    Write-Warning "Rust toolchain not automatically found in PATH. Please verify user environment."
}

# 4. Prepare Staging Directories
Write-Host "`n[4/4] Preparing directories..." -ForegroundColor Yellow
New-Item -ItemType Directory -Path "C:\vagrant" -Force | Out-Null
New-Item -ItemType Directory -Path "C:\ec2pass_target" -Force | Out-Null

Write-Host "`n================================================================" -ForegroundColor Green
Write-Host "  Windows 11 Provisioning Complete!                             " -ForegroundColor Green
Write-Host "================================================================" -ForegroundColor Green
