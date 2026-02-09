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

# Default TCG (proven fastest for this workload — MTTCG/WHPX both slower)
# vga std + virtio-gpu-pci = proven fastest device combo (virtio-vga is slower)
$qemuArgs = @(
    "-cdrom", $IsoPath,
    "-m", "256M",
    "-machine", "q35",
    "-smp", "4",
    "-display", "gtk",
    "-vga", "std",
    "-device", "virtio-gpu-pci",
    "-device", "virtio-net-pci,netdev=net0",
    "-netdev", "user,id=net0",
    "-drive", "if=pflash,format=raw,file=$PSScriptRoot\OVMF.fd",
    "-serial", "file:$serialFile",
    "-no-reboot",
    "-monitor", "tcp:127.0.0.1:4444,server,nowait"
)

Write-Host "Starting QEMU..." -ForegroundColor Yellow
& $QemuExe @qemuArgs
