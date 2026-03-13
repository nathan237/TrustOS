# ============================================================================
# TrustOS -- Official Build Script (Base Edition)
# ============================================================================
# Builds the base TrustOS ISO without pretrained JARVIS weights.
# JARVIS is still included in the kernel but starts untrained.
#
# Usage:
#   .\build-trustos.ps1              # Build + launch VirtualBox
#   .\build-trustos.ps1 -NoRun       # Build only
#   .\build-trustos.ps1 -Clean       # Clean build from scratch
# ============================================================================
param(
    [switch]$NoRun,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"
$Edition = "TrustOS"
$OutputDir = "builds\trustos"
$IsoName = "trustos.iso"

New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

$cmakeBin = "C:\Program Files\CMake\bin"
$llvmBin = "C:\Program Files\LLVM\bin"
if (Test-Path $cmakeBin) { $env:Path = "$cmakeBin;" + $env:Path }
if (Test-Path $llvmBin) { $env:Path = "$llvmBin;" + $env:Path }
$env:CC = "clang"; $env:CXX = "clang++"; $env:AR = "llvm-ar"

$root = $PSScriptRoot
$mbedtlsInclude = Join-Path $root "kernel\mbedtls-include"
if (Test-Path $mbedtlsInclude) {
    $env:CFLAGS = "-I""$mbedtlsInclude"" -mcmodel=kernel -mno-red-zone -ffreestanding"
    $env:BINDGEN_EXTRA_CLANG_ARGS = "-I""$mbedtlsInclude"""
    $env:C_INCLUDE_PATH = $mbedtlsInclude
    $env:CPLUS_INCLUDE_PATH = $mbedtlsInclude
}

Write-Host ""
Write-Host "=== $Edition -- Build Pipeline ===" -ForegroundColor Cyan

if ($Clean) {
    Write-Host "[0/4] Cleaning previous build..." -ForegroundColor Yellow
    cargo clean 2>$null
    if (Test-Path "iso_root") { Remove-Item -Recurse -Force "iso_root" }
    $target = Join-Path $OutputDir $IsoName
    if (Test-Path $target) { Remove-Item -Force $target }
}

Write-Host "[1/4] Building kernel..." -ForegroundColor Yellow
$ErrorActionPreference = "Continue"
cargo build --release -p trustos_kernel 2>&1 | ForEach-Object { Write-Host $_ }
$ErrorActionPreference = "Stop"
if ($LASTEXITCODE -ne 0) { Write-Host "Build failed!" -ForegroundColor Red; exit 1 }

$kernelPath = "target\x86_64-unknown-none\release\trustos_kernel"
if (-not (Test-Path $kernelPath)) { Write-Host "Kernel not found!" -ForegroundColor Red; exit 1 }
$kernelSize = [math]::Round((Get-Item $kernelPath).Length / 1MB, 2)
Write-Host "Kernel built: $kernelSize MB" -ForegroundColor Green

Write-Host "[2/4] Creating ISO structure..." -ForegroundColor Yellow
$isoDir = "iso_root"
if (Test-Path $isoDir) { Remove-Item -Recurse -Force $isoDir }
New-Item -ItemType Directory -Path (Join-Path $isoDir "boot\limine") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $isoDir "EFI\BOOT") -Force | Out-Null

Copy-Item $kernelPath (Join-Path $isoDir "boot\trustos_kernel")

# Generate limine config WITHOUT jarvis module for base edition
$limineBase = (Get-Content "limine.conf") | Where-Object { $_ -notmatch "module_path|module_cmdline" }
$limineBase | Set-Content (Join-Path $isoDir "boot\limine\limine.conf")
$limineBase | Set-Content (Join-Path $isoDir "limine.conf")
# Also place as limine.cfg for older Limine compat
$limineBase | Set-Content (Join-Path $isoDir "boot\limine\limine.cfg")
Copy-Item "limine\limine-bios.sys" (Join-Path $isoDir "boot\limine")
Copy-Item "limine\limine-bios-cd.bin" (Join-Path $isoDir "boot\limine")
Copy-Item "limine\limine-uefi-cd.bin" (Join-Path $isoDir "boot\limine")
Copy-Item "limine\BOOTX64.EFI" (Join-Path $isoDir "EFI\BOOT")
Copy-Item "limine\BOOTIA32.EFI" (Join-Path $isoDir "EFI\BOOT")

Write-Host "  Base edition -- no pretrained JARVIS weights" -ForegroundColor DarkGray

Write-Host "[3/4] Creating bootable ISO..." -ForegroundColor Yellow
$isoPath = Join-Path $OutputDir $IsoName

$xorriso = Get-Command xorriso -ErrorAction SilentlyContinue
$oldErr2 = $ErrorActionPreference
$ErrorActionPreference = "Continue"
if (-not $xorriso) {
    $full = [System.IO.Path]::GetFullPath($isoDir)
    $drive = $full.Substring(0, 1).ToLower()
    $rest = $full.Substring(2) -replace "\\", "/"
    $wslIsoDir = "/mnt/$drive$rest"

    $full2 = [System.IO.Path]::GetFullPath($isoPath)
    $drive2 = $full2.Substring(0, 1).ToLower()
    $rest2 = $full2.Substring(2) -replace "\\", "/"
    $wslIsoPath = "/mnt/$drive2$rest2"

    wsl -e xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label --mbr-force-bootable $wslIsoDir -o $wslIsoPath 2>&1 | Out-Null
} else {
    xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label --mbr-force-bootable $isoDir -o $isoPath 2>&1 | Out-Null
}
$ErrorActionPreference = $oldErr2
if ($LASTEXITCODE -ne 0) { Write-Host "ISO creation failed!" -ForegroundColor Red; exit 1 }

Write-Host "  Installing BIOS bootloader (Legacy BIOS support)..." -ForegroundColor DarkGray
$oldErr = $ErrorActionPreference
$ErrorActionPreference = "Continue"
$biosResult = & "limine\limine.exe" bios-install $isoPath 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "  WARNING: limine bios-install failed: $biosResult" -ForegroundColor Yellow
    Write-Host "  Legacy BIOS boot may not work" -ForegroundColor Yellow
} else {
    Write-Host "  BIOS boot sectors installed OK" -ForegroundColor DarkGreen
}
$ErrorActionPreference = $oldErr

# Patch MBR: set partition 1 bootable flag (0x80) for Legacy BIOS compatibility
$isoBytes = [System.IO.File]::ReadAllBytes($isoPath)
if ($isoBytes[446] -ne 0x80) {
    $isoBytes[446] = 0x80
    [System.IO.File]::WriteAllBytes($isoPath, $isoBytes)
    Write-Host "  MBR bootable flag patched for Legacy BIOS" -ForegroundColor DarkGreen
}

Copy-Item $isoPath "trustos.iso" -Force

$isoSize = [math]::Round((Get-Item $isoPath).Length / 1MB, 2)

Write-Host "[4/4] Build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "  Edition:  $Edition (Base)" -ForegroundColor Cyan
Write-Host "  Kernel:   $kernelSize MB" -ForegroundColor Cyan
Write-Host "  ISO:      $isoSize MB" -ForegroundColor Cyan
Write-Host "  Output:   $OutputDir" -ForegroundColor Cyan
Write-Host "  Boot:     UEFI + Legacy BIOS (hybrid)" -ForegroundColor Cyan
Write-Host "  JARVIS:   Code included, no pretrained weights" -ForegroundColor Cyan
Write-Host ""

if (-not $NoRun) {
    Write-Host "Launching VirtualBox..." -ForegroundColor Green
    $launch = Join-Path $root "scripts\launch\launch-vbox-clean.ps1"
    & $launch
}
