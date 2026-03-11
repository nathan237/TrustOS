#!/usr/bin/env pwsh
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
# TrustOS Universal Installer
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
#
# Boot TrustOS on ANYTHING.
#
# Supported targets:
#   PC (x86_64)     â€” UEFI + Legacy BIOS via Limine (ISO/USB)
#   Android phone   â€” fastboot flash boot (Pixel, OnePlus, Xiaomi, etc.)
#   Raspberry Pi    â€” SD card image (RPi 4/5, kernel8.img)
#   ARM board       â€” U-Boot / flat binary (Pine64, etc.)
#   RISC-V board    â€” OpenSBI + U-Boot (VisionFive, etc.)
#   VM              â€” QEMU auto-detect (all architectures)
#
# Usage:
#   .\trustos-install.ps1                    # Interactive mode
#   .\trustos-install.ps1 -Target pc-usb     # Flash to USB drive
#   .\trustos-install.ps1 -Target android    # Flash via fastboot
#   .\trustos-install.ps1 -Target rpi-sd     # Write RPi SD card
#   .\trustos-install.ps1 -Target qemu       # Launch in QEMU
#   .\trustos-install.ps1 -List              # Show all targets
#
# No exploitation. No jailbreak. Just freedom.
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

param(
    [ValidateSet('', 'pc-usb', 'pc-iso', 'android', 'rpi-sd', 'arm-generic', 'riscv', 'qemu-x86', 'qemu-arm', 'qemu-riscv')]
    [string]$Target = '',
    [switch]$List,
    [switch]$Release,
    [switch]$Verbose,
    [switch]$NoConfirm
)

$ErrorActionPreference = "Continue"
$ROOT = Split-Path -Parent $MyInvocation.MyCommand.Path
$BUILD_DIR = "$ROOT\target"
$PROFILE = if ($Release) { "release" } else { "debug" }

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
# Banner
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

function Show-Banner {
    Write-Host ""
    Write-Host "  â–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ•-â–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ•- â–ˆâ–ˆâ•-   â–ˆâ–ˆâ•-â–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ•-â–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ•- â–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ•- â–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ•-" -ForegroundColor Cyan
    Write-Host "  â•šâ•â•â–ˆâ–ˆâ•”â•â•â•â–ˆâ–ˆâ•”â•â•â–ˆâ–ˆâ•-â–ˆâ–ˆâ•‘   â–ˆâ–ˆâ•‘â–ˆâ–ˆâ•”â•â•â•â•â•â•šâ•â•â–ˆâ–ˆâ•”â•â•â•â–ˆâ–ˆâ•”â•â•â•â–ˆâ–ˆâ•-â–ˆâ–ˆâ•”â•â•â•â•â•" -ForegroundColor Cyan
    Write-Host "     â–ˆâ–ˆâ•‘   â–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ•”â•â–ˆâ–ˆâ•‘   â–ˆâ–ˆâ•‘â–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ•-   â–ˆâ–ˆâ•‘   â–ˆâ–ˆâ•‘   â–ˆâ–ˆâ•‘â–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ•-" -ForegroundColor Cyan
    Write-Host "     â–ˆâ–ˆâ•‘   â–ˆâ–ˆâ•”â•â•â–ˆâ–ˆâ•-â–ˆâ–ˆâ•‘   â–ˆâ–ˆâ•‘â•šâ•â•â•â•â–ˆâ–ˆâ•‘   â–ˆâ–ˆâ•‘   â–ˆâ–ˆâ•‘   â–ˆâ–ˆâ•‘â•šâ•â•â•â•â–ˆâ–ˆâ•‘" -ForegroundColor Cyan
    Write-Host "     â–ˆâ–ˆâ•‘   â–ˆâ–ˆâ•‘  â–ˆâ–ˆâ•‘â•šâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ•”â•â–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ•‘   â–ˆâ–ˆâ•‘   â•šâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ•”â•â–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ•‘" -ForegroundColor Cyan
    Write-Host "     â•šâ•â•   â•šâ•â•  â•šâ•â• â•šâ•â•â•â•â•â• â•šâ•â•â•â•â•â•â•   â•šâ•â•    â•šâ•â•â•â•â•â• â•šâ•â•â•â•â•â•â•" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "     Universal Installer - Run TrustOS on anything" -ForegroundColor White
    Write-Host "     No exploitation. No jailbreak. Just freedom." -ForegroundColor DarkGray
    Write-Host ""
}

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
# Target definitions
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

$TARGETS = [ordered]@{
    'pc-usb' = @{
        Name        = 'PC - USB Drive (UEFI + BIOS)'
        Arch        = "x86_64"
        RustTarget  = "x86_64-unknown-none"
        Description = "Write bootable TrustOS to a USB drive. Works on any PC (UEFI or Legacy BIOS)."
        Method      = "limine-usb"
        Icon        = "[PC]"
    }
    'pc-iso' = @{
        Name        = 'PC - ISO Image'
        Arch        = "x86_64"
        RustTarget  = "x86_64-unknown-none"
        Description = "Create a bootable ISO for burning to CD/DVD or mounting in a VM."
        Method      = "limine-iso"
        Icon        = "[ISO]"
    }
    'android' = @{
        Name        = "Android Phone (fastboot)"
        Arch        = "aarch64"
        RustTarget  = "aarch64-unknown-none"
        Description = "Flash TrustOS to any Android phone with unlocked bootloader via fastboot."
        Method      = "android-fastboot"
        Icon        = "[PHONE]"
    }
    'rpi-sd' = @{
        Name        = 'Raspberry Pi 4/5 - SD Card'
        Arch        = "aarch64"
        RustTarget  = "aarch64-unknown-none"
        Description = "Write TrustOS to SD card for Raspberry Pi 4 or 5. Direct bare-metal boot."
        Method      = "rpi-sdcard"
        Icon        = "[RPI]"
    }
    'arm-generic' = @{
        Name        = "Generic ARM64 Board"
        Arch        = "aarch64"
        RustTarget  = "aarch64-unknown-none"
        Description = "Flat binary for ARM64 boards (Pine64, Rock64, etc.) via U-Boot or direct load."
        Method      = "arm-binary"
        Icon        = "[ARM]"
    }
    'riscv' = @{
        Name        = "RISC-V Board"
        Arch        = "riscv64"
        RustTarget  = "riscv64gc-unknown-none-elf"
        Description = "Binary for RISC-V boards (VisionFive, Milk-V, etc.) via OpenSBI + U-Boot."
        Method      = "riscv-binary"
        Icon        = "[RISCV]"
    }
    'qemu-x86' = @{
        Name        = 'QEMU - x86_64 (test)'
        Arch        = "x86_64"
        RustTarget  = "x86_64-unknown-none"
        Description = "Launch TrustOS in QEMU x86_64 with UEFI. For development and testing."
        Method      = "qemu-x86"
        Icon        = "[QEMU]"
    }
    'qemu-arm' = @{
        Name        = 'QEMU - ARM64 (test)'
        Arch        = "aarch64"
        RustTarget  = "aarch64-unknown-none"
        Description = "Launch TrustOS in QEMU ARM64 virt machine. For testing Android/RPi boot."
        Method      = "qemu-arm"
        Icon        = "[QEMU]"
    }
    'qemu-riscv' = @{
        Name        = 'QEMU - RISC-V (test)'
        Arch        = "riscv64"
        RustTarget  = "riscv64gc-unknown-none-elf"
        Description = "Launch TrustOS in QEMU RISC-V virt machine."
        Method      = "qemu-riscv"
        Icon        = "[QEMU]"
    }
}

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
# List targets
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

if ($List) {
    Show-Banner
    Write-Host "  Available targets:" -ForegroundColor Yellow
    Write-Host ""
    foreach ($key in $TARGETS.Keys) {
        $t = $TARGETS[$key]
        $pad = $key.PadRight(14)
        Write-Host "    $($t.Icon) " -ForegroundColor Cyan -NoNewline
        Write-Host "$pad" -ForegroundColor White -NoNewline
        Write-Host " $($t.Name)" -ForegroundColor Green
        Write-Host "                          $($t.Description)" -ForegroundColor DarkGray
    }
    Write-Host ""
    Write-Host '  Usage: .\trustos-install.ps1 -Target <name>' -ForegroundColor White
    Write-Host ""
    exit 0
}

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
# Interactive target selection
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

if (-not $Target) {
    Show-Banner
    Write-Host "  Where do you want to run TrustOS?" -ForegroundColor Yellow
    Write-Host ""
    
    $i = 1
    $keys = @($TARGETS.Keys)
    foreach ($key in $keys) {
        $t = $TARGETS[$key]
        $num = "$i".PadLeft(2)
        Write-Host ('    ' + $num + ') ' + $t.Icon + ' ' + $t.Name) -ForegroundColor White
        $i++
    }
    Write-Host ""
    $choice = Read-Host "  Select target (1-$($keys.Count))"
    $idx = [int]$choice - 1
    if ($idx -lt 0 -or $idx -ge $keys.Count) {
        Write-Host "  Invalid choice." -ForegroundColor Red
        exit 1
    }
    $Target = $keys[$idx]
}

$config = $TARGETS[$Target]
Show-Banner
Write-Host "  Target: $($config.Icon) $($config.Name)" -ForegroundColor Green
Write-Host "  Arch:   $($config.Arch)" -ForegroundColor DarkGray
Write-Host "  Method: $($config.Method)" -ForegroundColor DarkGray
Write-Host ""

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
# Build kernel
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

function Build-Kernel {
    param([string]$RustTarget, [string]$Arch)
    
    Write-Host "  [BUILD] Compiling TrustOS for $Arch..." -ForegroundColor Yellow
    
    $buildArgs = @("build", "--target", $RustTarget, "-p", "trustos_kernel")
    if ($Release) { $buildArgs += "--release" }
    
    Push-Location $ROOT
    try {
        $prevPref = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        $output = & cargo @buildArgs 2>&1
        $ErrorActionPreference = $prevPref
        $errors = $output | Where-Object { $_ -match '^error\[' -or $_ -match '^error:' }
        if ($errors) {
            Write-Host "  [FAIL] Build failed:" -ForegroundColor Red
            $errors | ForEach-Object { Write-Host "    $_" -ForegroundColor Red }
            exit 1
        }
        # Check that build actually produced output
        $finished = $output | Where-Object { $_ -match 'Finished|Compiling' }
        if (-not $finished -and $LASTEXITCODE -ne 0) {
            Write-Host "  [FAIL] Build failed (exit code $LASTEXITCODE)" -ForegroundColor Red
            exit 1
        }
    } finally {
        Pop-Location
    }
    
    $elf = "$BUILD_DIR\$RustTarget\$PROFILE\trustos_kernel"
    if (-not (Test-Path $elf)) {
        Write-Host "  [FAIL] Kernel ELF not found: $elf" -ForegroundColor Red
        exit 1
    }
    
    $size = [math]::Round((Get-Item $elf).Length / 1MB, 1)
    Write-Host "  [OK]    Kernel: $size MB ($Arch)" -ForegroundColor Green
    return $elf
}

function Get-ObjCopy {
    $cmd = Get-Command "rust-objcopy" -ErrorAction SilentlyContinue
    if ($cmd) { return "rust-objcopy" }
    $cmd = Get-Command "llvm-objcopy" -ErrorAction SilentlyContinue
    if ($cmd) { return "llvm-objcopy" }
    Write-Host "  Installing cargo-binutils..." -ForegroundColor DarkYellow
    & cargo install cargo-binutils 2>&1 | Out-Null
    & rustup component add llvm-tools-preview 2>&1 | Out-Null
    return "rust-objcopy"
}

function Extract-Binary {
    param([string]$ElfPath)
    $binPath = $ElfPath + ".bin"
    $objcopy = Get-ObjCopy
    & $objcopy -O binary $ElfPath $binPath 2>&1 | Out-Null
    if (-not (Test-Path $binPath)) {
        Write-Host "  [FAIL] Binary extraction failed" -ForegroundColor Red
        exit 1
    }
    $size = [math]::Round((Get-Item $binPath).Length / 1KB, 0)
    Write-Host "  [OK]    Binary: $size KB" -ForegroundColor Green
    return $binPath
}

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
# Method: Limine USB (x86_64)
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

function Install-LimineUSB {
    # Delegate to existing scripts
    Write-Host "  [ISO]   Building Limine ISO..." -ForegroundColor Yellow
    & "$ROOT\build-limine.ps1" -NoRun
    
    $iso = "$ROOT\trustos.iso"
    if (-not (Test-Path $iso)) {
        Write-Host "  [FAIL] ISO not created" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "  [USB]   Flashing to USB..." -ForegroundColor Yellow
    & "$ROOT\flash-usb.ps1"
}

function Install-LimineISO {
    Write-Host "  [ISO]   Building Limine ISO..." -ForegroundColor Yellow
    & "$ROOT\build-limine.ps1" -NoRun
    
    $iso = "$ROOT\trustos.iso"
    if (Test-Path $iso) {
        $size = [math]::Round((Get-Item $iso).Length / 1MB, 0)
        Write-Host ""
        Write-Host "  Done! ISO ready: trustos.iso ($size MB)" -ForegroundColor Green
        Write-Host "  Burn to disc or mount in VirtualBox/VMware." -ForegroundColor DarkGray
    }
}

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
# Method: Android fastboot
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

function Install-AndroidFastboot {
    # Check fastboot
    $fb = Get-Command "fastboot" -ErrorAction SilentlyContinue
    if (-not $fb) {
        Write-Host "  [WARN] fastboot not found in PATH" -ForegroundColor Yellow
        Write-Host "         Download: https://developer.android.com/tools/releases/platform-tools" -ForegroundColor DarkGray
    }
    
    # Build boot.img
    Write-Host "  [IMG]   Building Android boot.img..." -ForegroundColor Yellow
    & "$ROOT\make-android-boot.ps1" -SoC generic
    
    $bootimg = "$BUILD_DIR\trustos-boot.img"
    if (-not (Test-Path $bootimg)) {
        Write-Host "  [FAIL] boot.img not created" -ForegroundColor Red
        exit 1
    }
    
    # Check for connected device
    if ($fb) {
        Write-Host ""
        Write-Host "  [DEVICE] Checking for connected device..." -ForegroundColor Yellow
        $devices = & fastboot devices 2>&1
        if ($devices -match '\S+\s+fastboot') {
            $deviceId = ($devices -split '\s+')[0]
            Write-Host "  [OK]    Device found: $deviceId" -ForegroundColor Green
            
            if (-not $NoConfirm) {
                Write-Host ""
                Write-Host "  WARNING: This will replace the boot partition!" -ForegroundColor Red
                Write-Host "  Your Android OS will be replaced by TrustOS." -ForegroundColor Red
                Write-Host "  To restore: fastboot flash boot <original-boot.img>" -ForegroundColor DarkGray
                Write-Host ""
                $confirm = Read-Host "  Type YES to flash TrustOS"
                if ($confirm -ne "YES") {
                    Write-Host "  Aborted." -ForegroundColor Yellow
                    return
                }
            }
            
            Write-Host ""
            Write-Host "  [FLASH] fastboot flash boot trustos-boot.img" -ForegroundColor Cyan
            & fastboot flash boot $bootimg
            
            if ($LASTEXITCODE -eq 0) {
                Write-Host ""
                Write-Host "  Flashed! Rebooting..." -ForegroundColor Green
                & fastboot reboot
                Write-Host ""
                Write-Host "  TrustOS is now booting on your phone." -ForegroundColor Green
                Write-Host "  Connect serial (USB UART) to see output." -ForegroundColor DarkGray
            } else {
                Write-Host "  [FAIL] Flash failed. Check device connection." -ForegroundColor Red
            }
        } else {
            Write-Host "  [INFO] No device in fastboot mode." -ForegroundColor Yellow
            Write-Host ""
            Write-Host "  To flash manually:" -ForegroundColor White
            Write-Host "    1. Boot phone into fastboot: Power + Vol Down" -ForegroundColor DarkGray
            Write-Host "    2. Unlock bootloader (one-time):" -ForegroundColor DarkGray
            Write-Host "       fastboot flashing unlock" -ForegroundColor Cyan
            Write-Host "    3. Flash TrustOS:" -ForegroundColor DarkGray
            Write-Host "       fastboot flash boot $bootimg" -ForegroundColor Cyan
            Write-Host "    4. Reboot:" -ForegroundColor DarkGray
            Write-Host "       fastboot reboot" -ForegroundColor Cyan
        }
    } else {
        Write-Host ""
        Write-Host "  boot.img ready: $bootimg" -ForegroundColor Green
        Write-Host "  Flash manually: fastboot flash boot $bootimg" -ForegroundColor Cyan
    }
}

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
# Method: Raspberry Pi SD card
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

function Install-RpiSD {
    $elf = Build-Kernel -RustTarget $config.RustTarget -Arch $config.Arch
    $bin = Extract-Binary -ElfPath $elf
    
    # Create RPi boot structure
    $rpiDir = "$BUILD_DIR\rpi-boot"
    if (Test-Path $rpiDir) { Remove-Item $rpiDir -Recurse -Force }
    New-Item -Path $rpiDir -ItemType Directory -Force | Out-Null
    
    # kernel8.img â€” the RPi bootloader loads this for aarch64
    Copy-Item $bin "$rpiDir\kernel8.img" -Force
    
    # config.txt â€” RPi GPU firmware configuration
    @"
# TrustOS â€” Raspberry Pi Boot Configuration
# This file is read by the RPi GPU firmware (not TrustOS)

# Boot in 64-bit mode (aarch64)
arm_64bit=1

# Disable Bluetooth (frees up PL011 UART for serial console)
dtoverlay=disable-bt

# Use PL011 UART (not mini UART) â€” proper 115200 baud serial
enable_uart=1

# GPU memory split â€” minimum for GPU, maximum for TrustOS
gpu_mem=16

# Disable overscan (full screen framebuffer)
disable_overscan=1

# Framebuffer settings 
framebuffer_width=1920
framebuffer_height=1080
framebuffer_depth=32

# Force HDMI output (even without a connected monitor)
hdmi_force_hotplug=1
hdmi_group=2
hdmi_mode=82

# Kernel filename (default for aarch64 is kernel8.img)
kernel=kernel8.img

# Pass DTB to kernel â€” TrustOS reads hardware info from this
device_tree=bcm2711-rpi-4-b.dtb

# Disable kernel command line (TrustOS doesn't use Linux cmdline)
# cmdline=
"@ | Out-File "$rpiDir\config.txt" -Encoding ascii -Force
    
    # cmdline.txt (empty â€” TrustOS ignores it but RPi firmware expects it)
    "" | Out-File "$rpiDir\cmdline.txt" -Encoding ascii -Force
    
    Write-Host ""
    Write-Host "  [OK]    RPi boot files ready in: target\rpi-boot\" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Files created:" -ForegroundColor White
    Write-Host '    kernel8.img  - TrustOS kernel binary' -ForegroundColor DarkGray
    Write-Host '    config.txt   - RPi boot configuration' -ForegroundColor DarkGray
    Write-Host '    cmdline.txt  - (empty, required by firmware)' -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "  To boot on Raspberry Pi 4/5:" -ForegroundColor Yellow
    Write-Host "    1. Format SD card as FAT32" -ForegroundColor DarkGray
    Write-Host "    2. Copy RPi firmware files from:" -ForegroundColor DarkGray
    Write-Host "       https://github.com/raspberrypi/firmware/tree/master/boot" -ForegroundColor Cyan
    Write-Host "       (bootcode.bin, start4.elf, fixup4.dat, bcm2711-rpi-4-b.dtb)" -ForegroundColor DarkGray
    Write-Host "    3. Copy kernel8.img + config.txt to SD card root" -ForegroundColor DarkGray
    Write-Host "    4. Insert SD card, connect HDMI + USB serial, power on" -ForegroundColor DarkGray
    Write-Host ""
    
    # Offer to auto-copy to SD card if a removable drive is detected
    $removable = Get-Volume | Where-Object { $_.DriveType -eq 'Removable' -and $_.FileSystemLabel -ne '' }
    if ($removable) {
        Write-Host "  Detected removable drives:" -ForegroundColor Yellow
        foreach ($vol in $removable) {
            Write-Host "    $($vol.DriveLetter): $($vol.FileSystemLabel) ($([math]::Round($vol.Size/1GB, 1)) GB)" -ForegroundColor White
        }
        Write-Host ""
        $drive = Read-Host "  Enter drive letter to copy files (or ENTER to skip)"
        if ($drive -and $drive -match '^[A-Za-z]$') {
            $dest = "${drive}:\"
            if (Test-Path $dest) {
                Copy-Item "$rpiDir\kernel8.img" $dest -Force
                Copy-Item "$rpiDir\config.txt" $dest -Force
                Copy-Item "$rpiDir\cmdline.txt" $dest -Force
                Write-Host "  [OK]    Files copied to ${drive}:\" -ForegroundColor Green
                Write-Host "          Don't forget the RPi firmware files!" -ForegroundColor Yellow
            }
        }
    }
}

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
# Method: Generic ARM binary
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

function Install-ArmBinary {
    $elf = Build-Kernel -RustTarget $config.RustTarget -Arch $config.Arch
    $bin = Extract-Binary -ElfPath $elf
    
    Write-Host ""
    Write-Host "  [OK]    ARM64 binary ready" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Output files:" -ForegroundColor White
    Write-Host "    ELF:    $elf" -ForegroundColor DarkGray
    Write-Host "    Binary: $bin" -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "  Loading methods:" -ForegroundColor Yellow
    Write-Host "    U-Boot:    load mmc 0 0x40080000 trustos.bin; go 0x40080000" -ForegroundColor Cyan
    Write-Host "    JTAG:      load_image trustos.bin 0x40080000 bin" -ForegroundColor Cyan
    Write-Host "    TFTP:      tftpboot 0x40080000 trustos.bin; go 0x40080000" -ForegroundColor Cyan
    Write-Host "    QEMU:      qemu-system-aarch64 -M virt -kernel $bin -nographic" -ForegroundColor Cyan
}

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
# Method: RISC-V binary
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

function Install-RiscvBinary {
    $elf = Build-Kernel -RustTarget $config.RustTarget -Arch $config.Arch
    $bin = Extract-Binary -ElfPath $elf
    
    Write-Host ""
    Write-Host "  [OK]    RISC-V binary ready" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Output files:" -ForegroundColor White
    Write-Host "    ELF:    $elf" -ForegroundColor DarkGray
    Write-Host "    Binary: $bin" -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "  Loading methods:" -ForegroundColor Yellow
    Write-Host "    U-Boot:    load mmc 0 0x80200000 trustos.bin; go 0x80200000" -ForegroundColor Cyan
    Write-Host "    QEMU:      qemu-system-riscv64 -M virt -kernel $bin -nographic" -ForegroundColor Cyan
}

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
# Method: QEMU launch
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

function Launch-QEMU-x86 {
    Write-Host "  [BUILD] Building for x86_64..." -ForegroundColor Yellow
    & "$ROOT\build-limine.ps1"
}

function Launch-QEMU-ARM {
    $elf = Build-Kernel -RustTarget $config.RustTarget -Arch $config.Arch
    $bin = Extract-Binary -ElfPath $elf
    
    $qemu = Get-Command "qemu-system-aarch64" -ErrorAction SilentlyContinue
    if (-not $qemu) {
        Write-Host "  [FAIL] qemu-system-aarch64 not found" -ForegroundColor Red
        Write-Host "         Install QEMU: https://www.qemu.org/download/" -ForegroundColor DarkGray
        exit 1
    }
    
    Write-Host "  [QEMU]  Launching ARM64 virt machine..." -ForegroundColor Yellow
    Write-Host "          Press Ctrl+A then X to exit" -ForegroundColor DarkGray
    Write-Host ""
    
    & qemu-system-aarch64 `
        -machine virt `
        -cpu cortex-a72 `
        -m 512M `
        -nographic `
        -kernel $bin `
        -serial mon:stdio
}

function Launch-QEMU-RISCV {
    $elf = Build-Kernel -RustTarget $config.RustTarget -Arch $config.Arch
    $bin = Extract-Binary -ElfPath $elf
    
    $qemu = Get-Command "qemu-system-riscv64" -ErrorAction SilentlyContinue
    if (-not $qemu) {
        Write-Host "  [FAIL] qemu-system-riscv64 not found" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "  [QEMU]  Launching RISC-V virt machine..." -ForegroundColor Yellow
    Write-Host "          Press Ctrl+A then X to exit" -ForegroundColor DarkGray
    Write-Host ""
    
    & qemu-system-riscv64 `
        -machine virt `
        -m 512M `
        -nographic `
        -kernel $bin `
        -serial mon:stdio
}

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
# Dispatch
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

switch ($config.Method) {
    'limine-usb'       { Install-LimineUSB }
    'limine-iso'       { Install-LimineISO }
    'android-fastboot' { Install-AndroidFastboot }
    'rpi-sdcard'       { Install-RpiSD }
    'arm-binary'       { Install-ArmBinary }
    'riscv-binary'     { Install-RiscvBinary }
    'qemu-x86'         { Launch-QEMU-x86 }
    'qemu-arm'         { Launch-QEMU-ARM }
    'qemu-riscv'       { Launch-QEMU-RISCV }
}

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
# Platform support matrix
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

Write-Host ""
Write-Host "  â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•" -ForegroundColor DarkGray
Write-Host "  TrustOS Platform Support" -ForegroundColor White
Write-Host "  â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•" -ForegroundColor DarkGray
Write-Host ""
Write-Host '  PC (x86_64):       UEFI + BIOS - USB, ISO, HDD' -ForegroundColor Green
Write-Host '  Android:           fastboot flash - Pixel, Samsung*, OnePlus, Xiaomi' -ForegroundColor Green
Write-Host '  Raspberry Pi:      SD card - RPi 4, RPi 5' -ForegroundColor Green
Write-Host '  ARM64 Boards:      U-Boot / flat binary - Pine64, Rock64' -ForegroundColor Green
Write-Host '  RISC-V Boards:     OpenSBI + U-Boot - VisionFive, Milk-V' -ForegroundColor Yellow
Write-Host ""
Write-Host "  * Samsung: Requires OEM unlock in Developer Settings first" -ForegroundColor DarkGray
Write-Host "  * Apple:   Not supported (hardware-locked bootchain)" -ForegroundColor DarkGray
Write-Host ""

