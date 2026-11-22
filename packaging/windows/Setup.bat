@echo off
setlocal
title InhausLab ec2pass Installer
echo ================================================================
echo   InhausLab ec2pass Windows Installer
echo ================================================================
echo.

powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1" %*

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Installation encountered an error.
    if /i not "%~1"=="-Silent" pause
    exit /b %ERRORLEVEL%
)

echo.
echo [SUCCESS] Installation completed!
echo.
if /i not "%~1"=="-Silent" pause
