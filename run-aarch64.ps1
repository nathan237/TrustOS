# TRustOS — ARM64 (aarch64) QEMU Runner
# Boots TRustOS kernel on qemu-system-aarch64 with Limine UEFI
#
# Usage:  .\run-aarch64.ps1

$ErrorActionPreference = "Continue"

$QEMU = "C:\Program Files\qemu\qemu-system-aarch64.exe"
$FIRMWARE = "C:\Program Files\qemu\share\edk2-aarch64-code.fd"
$KERNEL = "target\aarch64-unknown-none\debug\trustos_kernel"
$LIMINE_EFI = "limine\BOOTAA64.EFI"
$LIMINE_CONF = "limine.conf"
$ISO_DIR = "iso_root_aarch64"

# ── Build kernel ──
Write-Host "════ Building TRustOS for aarch64 ════" -ForegroundColor Cyan
$buildOutput = & cargo build --target aarch64-unknown-none -p trustos_kernel 2>&1
if (-not (Test-Path "target\aarch64-unknown-none\debug\trustos_kernel")) {
    Write-Host "BUILD FAILED" -ForegroundColor Red
    $buildOutput | Write-Host
    exit 1
}
Write-Host "Build OK" -ForegroundColor Green

# ── Create EFI disk image ──
Write-Host "════ Creating EFI boot image ════" -ForegroundColor Cyan

# Clean and recreate iso root
if (Test-Path $ISO_DIR) { Remove-Item -Recurse -Force $ISO_DIR }
New-Item -ItemType Directory -Path "$ISO_DIR\EFI\BOOT" -Force | Out-Null
New-Item -ItemType Directory -Path "$ISO_DIR\boot\limine" -Force | Out-Null

# Copy Limine EFI bootloader
Copy-Item $LIMINE_EFI "$ISO_DIR\EFI\BOOT\BOOTAA64.EFI"

# Copy limine config
Copy-Item $LIMINE_CONF "$ISO_DIR\boot\limine\limine.conf"

# Copy kernel
Copy-Item $KERNEL "$ISO_DIR\boot\trustos_kernel"

Write-Host "ISO root prepared" -ForegroundColor Green

# ── Create FAT32 EFI disk image using PowerShell ──
$IMG = "trustos_aarch64.img"
$IMG_SIZE = 256MB  # 256MB disk

# Create empty disk image
$fs = [System.IO.File]::Create($IMG)
$fs.SetLength($IMG_SIZE)
$fs.Close()

# Format as FAT32 using diskpart is complex, let's use QEMU's built-in -kernel + direct boot instead
# For aarch64 UEFI, we use the pflash firmware + a FAT directory

Write-Host "════ Launching QEMU aarch64 ════" -ForegroundColor Cyan
Write-Host "  Machine: virt (GICv2)" -ForegroundColor DarkGray
Write-Host "  CPU: cortex-a72 (4 cores)" -ForegroundColor DarkGray
Write-Host "  RAM: 512MB" -ForegroundColor DarkGray
Write-Host "  Firmware: EDK2 aarch64" -ForegroundColor DarkGray
Write-Host ""

# Create OVMF_VARS for aarch64
$VARS = "OVMF_VARS_aarch64.fd"
if (-not (Test-Path $VARS)) {
    # Create a 64MB empty vars file
    $fs = [System.IO.File]::Create($VARS)
    $fs.SetLength(64MB)
    $fs.Close()
}

# Let QEMU use the iso_root as a FAT drive (QEMU can mount a directory as FAT)
& $QEMU `
    -machine virt,gic-version=2 `
    -cpu cortex-a72 `
    -smp 4 `
    -m 512M `
    -drive "if=pflash,format=raw,file=$FIRMWARE,readonly=on" `
    -drive "if=pflash,format=raw,file=$VARS" `
    -drive "file=fat:rw:$ISO_DIR,format=raw,media=disk" `
    -device virtio-gpu-pci `
    -device qemu-xhci `
    -device usb-mouse `
    -serial stdio `
    -no-reboot

Write-Host "`n════ QEMU exited ════" -ForegroundColor Yellow
