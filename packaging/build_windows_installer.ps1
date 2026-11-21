# Build Windows Installer (.exe & .zip) for InhausLab ec2pass
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$version = "1.0.0"
$os = "windows"
$arch = "x86_64"

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  Building InhausLab ec2pass Windows Distribution Package" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan

$rootDir = (Resolve-Path ".").Path
$distBase = Join-Path $rootDir "dist"
$stageDir = Join-Path $distBase "ec2pass-v$version-$os-$arch"
$zipPath = Join-Path $distBase "ec2pass-v$version-$os-$arch.zip"
$installerExePath = Join-Path $distBase "ec2pass-setup.exe"
$sedFilePath = Join-Path $rootDir "packaging\windows\ec2pass_generated.sed"

if (-not (Test-Path $distBase)) {
    New-Item -ItemType Directory -Path $distBase -Force | Out-Null
}

# 1. Clean previous staging
if (Test-Path $stageDir) { Remove-Item -Path $stageDir -Recurse -Force }
if (Test-Path $zipPath) { Remove-Item -Path $zipPath -Force }
if (Test-Path $installerExePath) { Remove-Item -Path $installerExePath -Force }

New-Item -ItemType Directory -Path $stageDir -Force | Out-Null

# 2. Check and compile release binaries
Write-Host "Compiling release binaries with cargo build --workspace --release..." -ForegroundColor Cyan
cargo build --workspace --release

$binFiles = @("ec2pass.exe", "multipass.exe", "ec2passd.exe")
foreach ($b in $binFiles) {
    $src = Join-Path $rootDir "target\release\$b"
    if (-not (Test-Path $src)) {
        throw "Binary missing: $src. Please ensure cargo build --workspace --release completed."
    }
    Copy-Item -Path $src -Destination $stageDir -Force
    Write-Host "Staged: $b" -ForegroundColor Green
}

# 3. Copy scripts and documentation
$extraFiles = @(
    "packaging\windows\install.ps1",
    "packaging\windows\Setup.bat",
    "packaging\windows\uninstall.ps1",
    "packaging\windows\Uninstall.bat",
    "README.md",
    "MANUAL.md"
)
foreach ($f in $extraFiles) {
    $src = Join-Path $rootDir $f
    if (Test-Path $src) {
        Copy-Item -Path $src -Destination $stageDir -Force
        Write-Host "Staged: $(Split-Path $f -Leaf)" -ForegroundColor Gray
    }
}

# 4. Generate ZIP archive using System.IO.Compression.ZipFile
Write-Host "`nCompressing ZIP package: $zipPath..." -ForegroundColor Cyan
Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory($stageDir, $zipPath, [System.IO.Compression.CompressionLevel]::Optimal, $false)

$zipHash = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash.ToLower()
"$zipHash  $(Split-Path $zipPath -Leaf)" | Out-File -FilePath "$zipPath.sha256" -Encoding ascii
Write-Host "ZIP Created: $zipPath" -ForegroundColor Green
Write-Host "ZIP SHA256:  $zipHash" -ForegroundColor DarkGray

# 5. Build Standalone Installer EXE via IExpress
Write-Host "`nBuilding Self-Extracting Installer EXE via IExpress..." -ForegroundColor Cyan

# Create clean SED file with absolute paths
$sedContent = @"
[Version]
Class=IEXPRESS
SEDVersion=3
[Options]
PackagePurpose=InstallApp
ShowInstallProgramWindow=0
HideExtractAnimation=0
UseLongFileName=1
InsideCompressed=0
CAB_FixedSize=0
CAB_ResvCodeSigning=0
RebootMode=N
InstallPrompt=Do you want to install InhausLab ec2pass?
DisplayLicense=
FinishMessage=InhausLab ec2pass has been successfully installed!
TargetName=$installerExePath
FriendlyName=InhausLab ec2pass Setup
AppLaunched=cmd.exe /c Setup.bat
PostInstallCmd=<None>
AdminQuietInstCmd=powershell.exe -ExecutionPolicy Bypass -File install.ps1 -Silent
UserQuietInstCmd=powershell.exe -ExecutionPolicy Bypass -File install.ps1 -Silent
SourceFiles=SourceFiles
[Strings]
FILE0=ec2pass.exe
FILE1=multipass.exe
FILE2=ec2passd.exe
FILE3=install.ps1
FILE4=Setup.bat
FILE5=uninstall.ps1
FILE6=Uninstall.bat
FILE7=MANUAL.md
FILE8=README.md
[SourceFiles]
SourceFiles0=$stageDir\
[SourceFiles0]
%FILE0%=
%FILE1%=
%FILE2%=
%FILE3%=
%FILE4%=
%FILE5%=
%FILE6%=
%FILE7%=
%FILE8%=
"@

[System.IO.File]::WriteAllText($sedFilePath, $sedContent, [System.Text.Encoding]::ASCII)

$iexpress = "$env:SystemRoot\System32\iexpress.exe"
if (Test-Path $iexpress) {
    Write-Host "Invoking IExpress: $iexpress /N $sedFilePath" -ForegroundColor Gray
    $proc = Start-Process -FilePath $iexpress -ArgumentList "/N", "/Q", "$sedFilePath" -Wait -PassThru -NoNewWindow
    if ($proc.ExitCode -eq 0 -and (Test-Path $installerExePath)) {
        $exeHash = (Get-FileHash -Path $installerExePath -Algorithm SHA256).Hash.ToLower()
        "$exeHash  $(Split-Path $installerExePath -Leaf)" | Out-File -FilePath "$installerExePath.sha256" -Encoding ascii
        Write-Host "Standalone EXE Installer Created: $installerExePath" -ForegroundColor Green
        Write-Host "Installer SHA256: $exeHash" -ForegroundColor DarkGray
    } else {
        Write-Warning "IExpress build returned code $($proc.ExitCode) or output not found. Will verify alternative."
    }
}

Write-Host "`nWindows Packaging Finished Successfully!" -ForegroundColor Green
