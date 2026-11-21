# Local Release Packager for Windows (No GitHub Actions required)
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$version = "1.14.0"
$os = "windows"
$arch = "x86_64"

Write-Host "=== Building ec2pass release for $os-$arch ===" -ForegroundColor Cyan

# 1. Compile release binaries
cargo build --workspace --release

# 2. Run test verification
.\tests\e2e_test.ps1

# 3. Create dist archive
$distDir = "dist\ec2pass-v$version-$os-$arch"
$archiveName = "dist\ec2pass-v$version-$os-$arch.zip"

if (Test-Path $distDir) { Remove-Item -Path $distDir -Recurse -Force }
if (Test-Path $archiveName) { Remove-Item -Path $archiveName -Force }
New-Item -ItemType Directory -Path $distDir -Force | Out-Null

Copy-Item target\release\ec2pass.exe "$distDir\"
Copy-Item target\release\ec2passd.exe "$distDir\"
Copy-Item packaging\windows\install.ps1 "$distDir\"
Copy-Item packaging\windows\Setup.bat "$distDir\"
Copy-Item packaging\windows\uninstall.ps1 "$distDir\"
Copy-Item packaging\windows\Uninstall.bat "$distDir\"
Copy-Item README.md, MANUAL.md "$distDir\"

Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory((Resolve-Path $distDir).Path, (Resolve-Path "dist").Path + "\ec2pass-v$version-$os-$arch.zip", [System.IO.Compression.CompressionLevel]::Optimal, $false)

# 4. Generate SHA-256 Checksum
$hash = (Get-FileHash -Path $archiveName -Algorithm SHA256).Hash.ToLower()
"$hash  $(Split-Path $archiveName -Leaf)" | Out-File -FilePath "$archiveName.sha256" -Encoding ascii

Write-Host "Created release package: $archiveName" -ForegroundColor Green
Write-Host "Checksum: $hash" -ForegroundColor Green
