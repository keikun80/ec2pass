# Automated Build, Test, and Packaging Pipeline for ec2pass on Windows 11
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  InhausLab ec2pass Windows 11 Automated Build & Test Pipeline  " -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan

# 1. Ensure Cargo & Rust in PATH
$cargoBin = "$env:USERPROFILE\.cargo\bin"
if (Test-Path $cargoBin) {
    $env:Path = "$cargoBin;$env:Path"
}

# Source directory
$projectDir = "C:\vagrant"
if (-not (Test-Path $projectDir)) {
    $projectDir = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
}
Set-Location $projectDir

# Fast local target directory
$env:CARGO_TARGET_DIR = "C:\ec2pass_target"
New-Item -ItemType Directory -Path $env:CARGO_TARGET_DIR -Force | Out-Null

Write-Host "`n[Step 1/5] Verifying toolchain environment..." -ForegroundColor Yellow
Write-Host "Rustc:   $(rustc --version)"
Write-Host "Cargo:   $(cargo --version)"
Write-Host "Target:  $env:CARGO_TARGET_DIR"

# 2. Compile Release Binaries
Write-Host "`n[Step 2/5] Compiling release binaries with cargo build --release..." -ForegroundColor Yellow
cargo build --workspace --release

# 3. Execute Workspace Unit Tests
Write-Host "`n[Step 3/5] Running Rust unit & component tests..." -ForegroundColor Yellow
cargo test --workspace

# 4. Execute End-to-End Test Suite
Write-Host "`n[Step 4/5] Executing Windows E2E Verification Suite..." -ForegroundColor Yellow
$releaseDir = Join-Path $env:CARGO_TARGET_DIR "release"
$bin = Join-Path $releaseDir "ec2pass.exe"
$daemonBin = Join-Path $releaseDir "ec2passd.exe"

# Copy binaries into project target directory for e2e_test.ps1 compatibility
$targetRelease = ".\target\release"
New-Item -ItemType Directory -Path $targetRelease -Force | Out-Null
Copy-Item -Force $bin "$targetRelease\ec2pass.exe"
Copy-Item -Force $daemonBin "$targetRelease\ec2passd.exe"

powershell -ExecutionPolicy Bypass -File ".\tests\e2e_test.ps1"

# 5. Package Windows 11 Release Archive
Write-Host "`n[Step 5/5] Creating Windows release ZIP archive in dist/..." -ForegroundColor Yellow
$version = "0.1.1"
$distDir = ".\dist"
$stageName = "ec2pass-$version-windows-x86_64"
$stageDir = Join-Path $distDir $stageName

if (Test-Path $stageDir) {
    Remove-Item -Recurse -Force $stageDir
}
New-Item -ItemType Directory -Path $stageDir -Force | Out-Null

Copy-Item -Force $bin "$stageDir\ec2pass.exe"
Copy-Item -Force $daemonBin "$stageDir\ec2passd.exe"
Copy-Item -Force ".\packaging\windows\Setup.bat" "$stageDir\Setup.bat"
Copy-Item -Force ".\packaging\windows\install.ps1" "$stageDir\install.ps1"
Copy-Item -Force ".\packaging\windows\uninstall.ps1" "$stageDir\uninstall.ps1"
Copy-Item -Force ".\README.md" "$stageDir\README.md"
Copy-Item -Force ".\MANUAL.md" "$stageDir\MANUAL.md"
Copy-Item -Force ".\BUILD_AND_RELEASE.md" "$stageDir\BUILD_AND_RELEASE.md"
Copy-Item -Force ".\platform.md" "$stageDir\platform.md"
Copy-Item -Force ".\USERDATA.md" "$stageDir\USERDATA.md"

$zipPath = Join-Path $distDir "$stageName.zip"
if (Test-Path $zipPath) {
    Remove-Item -Force $zipPath
}
Compress-Archive -Path "$stageDir\*" -DestinationPath $zipPath -CompressionLevel Optimal

# Generate SHA256
$hash = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash.ToLower()
$shaPath = "$zipPath.sha256"
"$hash  $stageName.zip" | Out-File -FilePath $shaPath -Encoding ascii

Write-Host "`n================================================================" -ForegroundColor Green
Write-Host "  SUCCESS: Windows 11 Build, Test & Packaging PASSED!           " -ForegroundColor Green
Write-Host "  Package: $zipPath" -ForegroundColor Green
Write-Host "  SHA-256: $hash" -ForegroundColor Green
Write-Host "================================================================" -ForegroundColor Green
