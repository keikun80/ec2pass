@echo off
setlocal
title InhausLab ec2pass Uninstaller
echo ================================================================
echo   InhausLab ec2pass Windows Uninstaller
echo ================================================================
echo.

powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0uninstall.ps1" %*

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Uninstallation encountered an error.
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo [SUCCESS] Uninstallation completed!
echo.
pause
