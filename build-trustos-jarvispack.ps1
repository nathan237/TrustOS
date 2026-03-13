# ============================================================================
# TrustOS JarvisPack -- Official Build Script (AI Edition)
# ============================================================================
# Builds the full TrustOS ISO with pretrained JARVIS brain weights.
# Includes the 4.4M-parameter transformer ready for on-device inference.
#
# Usage:
#   .\build-trustos-jarvispack.ps1              # Build + launch VirtualBox
#   .\build-trustos-jarvispack.ps1 -NoRun       # Build only
#   .\build-trustos-jarvispack.ps1 -Clean       # Clean build from scratch
# ============================================================================
param(
    [switch]$NoRun,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"
$Edition = "TrustOS-JarvisPack"
$OutputDir = "builds\trustos-jarvispack"
$IsoName = "trustos-jarvispack.iso"

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
Write-Host "=== $Edition -- Build Pipeline ===" -ForegroundColor Magenta

if ($Clean) {
    Write-Host "[0/5] Cleaning previous build..." -ForegroundColor Yellow
    cargo clean 2>$null
    if (Test-Path "iso_root") { Remove-Item -Recurse -Force "iso_root" }
    $target = Join-Path $OutputDir $IsoName
    if (Test-Path $target) { Remove-Item -Force $target }
}

# Check for JARVIS brain weights
Write-Host "[1/5] Checking JARVIS brain weights..." -ForegroundColor Yellow
$brainLocations = @(
    (Join-Path $OutputDir "jarvis_pretrained.bin"),
    "jarvis-bench\jarvis_pretrained.bin"
)
$brainFile = $null
$brainSize = 0
foreach ($bp in $brainLocations) {
    if (Test-Path $bp) { $brainFile = $bp; $brainSize = [math]::Round((Get-Item $bp).Length / 1MB, 2); break }
}

if ($brainFile) {
    Write-Host "  JARVIS brain found: $brainFile ($brainSize MB)" -ForegroundColor Green
} else {
    Write-Host "  WARNING: No pretrained JARVIS brain found!" -ForegroundColor Red
    Write-Host "  Expected:" -ForegroundColor Yellow
    foreach ($bp in $brainLocations) { Write-Host "    - $bp" -ForegroundColor DarkGray }
    Write-Host "  ISO will be built without pretrained weights." -ForegroundColor Yellow
}

Write-Host "[2/5] Building kernel..." -ForegroundColor Yellow
$ErrorActionPreference = "Continue"
cargo build --release -p trustos_kernel 2>&1 | ForEach-Object { Write-Host $_ }
$ErrorActionPreference = "Stop"
if ($LASTEXITCODE -ne 0) { Write-Host "Build failed!" -ForegroundColor Red; exit 1 }

$kernelPath = "target\x86_64-unknown-none\release\trustos_kernel"
if (-not (Test-Path $kernelPath)) { Write-Host "Kernel not found!" -ForegroundColor Red; exit 1 }
$kernelSize = [math]::Round((Get-Item $kernelPath).Length / 1MB, 2)
Write-Host "Kernel built: $kernelSize MB" -ForegroundColor Green

Write-Host "[3/5] Creating ISO structure..." -ForegroundColor Yellow
$isoDir = "iso_root"
if (Test-Path $isoDir) { Remove-Item -Recurse -Force $isoDir }
New-Item -ItemType Directory -Path (Join-Path $isoDir "boot\limine") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $isoDir "EFI\BOOT") -Force | Out-Null

Copy-Item $kernelPath (Join-Path $isoDir "boot\trustos_kernel")
Copy-Item "limine.conf" (Join-Path $isoDir "boot\limine\limine.conf")
Copy-Item "limine.conf" (Join-Path $isoDir "limine.conf")
# Also place as limine.cfg for older Limine compat
Copy-Item "limine.conf" (Join-Path $isoDir "boot\limine\limine.cfg")
Copy-Item "limine\limine-bios.sys" (Join-Path $isoDir "boot\limine")
Copy-Item "limine\limine-bios-cd.bin" (Join-Path $isoDir "boot\limine")
Copy-Item "limine\limine-uefi-cd.bin" (Join-Path $isoDir "boot\limine")
Copy-Item "limine\BOOTX64.EFI" (Join-Path $isoDir "EFI\BOOT")
Copy-Item "limine\BOOTIA32.EFI" (Join-Path $isoDir "EFI\BOOT")

# JarvisPack: include pretrained brain
if ($brainFile) {
    Copy-Item $brainFile (Join-Path $isoDir "jarvis_pretrained.bin")
    Write-Host "  JARVIS brain weights included ($brainSize MB)" -ForegroundColor Cyan
}

Write-Host "[4/5] Creating bootable ISO..." -ForegroundColor Yellow
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

    wsl -e xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label $wslIsoDir -o $wslIsoPath 2>&1 | Out-Null
} else {
    xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label $isoDir -o $isoPath 2>&1 | Out-Null
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

$isoSize = [math]::Round((Get-Item $isoPath).Length / 1MB, 2)

Write-Host "[5/5] Build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "  Edition:  $Edition (AI)" -ForegroundColor Magenta
Write-Host "  Kernel:   $kernelSize MB" -ForegroundColor Magenta
Write-Host "  ISO:      $isoSize MB" -ForegroundColor Magenta
Write-Host "  Output:   $OutputDir" -ForegroundColor Magenta
Write-Host "  Boot:     UEFI + Legacy BIOS (hybrid)" -ForegroundColor Magenta
if ($brainFile) {
    Write-Host "  JARVIS:   Pretrained brain ($brainSize MB)" -ForegroundColor Magenta
} else {
    Write-Host "  JARVIS:   Code only (no weights)" -ForegroundColor Magenta
}
Write-Host ""

if (-not $NoRun) {
    Write-Host "Launching VirtualBox..." -ForegroundColor Cyan
    $launch = Join-Path $root "scripts\launch\launch-vbox-clean.ps1"
    & $launch
}
