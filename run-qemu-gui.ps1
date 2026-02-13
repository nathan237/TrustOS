# TRustOs - QEMU GUI with Limine ISO
param(
    [string]$IsoPath = "$PSScriptRoot\trustos.iso"
)

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"

if (-not (Test-Path $QemuExe)) {
    Write-Error "QEMU not found at $QemuExe"
    exit 1
}

if (-not (Test-Path $IsoPath)) {
    Write-Error "ISO not found: $IsoPath. Run build-limine.ps1 first."
    exit 1
}

Write-Host "=== TRustOs - QEMU GUI ===" -ForegroundColor Cyan
Write-Host "ISO: $IsoPath" -ForegroundColor Green

# Capture serial output to file
$serialFile = "$PSScriptRoot\serial_output.txt"

# Default TCG with multi-threading (proven fastest for this workload)
# vga std + virtio-gpu-pci = proven fastest device combo
# Optimized for Ryzen 5800X / 32GB RAM
$qemuArgs = @(
    "-cdrom", $IsoPath,
    "-m", "512M",
    "-machine", "q35",
    "-cpu", "max",
    "-smp", "4,sockets=1,cores=4,threads=1",
    "-accel", "tcg,thread=multi",
    "-display", "gtk",
    "-vga", "std",
    "-device", "virtio-gpu-pci,xres=1280,yres=800",
    "-device", "virtio-net-pci,netdev=net0",
    "-netdev", "user,id=net0",
    "-device", "intel-hda",
    "-device", "hda-duplex",
    "-drive", "if=pflash,format=raw,file=$PSScriptRoot\OVMF.fd",
    "-rtc", "base=utc,clock=vm",
    "-serial", "file:$serialFile",
    "-no-reboot",
    "-monitor", "tcp:127.0.0.1:4444,server,nowait"
)

Write-Host "Starting QEMU..." -ForegroundColor Yellow
& $QemuExe @qemuArgs
