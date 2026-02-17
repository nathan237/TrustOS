# ═══════════════════════════════════════════════════════════════
# TrustOS — Automated Setup (Windows PowerShell)
#
# Usage:
#   irm https://raw.githubusercontent.com/nathan237/TrustOS/main/setup.ps1 | iex
#   # — or —
#   git clone https://github.com/nathan237/TrustOS.git; cd TrustOS; .\setup.ps1
#
# What it does:
#   1. Checks/installs Rust nightly toolchain
#   2. Checks/installs QEMU (via winget)
#   3. Clones Limine bootloader
#   4. Builds TrustOS kernel
#   5. Provides launch instructions
# ═══════════════════════════════════════════════════════════════
$ErrorActionPreference = "Stop"

function Write-Step  { param($msg) Write-Host "▸ $msg" -ForegroundColor Cyan }
function Write-Ok    { param($msg) Write-Host "✓ $msg" -ForegroundColor Green }
function Write-Warn  { param($msg) Write-Host "⚠ $msg" -ForegroundColor Yellow }
function Write-Err   { param($msg) Write-Host "✗ $msg" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "═══════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "       TrustOS — Windows Setup             " -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

# ── Step 1: Check we are in the repo ──
if (-not (Test-Path "Cargo.toml") -or -not (Select-String -Path "Cargo.toml" -Pattern "trustos" -Quiet)) {
    if (Test-Path "TrustOS") {
        Set-Location TrustOS
    } else {
        Write-Step "Cloning TrustOS repository..."
        git clone https://github.com/nathan237/TrustOS.git
        Set-Location TrustOS
    }
}
Write-Ok "In TrustOS directory: $(Get-Location)"

# ── Step 2: Rust toolchain ──
Write-Step "Checking Rust toolchain..."
if (Get-Command rustup -ErrorAction SilentlyContinue) {
    Write-Ok "rustup found"
    rustup toolchain install nightly --profile minimal --component rust-src 2>$null
    Write-Ok "Nightly toolchain ready"
} else {
    Write-Step "Installing Rust via rustup..."
    $rustup_init = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustup_init
    & $rustup_init -y --default-toolchain nightly
    $env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
    rustup component add rust-src --toolchain nightly 2>$null
    Write-Ok "Rust nightly installed"
}

# ── Step 3: QEMU ──
Write-Step "Checking QEMU..."
$qemuPath = $null
$qemuLocations = @(
    "C:\Program Files\qemu\qemu-system-x86_64.exe",
    "C:\Program Files (x86)\qemu\qemu-system-x86_64.exe",
    "$env:LOCALAPPDATA\Programs\qemu\qemu-system-x86_64.exe"
)
foreach ($loc in $qemuLocations) {
    if (Test-Path $loc) { $qemuPath = $loc; break }
}
if (-not $qemuPath -and (Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue)) {
    $qemuPath = (Get-Command qemu-system-x86_64).Source
}

if ($qemuPath) {
    Write-Ok "QEMU found: $qemuPath"
} else {
    Write-Warn "QEMU not found."
    $installQemu = Read-Host "Install QEMU via winget? (Y/n)"
    if ($installQemu -ne "n") {
        winget install SoftwareFreedomConservancy.QEMU --accept-package-agreements --accept-source-agreements
        Write-Ok "QEMU installed — you may need to restart your terminal"
    } else {
        Write-Warn "QEMU not installed — you can still build the ISO and run it in VirtualBox"
    }
}

# ── Step 4: OVMF firmware ──
Write-Step "Checking OVMF firmware..."
if (Test-Path "OVMF.fd") {
    Write-Ok "OVMF.fd found in project root"
} elseif (Test-Path "OVMF_CODE.fd") {
    Write-Ok "OVMF_CODE.fd found in project root"
} else {
    Write-Warn "No OVMF firmware found in project root."
    Write-Host "  For UEFI boot in QEMU, download OVMF.fd and place it in this directory." -ForegroundColor Yellow
    Write-Host "  Download: https://retrage.github.io/edk2-nightly/" -ForegroundColor Yellow
    Write-Host "  Or use VirtualBox/Hyper-V which include their own UEFI firmware." -ForegroundColor Yellow
}

# ── Step 5: Limine bootloader ──
Write-Step "Checking Limine bootloader..."
if (Test-Path "limine/BOOTX64.EFI") {
    Write-Ok "Limine bootloader present"
} else {
    Write-Step "Downloading Limine bootloader..."
    if (Test-Path "limine") { Remove-Item -Recurse -Force "limine" }
    git clone https://github.com/limine-bootloader/limine.git --branch=v8.x-binary --depth=1
    if (Test-Path "limine/BOOTX64.EFI") {
        Write-Ok "Limine bootloader ready"
    } else {
        Write-Err "Limine download failed"
    }
}

# ── Step 6: Build kernel ──
Write-Step "Building TrustOS kernel (release)..."
cargo build --release -p trustos_kernel
$kernelBin = "target\x86_64-unknown-none\release\trustos_kernel"
if (Test-Path $kernelBin) {
    $ksize = (Get-Item $kernelBin).Length / 1MB
    Write-Ok ("Kernel built ({0:N1} MB)" -f $ksize)
} else {
    Write-Err "Kernel build failed. Check errors above."
}

# ── Done! ──
Write-Host ""
Write-Host "═══════════════════════════════════════════" -ForegroundColor Green
Write-Host "       TrustOS is ready!                   " -ForegroundColor Green
Write-Host "═══════════════════════════════════════════" -ForegroundColor Green
Write-Host ""
Write-Host "  Run in QEMU:" -ForegroundColor White
Write-Host "    .\run-qemu-gui.ps1          # QEMU with GUI" -ForegroundColor Gray
Write-Host ""
Write-Host "  Run in VirtualBox:" -ForegroundColor White
Write-Host "    .\run-vbox.ps1              # Full VirtualBox setup" -ForegroundColor Gray
Write-Host ""
Write-Host "  Build ISO only:" -ForegroundColor White
Write-Host "    Use WSL: wsl ./build.sh     # Creates trustos.iso" -ForegroundColor Gray
Write-Host ""
Write-Host "  First commands to try in TrustOS:" -ForegroundColor White
Write-Host "    showcase       — Automated feature tour" -ForegroundColor Gray
Write-Host "    desktop        — Launch desktop environment" -ForegroundColor Gray
Write-Host "    trustlab       — Kernel introspection laboratory" -ForegroundColor Gray
Write-Host "    neofetch       — System info" -ForegroundColor Gray
Write-Host ""
