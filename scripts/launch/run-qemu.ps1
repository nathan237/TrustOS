# TRustOs - QEMU Runner (Simple & Reliable)
$ErrorActionPreference = "Stop"

Write-Host "`n=== TRustOs - QEMU ===" -ForegroundColor Cyan

# Find bootimage
$img = "target\x86_64-unknown-none\release\bootimage-trustos_kernel.bin"
if (-not (Test-Path $img)) {
    Write-Host "ERROR: Bootimage not found!" -ForegroundColor Red
    Write-Host "Run: cd kernel; cargo bootimage --release" -ForegroundColor Yellow
    exit 1
}

$size = [math]::Round((Get-Item $img).Length/1KB, 1)
Write-Host "Image: $size KB" -ForegroundColor Green

# QEMU with correct settings for bootloader 0.9
# Using machine=pc (i440fx) and IDE disk - NOT q35 which uses AHCI
Write-Host "Starting QEMU (machine=pc, IDE disk)..." -ForegroundColor Yellow

$qemuArgs = @(
    "-machine", "pc"                      # i440fx chipset (not q35!)
    "-drive", "format=raw,file=$img,if=ide,index=0,media=disk"
    "-display", "gtk"                     # GTK window
    "-vga", "std"                         # Standard VGA
    "-serial", "stdio"                    # Serial to console
    "-no-reboot"                          # Don't reboot on crash
)

$qemuPath = "C:\Program Files\qemu\qemu-system-x86_64.exe"
if (-not (Test-Path $qemuPath)) {
    Write-Host "ERROR: QEMU not found at $qemuPath" -ForegroundColor Red
    exit 1
}

Write-Host "Press Ctrl+C to exit" -ForegroundColor Gray
& $qemuPath @qemuArgs
