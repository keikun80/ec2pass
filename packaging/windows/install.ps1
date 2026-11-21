# PowerShell Installer for InhausLab ec2pass on Windows
param (
    [string]$InstallDir = "",
    [switch]$Silent = $false
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

# 2. Check Hyper-V & Virtualization Prerequisite
Write-Host "`n[1/5] Checking Windows Virtualization & Hyper-V status..." -ForegroundColor Gray
try {
    $hypervFeature = Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V -ErrorAction SilentlyContinue
    if ($hypervFeature -and $hypervFeature.State -eq "Enabled") {
        Write-Host "  [OK] Hyper-V optional feature is enabled." -ForegroundColor Green
    } else {
        Write-Host "  [NOTICE] Hyper-V is not currently enabled in Windows features." -ForegroundColor DarkYellow
        Write-Host "           You can enable it via: Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V -All" -ForegroundColor DarkYellow
        Write-Host "           ec2pass also supports QEMU and fallback simulation mode automatically." -ForegroundColor DarkYellow
    }
} catch {
    Write-Host "  [INFO] Skipped Hyper-V feature query (non-elevated check)." -ForegroundColor DarkGray
}

# 3. Locate Source Binaries
Write-Host "`n[2/5] Locating binaries..." -ForegroundColor Gray
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

$candidates = @(
    $scriptDir,
    (Join-Path $scriptDir "..\..\target\release"),
    (Join-Path $scriptDir "target\release"),
    (Join-Path (Get-Location) "target\release"),
    (Join-Path (Get-Location) "dist\ec2pass-v1.14.0-windows-x86_64"),
    (Join-Path (Get-Location) "target\debug")
)

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

$filesToCopy = @("ec2pass.exe", "multipass.exe", "ec2passd.exe")
foreach ($f in $filesToCopy) {
    $src = Join-Path $binSource $f
    if (-not (Test-Path $src)) {
        if ($f -eq "multipass.exe" -and (Test-Path (Join-Path $binSource "ec2pass.exe"))) {
            $src = Join-Path $binSource "ec2pass.exe"
        }
    }
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

# 5. Configure Environment PATH
Write-Host "`n[4/5] Configuring environment variables..." -ForegroundColor Gray
$configured = $false
if ($isAdmin) {
    try {
        $machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
        $entries = $machinePath -split ';' | Where-Object { $_ -ne "" }
        if ($entries -notcontains $InstallDir) {
            [Environment]::SetEnvironmentVariable("Path", "$machinePath;$InstallDir", "Machine")
            Write-Host "  Added '$InstallDir' to Machine PATH." -ForegroundColor Green
        }
        $configured = $true
    } catch {
        Write-Host "  [INFO] Machine PATH access restricted; falling back to User PATH." -ForegroundColor DarkGray
    }
}

if (-not $configured) {
    try {
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        $entries = $userPath -split ';' | Where-Object { $_ -ne "" }
        if ($entries -notcontains $InstallDir) {
            [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
            Write-Host "  Added '$InstallDir' to User PATH." -ForegroundColor Green
        } else {
            Write-Host "  '$InstallDir' is already present in User PATH." -ForegroundColor DarkGray
        }
    } catch {
        Write-Warning "Could not update User PATH automatically: $_"
    }
}
$env:Path = "$env:Path;$InstallDir"

# 6. Register Windows Start Menu Shortcut and Add/Remove Programs Registry
Write-Host "`n[5/5] Registering Windows Start Menu and Add/Remove Programs..." -ForegroundColor Gray
try {
    $startMenuPrograms = [Environment]::GetFolderPath([Environment+SpecialFolder]::Programs)
    if ([string]::IsNullOrWhiteSpace($startMenuPrograms)) {
        $startMenuPrograms = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
    }
    if (Test-Path $startMenuPrograms) {
        $shortcutPath = Join-Path $startMenuPrograms "InhausLab ec2pass.lnk"
        $wshShell = New-Object -ComObject WScript.Shell
        $shortcut = $wshShell.CreateShortcut($shortcutPath)
        $shortcut.TargetPath = "powershell.exe"
        $shortcut.Arguments = "-NoExit -Command `"Write-Host 'Welcome to InhausLab ec2pass!' -ForegroundColor Cyan; ec2pass --version; ec2pass find`""
        $shortcut.WorkingDirectory = [Environment]::GetFolderPath([Environment+SpecialFolder]::UserProfile)
        $shortcut.Description = "Launch InhausLab ec2pass CLI"
        $shortcut.Save()
        Write-Host "  Created Start Menu shortcut: $shortcutPath" -ForegroundColor Green
    }
} catch {
    Write-Host "  [INFO] Skipped Start Menu shortcut creation: $_" -ForegroundColor DarkGray
}

# Registry entry for Installed Apps
try {
    $regKeyPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\ec2pass"
    if ($isAdmin) {
        try {
            $regKeyPath = "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\ec2pass"
            if (-not (Test-Path $regKeyPath)) { New-Item -Path $regKeyPath -Force | Out-Null }
        } catch {
            $regKeyPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\ec2pass"
        }
    }
    
    if (-not (Test-Path $regKeyPath)) {
        New-Item -Path $regKeyPath -Force | Out-Null
    }
    Set-ItemProperty -Path $regKeyPath -Name "DisplayName" -Value "InhausLab ec2pass (Amazon Linux 2023 VM Orchestrator)"
    Set-ItemProperty -Path $regKeyPath -Name "DisplayVersion" -Value "1.0.0"
    Set-ItemProperty -Path $regKeyPath -Name "Publisher" -Value "InhausLab"
    Set-ItemProperty -Path $regKeyPath -Name "InstallLocation" -Value $InstallDir
    Set-ItemProperty -Path $regKeyPath -Name "UninstallString" -Value "powershell.exe -ExecutionPolicy Bypass -File `"$InstallDir\uninstall.ps1`""
    Set-ItemProperty -Path $regKeyPath -Name "QuietUninstallString" -Value "powershell.exe -ExecutionPolicy Bypass -File `"$InstallDir\uninstall.ps1`" -Silent"
    Write-Host "  Registered in Windows Installed Apps." -ForegroundColor Green
} catch {
    Write-Host "  [INFO] Skipped registry registration: $_" -ForegroundColor DarkGray
}

Write-Host "`n================================================================" -ForegroundColor Cyan
Write-Host "  Installation Completed Successfully!" -ForegroundColor Green
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "Binaries located at: $InstallDir" -ForegroundColor White
Write-Host "Commands available: ec2pass, multipass, ec2passd" -ForegroundColor Yellow
Write-Host "Try running in a new terminal:" -ForegroundColor White
Write-Host "  ec2pass --version" -ForegroundColor Cyan
Write-Host "  ec2pass find" -ForegroundColor Cyan
Write-Host "  ec2pass launch al2023 --name test-vm" -ForegroundColor Cyan
Write-Host "  ec2pass shell test-vm" -ForegroundColor Cyan
