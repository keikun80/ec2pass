# Clean build artifacts, temporary caches, and runtime data before Git commit
$ErrorActionPreference = "Continue"

Write-Host "=== Cleaning build artifacts and runtime data for Git commit ===" -ForegroundColor Cyan

# 1. Stop any background daemons to release file locks
Get-Process -Name ec2passd, mpd, alpassd -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

# 2. Cargo clean
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    cargo clean
}

# 3. Remove target, dist, and data directories
$dirsToRemove = @("target", "dist", "data", ".ec2pass-data", ".ec2ps-data", ".data")
foreach ($d in $dirsToRemove) {
    if (Test-Path $d) {
        Remove-Item -Path $d -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "Removed: $d" -ForegroundColor Green
    }
}

# 4. Remove temporary generated files
$filesToRemove = @(
    "packaging\windows\ec2pass_generated.sed",
    "packaging\windows\ec2ps_generated.sed",
    "packaging\windows\alpass_generated.sed"
)
foreach ($f in $filesToRemove) {
    if (Test-Path $f) {
        Remove-Item -Path $f -Force -ErrorAction SilentlyContinue
        Write-Host "Removed: $f" -ForegroundColor Green
    }
}

# Remove stray log and zip files in root
Get-ChildItem -Path . -Filter "*.log" -ErrorAction SilentlyContinue | Remove-Item -Force
Get-ChildItem -Path . -Filter "*.zip" -ErrorAction SilentlyContinue | Remove-Item -Force
Get-ChildItem -Path . -Filter "*.sha256" -ErrorAction SilentlyContinue | Remove-Item -Force

Write-Host "Working directory is clean and ready for Git commit!" -ForegroundColor Cyan
