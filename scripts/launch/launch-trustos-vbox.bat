@echo off
title TrustOS - VirtualBox Launcher
color 0A

echo.
echo  =============================================
echo     TrustOS - Build ^& Launch (VirtualBox)
echo  =============================================
echo.

cd /d "C:\Users\nathan\Documents\Scripts\OSrust"

REM Step 1: Build kernel
echo [1/3] Building kernel (cargo build --release)...
cargo build --release 2>&1 | findstr /R "^error"
if %ERRORLEVEL% NEQ 0 (
    echo [OK] Build successful!
) else (
    echo [ERROR] Build failed! Check errors above.
    pause
    exit /b 1
)

REM Step 2: Stop existing VM if running
echo [2/3] Stopping old VM if running...
"C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" controlvm TRustOs poweroff 2>nul
timeout /t 3 /nobreak >nul

REM Step 3: Run VBox deployment script
echo [3/3] Deploying and launching VM...
powershell -ExecutionPolicy Bypass -File "%~dp0run-vbox.ps1"

echo.
echo TrustOS VBox session launched!
echo Close this window anytime - the VM will keep running.
pause
