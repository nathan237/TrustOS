# ═══════════════════════════════════════════════════════════════════════
# TrustOS EL2 Hypervisor — Android Guest Demo
#
# This script boots TrustOS at EL2 in QEMU and optionally loads an
# Android/Linux kernel image as a guest running at EL1.
#
# TrustOS intercepts ALL guest hardware access (MMIO, SMC, IRQ) 
# transparently — the guest never knows it's being surveilled.
#
# ═══════════════════════════════════════════════════════════════════════
#
# MODES:
#   1. Solo mode (default): TrustOS boots alone at EL2, run hv commands
#   2. Android mode: Boot with an Android emulator kernel as guest
#
# USAGE:
#   # Solo — TrustOS only (test EL2 detection, hwscan, etc.)
#   .\run-android-el2.ps1
#
#   # With Android kernel guest
#   .\run-android-el2.ps1 -Kernel .\android\Image -Initrd .\android\ramdisk.img
#
#   # With DTB override
#   .\run-android-el2.ps1 -Kernel .\Image -Dtb .\custom.dtb
#
# WHERE TO GET ANDROID IMAGES:
#   Option A — AOSP CI (official Google):
#     1. Go to: https://ci.android.com/builds/branches/aosp-main/grid
#     2. Select: aosp_cf_arm64_phone-trunk_staging-userdebug
#     3. Download: cvd-host_package.tar.gz (contains Image + initrd)
#
#   Option B — Android SDK emulator images:
#     1. sdkmanager "system-images;android-35;google_apis;arm64-v8a"
#     2. Images at: $ANDROID_HOME/system-images/android-35/google_apis/arm64-v8a/
#     3. kernel-ranchu = ARM64 Image, ramdisk.img = initrd
#
#   Option C — Generic ARM64 Linux (for testing):
#     1. Download any arm64 Linux Image (Alpine, Buildroot, etc.)
#     2. Use with: -Kernel .\Image
#
# ═══════════════════════════════════════════════════════════════════════

param(
    # Path to guest ARM64 kernel Image (optional)
    [string]$Kernel = "",
    
    # Path to guest initrd/ramdisk (optional)
    [string]$Initrd = "",
    
    # Path to DTB (optional, QEMU generates one by default)
    [string]$Dtb = "",
    
    # RAM size for guest
    [string]$Memory = "1G",
    
    # Number of CPU cores
    [int]$Cpus = 2,
    
    # CPU model
    [string]$Cpu = "cortex-a72",
    
    # Extra kernel command line for guest
    [string]$Append = "console=ttyAMA0 earlycon=pl011,0x09000000",
    
    # Enable GDB debug server
    [switch]$Debug,
    
    # Run without display (serial only)
    [switch]$NoGui,
    
    # Enable QEMU tracing
    [switch]$Trace,
    
    # Capture serial output to file
    [string]$SerialLog = ""
)

$ErrorActionPreference = "Stop"

# ─── Banner ───
Write-Host ""
Write-Host "  ╔════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "  ║  TrustOS EL2 Hypervisor — Android Guest Demo         ║" -ForegroundColor Cyan
Write-Host "  ║  MMIO Spy • SMC Intercept • Stage-2 Translation      ║" -ForegroundColor Cyan
Write-Host "  ╚════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# ─── Step 1: Build TrustOS ───
Write-Host "[1] Building TrustOS for aarch64..." -ForegroundColor Yellow
$buildOutput = & cargo build --target aarch64-unknown-none -p trustos_kernel 2>&1
$trustosKernel = "target/aarch64-unknown-none/debug/trustos_kernel"

if (-not (Test-Path $trustosKernel)) {
    Write-Host "  FAIL: Kernel build failed" -ForegroundColor Red
    $buildOutput | Select-String "^error" | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    exit 1
}
$size = (Get-Item $trustosKernel).Length
Write-Host "  OK: TrustOS built ($([math]::Round($size/1024))KB)" -ForegroundColor Green

# ─── Step 2: Check QEMU ───
Write-Host "[2] Checking QEMU..." -ForegroundColor Yellow
$qemu = Get-Command qemu-system-aarch64 -ErrorAction SilentlyContinue
if (-not $qemu) {
    Write-Host "  FAIL: qemu-system-aarch64 not found" -ForegroundColor Red
    Write-Host "  Install: winget install QEMU.QEMU" -ForegroundColor Yellow
    exit 1
}
Write-Host "  OK: $(& qemu-system-aarch64 --version | Select-Object -First 1)" -ForegroundColor Green

# ─── Step 3: Validate guest images (if provided) ───
$hasGuest = $false
if ($Kernel -ne "") {
    Write-Host "[3] Validating guest kernel..." -ForegroundColor Yellow
    if (-not (Test-Path $Kernel)) {
        Write-Host "  FAIL: Kernel not found: $Kernel" -ForegroundColor Red
        exit 1
    }
    $kSize = (Get-Item $Kernel).Length
    Write-Host "  Kernel: $Kernel ($([math]::Round($kSize/1024/1024, 1))MB)" -ForegroundColor Green
    
    # Check ARM64 magic at offset 0x38
    $bytes = [System.IO.File]::ReadAllBytes((Resolve-Path $Kernel))
    if ($bytes.Length -ge 0x3C) {
        $magic = [BitConverter]::ToUInt32($bytes, 0x38)
        if ($magic -eq 0x644d5241) {
            Write-Host "  ARM64 Image magic: OK (0x644d5241)" -ForegroundColor Green
        } else {
            Write-Host "  WARNING: ARM64 magic not found (got 0x$($magic.ToString('X8')))" -ForegroundColor Yellow
            Write-Host "  This may not be a valid ARM64 Image" -ForegroundColor Yellow
        }
    }
    $hasGuest = $true
    
    if ($Initrd -ne "" -and (Test-Path $Initrd)) {
        $iSize = (Get-Item $Initrd).Length
        Write-Host "  Initrd: $Initrd ($([math]::Round($iSize/1024/1024, 1))MB)" -ForegroundColor Green
    }
} else {
    Write-Host "[3] Solo mode — TrustOS only (no guest kernel)" -ForegroundColor Yellow
    Write-Host "  Use 'hv el2' in shell to verify EL2" -ForegroundColor DarkGray
    Write-Host "  Use 'hv boot test' to launch self-test guest" -ForegroundColor DarkGray
}

# ─── Step 4: Build QEMU command ───
Write-Host "[4] Launching QEMU..." -ForegroundColor Yellow

$qemuArgs = @(
    "-machine", "virt,virtualization=on,gic-version=3",
    "-cpu", $Cpu,
    "-smp", "$Cpus",
    "-m", $Memory,
    "-kernel", $trustosKernel,
    "-nographic"
)

# Add VirtIO devices for guest
$qemuArgs += @(
    "-device", "virtio-net-pci,netdev=net0",
    "-netdev", "user,id=net0",
    "-device", "virtio-rng-pci"
)

# If we have a guest kernel, we can load it via QEMU's -device loader
# But for the hypervisor demo, TrustOS itself handles loading
# The guest kernel would be passed as a raw binary in guest RAM
if ($hasGuest) {
    # Pass the guest kernel as a raw binary loaded at a specific address
    # TrustOS at EL2 will configure Stage-2 tables around it
    $qemuArgs += @(
        "-device", "loader,file=$Kernel,addr=0x40200000,force-raw=on"
    )
    if ($Initrd -ne "" -and (Test-Path $Initrd)) {
        $qemuArgs += @(
            "-device", "loader,file=$Initrd,addr=0x45000000,force-raw=on"
        )
    }
    # Pass kernel size info via fw_cfg so TrustOS can find it
    $qemuArgs += @(
        "-fw_cfg", "name=opt/trustos/guest_kernel_size,string=$([System.IO.File]::ReadAllBytes((Resolve-Path $Kernel)).Length)",
        "-fw_cfg", "name=opt/trustos/guest_kernel_addr,string=0x40200000"
    )
    if ($Append -ne "") {
        $qemuArgs += @("-append", $Append)
    }
}

# Serial logging
if ($SerialLog -ne "") {
    $qemuArgs += @("-serial", "file:$SerialLog")
    # Add a second serial for interactive use
    $qemuArgs += @("-serial", "mon:stdio")
} else {
    $qemuArgs += @("-serial", "mon:stdio")
}

# Debug
if ($Debug) {
    $qemuArgs += @("-s", "-S")
    Write-Host "  GDB server on :1234, waiting for connection..." -ForegroundColor Yellow
}
if ($Trace) {
    $qemuArgs += @("-d", "int,guest_errors,cpu_reset", "-D", "qemu_el2_trace.log")
}

# ─── Print summary ───
Write-Host ""
Write-Host "  ┌─────────────────────────────────────────────────┐" -ForegroundColor DarkGray
Write-Host "  │ Mode:    $(if ($hasGuest) { 'EL2 Hypervisor + Android Guest' } else { 'EL2 Solo (TrustOS only)' })" -ForegroundColor DarkGray
Write-Host "  │ Machine: virt (GICv3, virtualization=on)" -ForegroundColor DarkGray
Write-Host "  │ CPU:     $Cpu x$Cpus" -ForegroundColor DarkGray
Write-Host "  │ RAM:     $Memory" -ForegroundColor DarkGray
if ($hasGuest) {
Write-Host "  │ Guest:   $Kernel" -ForegroundColor DarkGray
}
Write-Host "  └─────────────────────────────────────────────────┘" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  TrustOS shell commands:" -ForegroundColor White
Write-Host "    hv el2       — Verify EL2 status" -ForegroundColor DarkGray
Write-Host "    hv boot test — Launch self-test guest" -ForegroundColor DarkGray
Write-Host "    hv spy       — View live MMIO spy data" -ForegroundColor DarkGray
Write-Host "    hv smc       — View SMC call log" -ForegroundColor DarkGray
Write-Host "    hv devices   — Per-device activity stats" -ForegroundColor DarkGray
Write-Host "    hv report    — Full spy report" -ForegroundColor DarkGray
Write-Host "    hwscan auto  — Run TrustProbe hardware scan" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  Press Ctrl+A then X to exit QEMU" -ForegroundColor Yellow
Write-Host ""

# ─── Launch ───
$cmdLine = "qemu-system-aarch64 $($qemuArgs -join ' ')"
Write-Host "  $cmdLine" -ForegroundColor DarkGray
Write-Host ""

& qemu-system-aarch64 @qemuArgs
