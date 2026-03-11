#!/usr/bin/env pwsh
# ═══════════════════════════════════════════════════════════════════════════
# make-android-boot.ps1 — Build TrustOS Android boot.img
# ═══════════════════════════════════════════════════════════════════════════
#
# Builds TrustOS for aarch64 and packages it into an Android boot.img
# that can be flashed via: fastboot flash boot trustos-boot.img
#
# Prerequisites:
#   - Rust nightly with aarch64-unknown-none target
#   - rust-lld (comes with Rust)
#   - Python 3 (for boot.img packaging)
#
# Usage:
#   .\make-android-boot.ps1                    # Build for QEMU virt
#   .\make-android-boot.ps1 -SoC pixel         # Build for Google Pixel
#   .\make-android-boot.ps1 -SoC qemu -Test    # Build + test in QEMU
#
# Boot.img layout (Android Boot Image v2):
#   Page 0:     Boot header (4096 bytes)
#   Page 1+:    Kernel binary (TrustOS flat binary)
#   (optional): Ramdisk, DTB
# ═══════════════════════════════════════════════════════════════════════════

param(
    [ValidateSet('qemu', 'pixel', 'oneplus', 'rpi', 'generic')]
    [string]$SoC = 'qemu',
    
    [switch]$Test,      # Launch in QEMU after build
    [switch]$Release,   # Build with optimizations
    [switch]$Verbose     # Show detailed build output
)

$ErrorActionPreference = "Stop"
$ROOT = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host "═══════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  TrustOS Android Boot Image Builder" -ForegroundColor Cyan
Write-Host "  Target SoC: $SoC" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════" -ForegroundColor Cyan

# ── SoC-specific configuration ──
$SoCConfig = @{
    'qemu' = @{
        KernelBase = 0x40080000
        UartBase   = 0x09000000
        Machine    = 'virt'
        DTB        = $null  # QEMU generates its own
    }
    'pixel' = @{
        KernelBase = 0x80080000
        UartBase   = 0x10A00000  # Tensor USI UART
        Machine    = $null
        DTB        = $null
    }
    'oneplus' = @{
        KernelBase = 0x80080000
        UartBase   = 0x0078AF00  # Qualcomm GENI UART
        Machine    = $null
        DTB        = $null
    }
    'rpi' = @{
        KernelBase = 0x80000
        UartBase   = 0xFE201000  # BCM2711 PL011
        Machine    = $null
        DTB        = $null
    }
    'generic' = @{
        KernelBase = 0x80080000
        UartBase   = 0x09000000
        Machine    = $null
        DTB        = $null
    }
}

$cfg = $SoCConfig[$SoC]
$KERNEL_BASE = $cfg.KernelBase
$KERNEL_BASE_HEX = '0x{0:X}' -f $KERNEL_BASE

Write-Host "`n[1/4] Building TrustOS kernel (aarch64)..." -ForegroundColor Yellow

$buildProfile = if ($Release) { "--release" } else { "" }
$targetDir = if ($Release) { "release" } else { "debug" }

# Build the kernel for aarch64
$buildArgs = @(
    "build"
    "--target", "aarch64-unknown-none"
    "-p", "trustos_kernel"
)
if ($Release) { $buildArgs += "--release" }

$env:RUSTFLAGS = "-C link-arg=-T$ROOT\kernel\linker-aarch64.ld"

Push-Location $ROOT
try {
    if ($Verbose) {
        & cargo @buildArgs 2>&1 | ForEach-Object { Write-Host $_ }
    } else {
        $output = & cargo @buildArgs 2>&1
        $errors = $output | Where-Object { $_ -match '^error' }
        if ($errors) {
            Write-Host "BUILD FAILED:" -ForegroundColor Red
            $errors | ForEach-Object { Write-Host $_ -ForegroundColor Red }
            exit 1
        }
    }
} finally {
    Pop-Location
    $env:RUSTFLAGS = $null
}

$ELF = "$ROOT\target\aarch64-unknown-none\$targetDir\trustos_kernel"
if (-not (Test-Path $ELF)) {
    Write-Host "ERROR: Kernel ELF not found at $ELF" -ForegroundColor Red
    exit 1
}

$elfSize = (Get-Item $ELF).Length
Write-Host "  Kernel ELF: $([math]::Round($elfSize / 1MB, 1)) MB" -ForegroundColor Green

# ── Step 2: Extract flat binary from ELF ──
Write-Host "`n[2/4] Extracting flat binary..." -ForegroundColor Yellow

$BIN = "$ROOT\target\aarch64-unknown-none\$targetDir\trustos_kernel.bin"

# Use rust-objcopy (from cargo-binutils) or llvm-objcopy
$objcopy = $null
$rustObjcopy = Get-Command "rust-objcopy" -ErrorAction SilentlyContinue
$llvmObjcopy = Get-Command "llvm-objcopy" -ErrorAction SilentlyContinue

if ($rustObjcopy) {
    $objcopy = "rust-objcopy"
} elseif ($llvmObjcopy) {
    $objcopy = "llvm-objcopy"
} else {
    Write-Host "  rust-objcopy not found, installing cargo-binutils..." -ForegroundColor DarkYellow
    & cargo install cargo-binutils 2>&1 | Out-Null
    & rustup component add llvm-tools-preview 2>&1 | Out-Null
    $objcopy = "rust-objcopy"
}

& $objcopy -O binary $ELF $BIN 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: objcopy failed" -ForegroundColor Red
    exit 1
}

$binSize = (Get-Item $BIN).Length
Write-Host "  Flat binary: $([math]::Round($binSize / 1KB, 0)) KB ($binSize bytes)" -ForegroundColor Green

# ── Step 3: Create boot.img (Android Boot Image v2) ──
Write-Host "`n[3/4] Packaging boot.img (Android Boot Image v2)..." -ForegroundColor Yellow

$BOOTIMG = "$ROOT\target\trustos-boot.img"
$PAGE_SIZE = 4096

# Python script to generate boot.img header
$pyScript = @"
import struct, sys, hashlib

kernel_path = sys.argv[1]
output_path = sys.argv[2]
kernel_addr = int(sys.argv[3], 0)
page_size = int(sys.argv[4])

with open(kernel_path, 'rb') as f:
    kernel = f.read()

kernel_size = len(kernel)

# Android Boot Image Header v2
magic = b'ANDROID!'
ramdisk_size = 0
ramdisk_addr = 0x01000000
second_size = 0
second_addr = 0
tags_addr = 0x00000100
header_version = 2
os_version = (1 << 25) | (0 << 18) | (0 << 11) | (26 << 4) | 2  # v1.0.0, 2026-02

name = b'TrustOS\x00' + b'\x00' * 8  # 16 bytes
cmdline = b'trustos.mode=desktop trustos.serial=ttyS0'
cmdline += b'\x00' * (512 - len(cmdline))
extra_cmdline = b'\x00' * 1024

# SHA-1 id
h = hashlib.sha1()
h.update(kernel)
h.update(struct.pack('<I', kernel_size))
sha_id = h.digest() + b'\x00' * 12  # 32 bytes

# Header v1 extensions
recovery_dtbo_size = 0
recovery_dtbo_offset = 0
header_size = 1660  # sizeof(boot_img_hdr_v2)

# Header v2 extensions
dtb_size = 0
dtb_addr = 0

# Pack header
header = struct.pack('<8s'     # magic
                     'I'       # kernel_size
                     'I'       # kernel_addr
                     'I'       # ramdisk_size
                     'I'       # ramdisk_addr
                     'I'       # second_size
                     'I'       # second_addr
                     'I'       # tags_addr
                     'I'       # page_size
                     'I'       # header_version
                     'I',      # os_version
                     magic,
                     kernel_size,
                     kernel_addr,
                     ramdisk_size,
                     ramdisk_addr,
                     second_size,
                     second_addr,
                     tags_addr,
                     page_size,
                     header_version,
                     os_version)

header += name          # 16 bytes
header += cmdline       # 512 bytes
header += sha_id        # 32 bytes
header += extra_cmdline # 1024 bytes

# v1 extensions
header += struct.pack('<I Q I',
                      recovery_dtbo_size,
                      recovery_dtbo_offset,
                      header_size)

# v2 extensions
header += struct.pack('<I Q',
                      dtb_size,
                      dtb_addr)

# Pad header to page size
header += b'\x00' * (page_size - len(header))

# Pad kernel to page boundary
kernel_padded = kernel + b'\x00' * (page_size - (len(kernel) % page_size)) if len(kernel) % page_size else kernel

# Write boot.img
with open(output_path, 'wb') as f:
    f.write(header)
    f.write(kernel_padded)

total = len(header) + len(kernel_padded)
print(f'boot.img: {total} bytes ({total // 1024} KB)')
print(f'  Header:  {page_size} bytes')
print(f'  Kernel:  {len(kernel_padded)} bytes ({kernel_size} raw)')
print(f'  Load at: 0x{kernel_addr:08X}')
"@

$pyScriptPath = "$ROOT\target\_make_bootimg.py"
$pyScript | Out-File -FilePath $pyScriptPath -Encoding utf8 -Force

& python $pyScriptPath $BIN $BOOTIMG $KERNEL_BASE_HEX $PAGE_SIZE
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: boot.img generation failed" -ForegroundColor Red
    exit 1
}

$bootimgSize = (Get-Item $BOOTIMG).Length
Write-Host "  boot.img: $([math]::Round($bootimgSize / 1KB, 0)) KB" -ForegroundColor Green
Write-Host "  Output: $BOOTIMG" -ForegroundColor Green

# ── Step 4: Test in QEMU (optional) ──
if ($Test -and $SoC -eq 'qemu') {
    Write-Host "`n[4/4] Launching QEMU aarch64..." -ForegroundColor Yellow
    Write-Host "  Machine: virt, CPU: cortex-a57, RAM: 512MB" -ForegroundColor DarkGray
    Write-Host "  Press Ctrl+A then X to exit QEMU" -ForegroundColor DarkGray
    Write-Host ""

    & qemu-system-aarch64 `
        -machine virt,secure=on `
        -cpu cortex-a57 `
        -m 512M `
        -nographic `
        -kernel $BIN `
        -append "trustos.mode=desktop" `
        -serial mon:stdio
} else {
    Write-Host "`n[4/4] Done!" -ForegroundColor Yellow
}

# ── Summary ──
Write-Host ""
Write-Host "═══════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  TrustOS Android Boot Image Ready" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Flash to Android device:" -ForegroundColor White
Write-Host "    fastboot flash boot $BOOTIMG" -ForegroundColor Yellow
Write-Host ""
Write-Host "  Test in QEMU:" -ForegroundColor White
Write-Host "    .\make-android-boot.ps1 -SoC qemu -Test" -ForegroundColor Yellow
Write-Host ""
Write-Host "  Target SoCs:" -ForegroundColor White
Write-Host "    qemu     — QEMU virt (testing)" -ForegroundColor DarkGray
Write-Host "    pixel    — Google Pixel (Tensor)" -ForegroundColor DarkGray
Write-Host "    oneplus  — OnePlus (Snapdragon)" -ForegroundColor DarkGray
Write-Host "    rpi      — Raspberry Pi 4/5" -ForegroundColor DarkGray
Write-Host "    generic  — Generic ARM64" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  Prerequisites for real device:" -ForegroundColor White
Write-Host "    1. Unlock bootloader: fastboot flashing unlock" -ForegroundColor DarkGray
Write-Host "    2. Flash: fastboot flash boot trustos-boot.img" -ForegroundColor DarkGray
Write-Host "    3. Reboot: fastboot reboot" -ForegroundColor DarkGray
Write-Host ""
