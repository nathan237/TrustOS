# TRustOs - Build & Run with Limine
param(
    [switch]$NoRun
)

$ErrorActionPreference = "Stop"

# Ensure CMake and LLVM are available in PATH for mbedtls-sys
$cmakeBin = "C:\Program Files\CMake\bin"
$llvmBin = "C:\Program Files\LLVM\bin"
if (Test-Path $cmakeBin) { $env:Path = "$cmakeBin;" + $env:Path }
if (Test-Path $llvmBin) { $env:Path = "$llvmBin;" + $env:Path }

# Prefer clang toolchain for C builds
$env:CC = "clang"
$env:CXX = "clang++"
$env:AR = "llvm-ar"

# Provide freestanding headers for mbedtls (assert.h, stddef.h, etc.)
$mbedtlsInclude = "$PSScriptRoot\kernel\mbedtls-include"
if (Test-Path $mbedtlsInclude) {
    $env:CFLAGS = "-I`"$mbedtlsInclude`" -mcmodel=kernel -mno-red-zone -ffreestanding"
    $env:BINDGEN_EXTRA_CLANG_ARGS = "-I`"$mbedtlsInclude`""
    $env:C_INCLUDE_PATH = $mbedtlsInclude
    $env:CPLUS_INCLUDE_PATH = $mbedtlsInclude
}

Write-Host "`n=== TRUSTOS BUILD (Limine) ===" -ForegroundColor Cyan

# Build kernel
Write-Host "`n[1/3] Building kernel..." -ForegroundColor Yellow
$ErrorActionPreference = "Continue"
cargo build --release -p trustos_kernel 2>&1 | ForEach-Object { Write-Host $_ }
$ErrorActionPreference = "Stop"
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

$kernelPath = "target\x86_64-unknown-none\release\trustos_kernel"
if (-not (Test-Path $kernelPath)) {
    Write-Host "Kernel not found at $kernelPath" -ForegroundColor Red
    exit 1
}
Write-Host "Kernel built: $kernelPath" -ForegroundColor Green

# Create ISO directory structure
Write-Host "`n[2/3] Creating ISO structure..." -ForegroundColor Yellow
$isoDir = "iso_root"
if (Test-Path $isoDir) { Remove-Item -Recurse -Force $isoDir }
New-Item -ItemType Directory -Path $isoDir | Out-Null
New-Item -ItemType Directory -Path "$isoDir\boot" | Out-Null
New-Item -ItemType Directory -Path "$isoDir\boot\limine" | Out-Null
New-Item -ItemType Directory -Path "$isoDir\EFI\BOOT" -Force | Out-Null

# Copy files
Copy-Item $kernelPath "$isoDir\boot\trustos_kernel"
Copy-Item "limine.cfg" "$isoDir\boot\limine\limine.cfg"
Copy-Item "limine.cfg" "$isoDir\boot\limine\limine.conf"
Copy-Item "limine.cfg" "$isoDir\limine.conf"
Copy-Item "limine\limine-bios.sys" "$isoDir\boot\limine\"
Copy-Item "limine\limine-bios-cd.bin" "$isoDir\boot\limine\"
Copy-Item "limine\limine-uefi-cd.bin" "$isoDir\boot\limine\"
Copy-Item "limine\BOOTX64.EFI" "$isoDir\EFI\BOOT\"
Copy-Item "limine\BOOTIA32.EFI" "$isoDir\EFI\BOOT\"

Write-Host "ISO structure created" -ForegroundColor Green

# Create ISO using xorriso
Write-Host "`n[3/3] Creating bootable ISO..." -ForegroundColor Yellow
$isoPath = "trustos.iso"

# Check for xorriso
$xorriso = Get-Command xorriso -ErrorAction SilentlyContinue
if (-not $xorriso) {
    Write-Host "xorriso not found. Using WSL xorriso..." -ForegroundColor Yellow

    function Convert-ToWslPath([string]$winPath) {
        $full = [System.IO.Path]::GetFullPath($winPath)
        $drive = $full.Substring(0, 1).ToLower()
        $rest = $full.Substring(2) -replace '\\', '/'
        return "/mnt/$drive$rest"
    }

    $wslIsoDir = Convert-ToWslPath $isoDir
    $wslIsoPath = Convert-ToWslPath $isoPath

    wsl -e xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin `
        -no-emul-boot -boot-load-size 4 -boot-info-table `
        --efi-boot boot/limine/limine-uefi-cd.bin `
        -efi-boot-part --efi-boot-image --protective-msdos-label `
        $wslIsoDir -o $wslIsoPath
} else {
    xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin `
        -no-emul-boot -boot-load-size 4 -boot-info-table `
        --efi-boot boot/limine/limine-uefi-cd.bin `
        -efi-boot-part --efi-boot-image --protective-msdos-label `
        $isoDir -o $isoPath
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "ISO creation failed!" -ForegroundColor Red
    exit 1
}

# Install Limine to ISO
$oldErrPref = $ErrorActionPreference
$ErrorActionPreference = "Continue"
& "limine\limine.exe" bios-install $isoPath 2>$null | Out-Null
$ErrorActionPreference = $oldErrPref
if ($LASTEXITCODE -ne 0) {
    Write-Host "Limine BIOS install returned a non-zero exit code. Continuing..." -ForegroundColor Yellow
}

Write-Host "`nISO created: $isoPath" -ForegroundColor Green

# Launch VirtualBox unless suppressed
if (-not $NoRun) {
    Write-Host "`n=== LAUNCHING VIRTUALBOX ===" -ForegroundColor Green
    & "$PSScriptRoot\run-vbox.ps1"
} else {
    Write-Host "`n[SKIP] VirtualBox launch suppressed (-NoRun)" -ForegroundColor Yellow
}
