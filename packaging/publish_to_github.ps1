# ==============================================================================
# InhausLab ec2pass - GitHub Publish & Release Script
# ==============================================================================
[CmdletBinding()]
param(
    [string]$RepoUrl = "https://github.com/keikun80/ec2pass.git",
    [string]$TagName = "0.1.1",
    [string]$ReleaseTitle = "InhausLab ec2pass 0.1.1",
    [string]$ReleaseNotes = "Official release 0.1.1 of InhausLab ec2pass (Amazon Linux 2023 EC2 VM manager for Windows x86_64)."
)

$ErrorActionPreference = "Continue"
$WorkspaceDir = (Get-Item -Path $PSScriptRoot\..).FullName
Set-Location $WorkspaceDir

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  InhausLab ec2pass GitHub Publishing & Release Automation" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "Workspace: $WorkspaceDir"
Write-Host "Remote:    $RepoUrl"
Write-Host "Tag:       $TagName"

# 1. Locate git and gh
$gitCandidates = @(
    "C:\Users\kokid\AppData\Local\Microsoft\WinGet\Packages\Git.MinGit_Microsoft.Winget.Source_8wekyb3d8bbwe\cmd\git.exe",
    "C:\Program Files\Git\cmd\git.exe"
)
$gitExe = $null
foreach ($cand in $gitCandidates) {
    if (Test-Path $cand) {
        $gitExe = $cand
        break
    }
}
if (-not $gitExe) {
    $found = Get-Command git -ErrorAction SilentlyContinue
    if ($found) { $gitExe = $found.Source }
}

$ghCandidates = @(
    "C:\Program Files\GitHub CLI\gh.exe"
)
$ghExe = $null
foreach ($cand in $ghCandidates) {
    if (Test-Path $cand) {
        $ghExe = $cand
        break
    }
}
if (-not $ghExe) {
    $found = Get-Command gh -ErrorAction SilentlyContinue
    if ($found) { $ghExe = $found.Source }
}

if (-not $gitExe) { throw "git.exe not found!" }
if (-not $ghExe) { throw "gh.exe not found!" }

$gitDir = Split-Path $gitExe -Parent
$ghDir = Split-Path $ghExe -Parent
$env:Path = "$gitDir;$ghDir;$env:Path"

Write-Host "Using Git: $gitExe" -ForegroundColor Green
Write-Host "Using GH:  $ghExe" -ForegroundColor Green

# 2. Configure Git auth helper via gh
Write-Host "[1/6] Configuring Git credential helper via gh..." -ForegroundColor Yellow
& $ghExe auth setup-git

# 3. Check / Initialize Git repository
Write-Host "[2/6] Checking Git repository initialization..." -ForegroundColor Yellow
if (-not (Test-Path "$WorkspaceDir\.git")) {
    & $gitExe init -b main
    Write-Host "Initialized empty Git repository with branch main." -ForegroundColor Green
} else {
    & $gitExe branch -M main
}

# Ensure Git user config is set
$gitUser = & $gitExe config user.name
$gitEmail = & $gitExe config user.email
if (-not $gitUser) {
    & $gitExe config user.name "keikun80"
}
if (-not $gitEmail) {
    & $gitExe config user.email "keikun80@users.noreply.github.com"
}

# Ensure remote origin is set
$remotes = & $gitExe remote
if ($remotes -notcontains "origin") {
    & $gitExe remote add origin $RepoUrl
    Write-Host "Added remote origin: $RepoUrl" -ForegroundColor Green
} else {
    & $gitExe remote set-url origin $RepoUrl
    Write-Host "Updated remote origin to: $RepoUrl" -ForegroundColor Green
}

# 4. Stage files & Commit
Write-Host "[3/6] Staging files and creating commit..." -ForegroundColor Yellow
& $gitExe add .

# Check if there is anything to commit
$status = & $gitExe status --porcelain
if ($status) {
    & $gitExe commit -m "feat: release $TagName of InhausLab ec2pass"
    Write-Host "Created commit for $TagName." -ForegroundColor Green
} else {
    Write-Host "Working tree clean, no new changes to commit." -ForegroundColor Cyan
}

# 5. Tag release
Write-Host "[4/6] Creating tag $TagName..." -ForegroundColor Yellow
$tagList = & $gitExe tag -l $TagName
if (-not $tagList) {
    & $gitExe tag -a $TagName -m "Release $TagName - InhausLab ec2pass"
    Write-Host "Created tag $TagName." -ForegroundColor Green
} else {
    Write-Host "Tag $TagName already exists." -ForegroundColor Cyan
}

# 6. Push to remote
Write-Host "[5/6] Pushing main branch and tag $TagName to GitHub..." -ForegroundColor Yellow
& $gitExe push -u origin main --force
& $gitExe push origin $TagName --force
Write-Host "Pushed to GitHub successfully!" -ForegroundColor Green

# 7. Create GitHub Release and upload distribution assets
Write-Host "[6/6] Creating GitHub Release and uploading artifacts..." -ForegroundColor Yellow
$setupExe = "$WorkspaceDir\dist\ec2pass-setup.exe"
$setupSha = "$WorkspaceDir\dist\ec2pass-setup.exe.sha256"
$zipPkg   = "$WorkspaceDir\dist\ec2pass-v1.0.0-windows-x86_64.zip"
$zipSha   = "$WorkspaceDir\dist\ec2pass-v1.0.0-windows-x86_64.zip.sha256"

$assets = @()
if (Test-Path $setupExe) { $assets += $setupExe }
if (Test-Path $setupSha) { $assets += $setupSha }
if (Test-Path $zipPkg)   { $assets += $zipPkg }
if (Test-Path $zipSha)   { $assets += $zipSha }

# Check if release exists
$releaseList = & $ghExe release list --repo keikun80/ec2pass
$releaseExists = ($releaseList -match [regex]::Escape($TagName))
if (-not $releaseExists) {
    & $ghExe release create $TagName $assets `
        --repo keikun80/ec2pass `
        --title $ReleaseTitle `
        --notes $ReleaseNotes
    Write-Host "GitHub Release $TagName created with assets:" -ForegroundColor Green
} else {
    Write-Host "GitHub Release $TagName already exists. Uploading/updating assets..." -ForegroundColor Yellow
    & $ghExe release upload $TagName $assets --repo keikun80/ec2pass --clobber
    Write-Host "GitHub Release assets updated." -ForegroundColor Green
}

foreach ($asset in $assets) {
    Write-Host "  Uploaded: $(Split-Path $asset -Leaf)" -ForegroundColor Cyan
}

Write-Host "================================================================" -ForegroundColor Green
Write-Host "  Publish & Release $TagName Complete!" -ForegroundColor Green
Write-Host "  Repository URL: https://github.com/keikun80/ec2pass" -ForegroundColor Green
Write-Host "  Release URL:    https://github.com/keikun80/ec2pass/releases/tag/$TagName" -ForegroundColor Green
Write-Host "================================================================" -ForegroundColor Green
