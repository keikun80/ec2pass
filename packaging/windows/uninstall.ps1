# PowerShell Uninstaller for InhausLab ec2pass on Windows
param (
    [switch]$Silent = $false,
    [switch]$PurgeData = $false
)

$ErrorActionPreference = "Continue"
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  InhausLab ec2pass Windows Uninstaller" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan

# 1. Terminate daemon processes
Write-Host "[1/5] Stopping running daemons and background tasks..." -ForegroundColor Gray
Get-Process -Name ec2passd, mpd, alpassd -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

# 2. Determine install directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$installDir = $scriptDir

Write-Host "[2/5] Target install directory: $installDir" -ForegroundColor Yellow

# 3. Clean PATH
Write-Host "[3/5] Cleaning environment variables (PATH)..." -ForegroundColor Gray
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if ($isAdmin) {
    try {
        $machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
        if ($machinePath) {
            $filtered = ($machinePath -split ';' | Where-Object { $_ -ne "" -and $_ -ne $installDir }) -join ';'
            [Environment]::SetEnvironmentVariable("Path", $filtered, "Machine")
            Write-Host "  Removed '$installDir' from Machine PATH." -ForegroundColor Green
        }
    } catch {
        # Fall back to user path
    }
}

try {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath) {
        $filtered = ($userPath -split ';' | Where-Object { $_ -ne "" -and $_ -ne $installDir }) -join ';'
        [Environment]::SetEnvironmentVariable("Path", $filtered, "User")
        Write-Host "  Removed '$installDir' from User PATH." -ForegroundColor Green
    }
} catch {
    # Ignored
}

# 4. Remove Start Menu Shortcut
Write-Host "[4/5] Removing Start Menu shortcuts and Registry entries..." -ForegroundColor Gray
try {
    $startMenuPrograms = [Environment]::GetFolderPath([Environment+SpecialFolder]::Programs)
    if ([string]::IsNullOrWhiteSpace($startMenuPrograms)) {
        $startMenuPrograms = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
    }
    if (Test-Path $startMenuPrograms) {
        $shortcuts = @(
            "InhausLab ec2pass.lnk",
            "Amazon Linux Multipass (ec2pass).lnk",
            "Amazon Linux Multipass (mp).lnk",
            "Amazon Linux Multipass (alpass).lnk"
        )
        foreach ($s in $shortcuts) {
            $shortcutPath = Join-Path $startMenuPrograms $s
            if (Test-Path $shortcutPath) {
                Remove-Item -Path $shortcutPath -Force -ErrorAction SilentlyContinue
                Write-Host "  Removed Start Menu shortcut: $s" -ForegroundColor Green
            }
        }
    }
} catch {
    # Ignored
}

# 5. Remove Registry Entries (Windows Installed Apps / Control Panel)
$regPaths = @(
    "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\ec2pass",
    "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\ec2pass",
    "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\mp",
    "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\mp",
    "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\alpass",
    "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\alpass"
)
foreach ($rp in $regPaths) {
    if (Test-Path $rp) {
        Remove-Item -Path $rp -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "  Removed Windows registry uninstall entry ($rp)." -ForegroundColor Green
    }
}

# 6. Optional: Purge VM instance data & images if requested
if ($PurgeData) {
    Write-Host "[Data Purge] Removing virtual machine images and configuration..." -ForegroundColor DarkYellow
    $dataDirs = @(
        (Join-Path $env:LOCALAPPDATA "ec2pass"),
        (Join-Path $env:USERPROFILE ".ec2pass"),
        (Join-Path $env:LOCALAPPDATA "alpass"),
        (Join-Path $env:USERPROFILE ".alpass")
    )
    foreach ($dd in $dataDirs) {
        if (Test-Path $dd) {
            Remove-Item -Path $dd -Recurse -Force -ErrorAction SilentlyContinue
            Write-Host "  Removed data directory: $dd" -ForegroundColor Green
        }
    }
}

# 7. Self-delete installation directory (with safety check)
Write-Host "[5/5] Removing installed files..." -ForegroundColor Gray
$isSourceRepo = (Test-Path (Join-Path $installDir "Cargo.toml")) -or (Test-Path (Join-Path $installDir ".git"))

if ($isSourceRepo) {
    Write-Host "  [Safety Warning] Target directory appears to be the source repository." -ForegroundColor Yellow
    Write-Host "  Skipping folder deletion to protect source code." -ForegroundColor Yellow
} else {
    # Remove all non-running files immediately
    Get-ChildItem -Path $installDir -Exclude "uninstall.ps1", "Uninstall.bat" -Recurse -ErrorAction SilentlyContinue |
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue

    # Schedule complete directory deletion after script exit
    Start-Process -FilePath "cmd.exe" -ArgumentList "/c timeout /t 2 /nobreak >nul & rmdir /s /q `"$installDir`"" -WindowStyle Hidden
    Write-Host "  Scheduled installation directory removal: $installDir" -ForegroundColor Green
}

Write-Host "`n================================================================" -ForegroundColor Cyan
Write-Host "  InhausLab ec2pass has been successfully uninstalled from this system." -ForegroundColor Green
Write-Host "================================================================" -ForegroundColor Cyan
