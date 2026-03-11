# Setup QEMU with Alpine Linux GUI
# Uses WHPX acceleration (works WITH Hyper-V enabled)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  QEMU + Alpine Linux GUI Setup" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check for QEMU
$qemuPath = "C:\Program Files\qemu\qemu-system-x86_64.exe"
if (-not (Test-Path $qemuPath)) {
    Write-Host "[1/4] QEMU not found. Installing..." -ForegroundColor Yellow
    
    # Download QEMU installer
    $qemuUrl = "https://qemu.weilnetz.de/w64/2023/qemu-w64-setup-20231224.exe"
    $qemuInstaller = "$env:TEMP\qemu-setup.exe"
    
    Write-Host "      Downloading QEMU..." -ForegroundColor Gray
    Invoke-WebRequest -Uri $qemuUrl -OutFile $qemuInstaller -UseBasicParsing
    
    Write-Host "      Running installer (please complete the installation)..." -ForegroundColor Gray
    Start-Process -FilePath $qemuInstaller -Wait
    
    if (-not (Test-Path $qemuPath)) {
        Write-Host "ERROR: QEMU installation failed or was cancelled" -ForegroundColor Red
        exit 1
    }
    Write-Host "      QEMU installed successfully" -ForegroundColor Green
} else {
    Write-Host "[1/4] QEMU found at $qemuPath" -ForegroundColor Green
}

# Create VM directory
$vmDir = "C:\Users\nathan\Documents\Scripts\OSrust\qemu-alpine"
if (-not (Test-Path $vmDir)) {
    New-Item -ItemType Directory -Path $vmDir | Out-Null
}
Write-Host "[2/4] VM directory: $vmDir" -ForegroundColor Green

# Download Alpine Linux ISO if not present
$alpineIso = "$vmDir\alpine-virt-3.19.0-x86_64.iso"
if (-not (Test-Path $alpineIso)) {
    Write-Host "[3/4] Downloading Alpine Linux ISO..." -ForegroundColor Yellow
    $alpineUrl = "https://dl-cdn.alpinelinux.org/alpine/v3.19/releases/x86_64/alpine-virt-3.19.0-x86_64.iso"
    Invoke-WebRequest -Uri $alpineUrl -OutFile $alpineIso -UseBasicParsing
    Write-Host "      Downloaded: $alpineIso" -ForegroundColor Green
} else {
    Write-Host "[3/4] Alpine ISO already present" -ForegroundColor Green
}

# Create disk image if not present
$diskImage = "$vmDir\alpine.qcow2"
if (-not (Test-Path $diskImage)) {
    Write-Host "[4/4] Creating 8GB disk image..." -ForegroundColor Yellow
    & "C:\Program Files\qemu\qemu-img.exe" create -f qcow2 $diskImage 8G
    Write-Host "      Created: $diskImage" -ForegroundColor Green
} else {
    Write-Host "[4/4] Disk image already exists" -ForegroundColor Green
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Setup Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "To install Alpine Linux, run:" -ForegroundColor Cyan
Write-Host "  .\run-alpine-install.ps1" -ForegroundColor White
Write-Host ""
Write-Host "After installation, run:" -ForegroundColor Cyan
Write-Host "  .\run-alpine.ps1" -ForegroundColor White
