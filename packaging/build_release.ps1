# Local Release Packager for Windows (No GitHub Actions required)
param (
    [string]$version = ""
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

if (-not $version) {
    if (Test-Path "Cargo.toml") {
        $matched = Select-String -Path "Cargo.toml" -Pattern '^version\s*=\s*"([^"]+)"'
        if ($matched) {
            $version = $matched.Matches[0].Groups[1].Value
        }
    }
    if (-not $version) { $version = "0.1.0" }
}
$version = $version.TrimStart('v')
$os = "windows"
$arch = "x86_64"

Write-Host "=== Building ec2pass release for $os-$arch ===" -ForegroundColor Cyan

# 1. Compile release binaries
cargo build --workspace --release

# 2. Run test verification
cargo test --workspace

# 3. Create dist archive
$distDir = "dist\ec2pass-$version-$os-$arch"
$archiveName = "dist\ec2pass-$version-$os-$arch.zip"

if (Test-Path $distDir) { Remove-Item -Path $distDir -Recurse -Force }
if (Test-Path $archiveName) { Remove-Item -Path $archiveName -Force }
New-Item -ItemType Directory -Path $distDir -Force | Out-Null

Copy-Item target\release\ec2pass.exe "$distDir\"
Copy-Item target\release\ec2passd.exe "$distDir\"
Copy-Item packaging\windows\install.ps1 "$distDir\"
Copy-Item packaging\windows\Setup.bat "$distDir\"
Copy-Item packaging\windows\uninstall.ps1 "$distDir\"
Copy-Item packaging\windows\Uninstall.bat "$distDir\"
Copy-Item README.md, MANUAL.md, BUILD_AND_RELEASE.md, platform.md, USERDATA.md -Destination "$distDir\"

Compress-Archive -Path "$distDir" -DestinationPath "$archiveName" -Force

# 4. Generate SHA-256 Checksum
$hash = (Get-FileHash -Path $archiveName -Algorithm SHA256).Hash.ToLower()
"$hash  $(Split-Path $archiveName -Leaf)" | Out-File -FilePath "$archiveName.sha256" -Encoding ascii

Write-Host "Created release package: $archiveName" -ForegroundColor Green
Write-Host "Checksum: $hash" -ForegroundColor Green
