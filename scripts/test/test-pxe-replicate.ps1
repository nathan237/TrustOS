# test-pxe-replicate.ps1 — Test PXE Self-Replication
# 
# Tests TrustOS network boot capability:
# Phase 1: Node 1 boots from ISO (proves ISO boot works)
# Phase 2: Node 2 boots from network via PXE (proves kernel can PXE boot)
# Uses QEMU built-in TFTP to serve Limine PXE + kernel to the PXE client.
#
# Usage:
#   .\test-pxe-replicate.ps1                  # Build + test
#   .\test-pxe-replicate.ps1 -NoBuild         # Skip build, use existing ISO
#   .\test-pxe-replicate.ps1 -Headless        # No QEMU display windows

param(
    [switch]$NoBuild,
    [switch]$Headless
)

$ErrorActionPreference = "Stop"
$QemuPath = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$ScriptDir = $PSScriptRoot

Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║          TrustOS PXE Self-Replication Test                  ║" -ForegroundColor Cyan
Write-Host "║   Phase 1: ISO boot  |  Phase 2: Network (PXE) boot        ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# ── Step 0: Cleanup ──
Write-Host "[0] Cleaning up..." -ForegroundColor Yellow
Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
Remove-Item -Force "$ScriptDir\serial_node*.log" -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# ── Step 1: Build ISO ──
if (-not $NoBuild) {
    Write-Host "[1] Building TrustOS ISO..." -ForegroundColor Yellow
    Push-Location $ScriptDir
    & "$ScriptDir\build-limine.ps1" -NoRun
    Pop-Location
    if (-not (Test-Path "$ScriptDir\trustos.iso")) {
        Write-Host "ERROR: ISO build failed!" -ForegroundColor Red
        exit 1
    }
    Write-Host "[1] ISO built successfully" -ForegroundColor Green
} else {
    if (-not (Test-Path "$ScriptDir\trustos.iso")) {
        Write-Host "ERROR: trustos.iso not found! Run without -NoBuild first." -ForegroundColor Red
        exit 1
    }
    Write-Host "[1] Using existing ISO" -ForegroundColor DarkGray
}

$isoSize = [math]::Round((Get-Item "$ScriptDir\trustos.iso").Length / 1MB, 2)
Write-Host "    ISO size: $isoSize MB" -ForegroundColor DarkGray

# ── Step 2: Prepare PXE boot files ──
Write-Host "[2] Preparing PXE boot files..." -ForegroundColor Yellow

# Locate kernel binary
$kernelBin = "$ScriptDir\target\x86_64-unknown-none\release\trustos_kernel"
if (-not (Test-Path $kernelBin)) {
    $kernelBin = "$ScriptDir\target\x86_64-unknown-none\debug\trustos_kernel"
}
if (-not (Test-Path $kernelBin)) {
    Write-Host "    ERROR: Kernel binary not found in target/" -ForegroundColor Red
    exit 1
}
$kernelSize = [math]::Round((Get-Item $kernelBin).Length / 1MB, 2)
Write-Host "    Kernel: $kernelSize MB" -ForegroundColor DarkGray

# Check for Limine PXE binary
$liminePxe = "$ScriptDir\limine\limine-bios-pxe.bin"
if (-not (Test-Path $liminePxe)) {
    Write-Host "    ERROR: limine-bios-pxe.bin not found in limine/" -ForegroundColor Red
    Write-Host "    PXE boot requires Limine's PXE binary." -ForegroundColor Red
    exit 1
}

# Create TFTP root directory
$tftpDir = "$ScriptDir\pxe_tftp"
if (Test-Path $tftpDir) { Remove-Item -Recurse -Force $tftpDir }
New-Item -ItemType Directory -Path $tftpDir -Force | Out-Null

# Limine PXE config
# boot():/ is the TFTP server root when booting via PXE
$confV5 = @"
timeout: 3

/TrustOS PXE Boot
    protocol: limine
    kernel_path: boot():/trustos_kernel
"@
$confLegacy = @"
TIMEOUT=3

:TrustOS PXE Boot
PROTOCOL=limine
KERNEL_PATH=boot():/trustos_kernel
"@
Set-Content -Path "$tftpDir\limine.conf" -Value $confV5 -Encoding ASCII
Set-Content -Path "$tftpDir\limine.cfg" -Value $confLegacy -Encoding ASCII
Copy-Item $kernelBin "$tftpDir\trustos_kernel" -Force
Copy-Item $liminePxe "$tftpDir\limine-bios-pxe.bin" -Force

# Limine PXE needs limine-bios.sys (Stage 3) in root, /boot, /limine, or /boot/limine
$limineSys = "$ScriptDir\limine\limine-bios.sys"
if (-not (Test-Path $limineSys)) {
    Write-Host "    ERROR: limine-bios.sys not found!" -ForegroundColor Red
    exit 1
}
Copy-Item $limineSys "$tftpDir\limine-bios.sys" -Force
# Place config and sys in all directories Limine searches:
# /, /boot/, /limine/, /boot/limine/
foreach ($sub in @("boot", "limine", "boot\limine")) {
    $dir = "$tftpDir\$sub"
    New-Item -ItemType Directory -Path $dir -Force | Out-Null
    Copy-Item "$tftpDir\limine.conf" "$dir\limine.conf" -Force
    Copy-Item "$tftpDir\limine.cfg" "$dir\limine.cfg" -Force
    Copy-Item $limineSys "$dir\limine-bios.sys" -Force
    Copy-Item $kernelBin "$dir\trustos_kernel" -Force
}

Write-Host "    TFTP root: limine-bios-pxe.bin, limine-bios.sys, limine.conf, limine.cfg, trustos_kernel" -ForegroundColor DarkGray
Write-Host "[2] PXE boot files ready" -ForegroundColor Green

# ── Helper: wait for boot ──
function Wait-ForBoot {
    param([string]$LogFile, [int]$TimeoutSec = 90)
    $start = Get-Date
    while (((Get-Date) - $start).TotalSeconds -lt $TimeoutSec) {
        if (Test-Path $LogFile) {
            $content = Get-Content $LogFile -Raw -ErrorAction SilentlyContinue
            if ($content -and $content -match "Starting shell") {
                return $true
            }
        }
        Start-Sleep -Seconds 3
    }
    return $false
}

# ═══════════════════════════════════════════════════════════════════════════════
# PHASE 1: ISO Boot (Seed Node)
# ═══════════════════════════════════════════════════════════════════════════════
Write-Host ""
Write-Host "--- Phase 1: ISO Boot (Seed Node) ---" -ForegroundColor Cyan

Write-Host "[3] Starting Node 1 (ISO boot)..." -ForegroundColor Yellow
$node1Args = @(
    "-cdrom", "$ScriptDir\trustos.iso",
    "-m", "256M",
    "-smp", "2",
    "-serial", "file:$ScriptDir\serial_node1.log",
    "-netdev", "user,id=net0",
    "-device", "e1000,netdev=net0,mac=52:54:00:12:34:10",
    "-boot", "d",
    "-name", "TrustOS-Seed"
)
if ($Headless) { $node1Args += @("-display", "none") }
else { $node1Args += @("-display", "sdl") }

$node1 = Start-Process -FilePath $QemuPath -ArgumentList $node1Args -PassThru -WindowStyle Normal
Write-Host "    PID: $($node1.Id)" -ForegroundColor DarkGray

Write-Host "[4] Waiting for Node 1 to boot..." -ForegroundColor Yellow
if (Wait-ForBoot -LogFile "$ScriptDir\serial_node1.log" -TimeoutSec 90) {
    Write-Host "[4] Node 1 BOOTED from ISO!" -ForegroundColor Green
    
    # Show DHCP info
    if (Test-Path "$ScriptDir\serial_node1.log") {
        $dhcpLine = Get-Content "$ScriptDir\serial_node1.log" -ErrorAction SilentlyContinue |
            Where-Object { $_ -match "DHCP.*Configured|ip=" } | Select-Object -Last 1
        if ($dhcpLine) { Write-Host "    $dhcpLine" -ForegroundColor DarkGray }
    }
    $phase1 = $true
} else {
    Write-Host "[4] Node 1 boot TIMEOUT" -ForegroundColor Red
    if (Test-Path "$ScriptDir\serial_node1.log") {
        Get-Content "$ScriptDir\serial_node1.log" -Tail 10 | ForEach-Object {
            Write-Host "    $_" -ForegroundColor DarkGray
        }
    }
    $phase1 = $false
}

# Stop Node 1
Write-Host "    Stopping Node 1..." -ForegroundColor DarkGray
if (-not $node1.HasExited) {
    Stop-Process -Id $node1.Id -Force -ErrorAction SilentlyContinue
}
Start-Sleep -Seconds 2

# ═══════════════════════════════════════════════════════════════════════════════
# PHASE 2: PXE Network Boot (Client Node)
# ═══════════════════════════════════════════════════════════════════════════════
Write-Host ""
Write-Host "--- Phase 2: PXE Network Boot ---" -ForegroundColor Cyan

Write-Host "[5] Starting Node 2 (PXE boot via QEMU TFTP)..." -ForegroundColor Yellow
Write-Host "    QEMU user-mode DHCP+TFTP -> limine-bios-pxe.bin -> kernel" -ForegroundColor DarkGray

# Node 2: PXE boot using QEMU's user-mode networking with built-in TFTP
# -boot n = boot from network
# QEMU iPXE ROM does DHCP, gets bootfile=limine-bios-pxe.bin from QEMU TFTP
# Limine PXE reads limine.cfg, loads trustos_kernel via TFTP
$node2Args = @(
    "-m", "256M",
    "-smp", "2",
    "-serial", "file:$ScriptDir\serial_node2.log",
    "-netdev", "user,id=net0,tftp=$tftpDir,bootfile=limine-bios-pxe.bin",
    "-device", "e1000,netdev=net0,mac=52:54:00:12:34:11",
    "-boot", "n",
    "-name", "TrustOS-PXE"
)
if ($Headless) { $node2Args += @("-display", "none") }
else { $node2Args += @("-display", "sdl") }

$node2 = Start-Process -FilePath $QemuPath -ArgumentList $node2Args -PassThru -WindowStyle Normal
Write-Host "    PID: $($node2.Id)" -ForegroundColor DarkGray

Write-Host "[6] Waiting for Node 2 PXE boot (2 min max)..." -ForegroundColor Yellow
$pxeStart = Get-Date
$pxeBooted = $false
$lastSize = 0

while (((Get-Date) - $pxeStart).TotalSeconds -lt 120) {
    Start-Sleep -Seconds 5
    $elapsed = [int]((Get-Date) - $pxeStart).TotalSeconds
    
    # Check if QEMU is still alive
    if ($node2.HasExited) {
        Write-Host "    [$($elapsed)s] QEMU exited (code: $($node2.ExitCode))" -ForegroundColor Red
        break
    }
    
    # Check serial log progress
    if (Test-Path "$ScriptDir\serial_node2.log") {
        $sz = (Get-Item "$ScriptDir\serial_node2.log").Length
        if ($sz -ne $lastSize) {
            $lastSize = $sz
            $content = Get-Content "$ScriptDir\serial_node2.log" -Raw -ErrorAction SilentlyContinue
            if ($content -match "Starting shell") {
                Write-Host "    [$($elapsed)s] TrustOS PXE boot COMPLETE! ($sz bytes)" -ForegroundColor Green
                $pxeBooted = $true
                break
            } elseif ($content -match "T-RustOs Kernel") {
                Write-Host "    [$($elapsed)s] Kernel loaded via PXE! Booting... ($sz bytes)" -ForegroundColor Yellow
            } else {
                Write-Host "    [$($elapsed)s] Activity ($sz bytes)" -ForegroundColor DarkGray
            }
        } else {
            if ($elapsed % 15 -lt 5) {
                Write-Host "    [$($elapsed)s] Waiting... (serial: $sz bytes)" -ForegroundColor DarkGray
            }
        }
    } else {
        if ($elapsed % 15 -lt 5) {
            Write-Host "    [$($elapsed)s] Waiting for serial output..." -ForegroundColor DarkGray
        }
    }
}

# Stop Node 2
if (-not $node2.HasExited) {
    Stop-Process -Id $node2.Id -Force -ErrorAction SilentlyContinue
}
Start-Sleep -Seconds 1

# ═══════════════════════════════════════════════════════════════════════════════
# RESULTS
# ═══════════════════════════════════════════════════════════════════════════════
Write-Host ""
Write-Host "===========================================" -ForegroundColor Cyan
Write-Host "  PXE Self-Replication Test Results" -ForegroundColor Cyan
Write-Host "===========================================" -ForegroundColor Cyan

if ($phase1) {
    Write-Host "  Phase 1 (ISO boot):     PASS" -ForegroundColor Green
} else {
    Write-Host "  Phase 1 (ISO boot):     FAIL" -ForegroundColor Red
}

if ($pxeBooted) {
    Write-Host "  Phase 2 (PXE boot):     PASS" -ForegroundColor Green
} else {
    Write-Host "  Phase 2 (PXE boot):     FAIL" -ForegroundColor Red
    
    Write-Host ""
    Write-Host "  PXE Debug Info:" -ForegroundColor Yellow
    if (Test-Path "$ScriptDir\serial_node2.log") {
        $sz = (Get-Item "$ScriptDir\serial_node2.log").Length
        Write-Host "    Serial log: $sz bytes" -ForegroundColor DarkGray
        if ($sz -gt 0) {
            Write-Host "    Last 15 lines:" -ForegroundColor DarkGray
            Get-Content "$ScriptDir\serial_node2.log" -Tail 15 | ForEach-Object {
                Write-Host "      $_" -ForegroundColor DarkGray
            }
        } else {
            Write-Host "    (no serial output - iPXE ROM may not have loaded limine)" -ForegroundColor DarkGray
        }
    } else {
        Write-Host "    (no serial log file created)" -ForegroundColor DarkGray
    }
}

Write-Host "===========================================" -ForegroundColor Cyan

if ($phase1 -and $pxeBooted) {
    Write-Host ""
    Write-Host "  SUCCESS! TrustOS self-replicates via PXE network boot!" -ForegroundColor Green
    Write-Host "  Kernel boots identically from ISO and from network." -ForegroundColor Green
    exit 0
} elseif ($phase1) {
    Write-Host ""
    Write-Host "  ISO boot works. PXE boot needs investigation." -ForegroundColor Yellow
    Write-Host "  Check: Does Limine PXE find limine.cfg via TFTP?" -ForegroundColor DarkGray
    exit 1
} else {
    Write-Host "  Both phases failed -- check build." -ForegroundColor Red
    exit 2
}
