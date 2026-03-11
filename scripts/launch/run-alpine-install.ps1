# Run Alpine Linux installation in QEMU with WHPX acceleration
# This boots from the ISO to install Alpine to disk

$qemu = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$vmDir = "C:\Users\nathan\Documents\Scripts\OSrust\qemu-alpine"

Write-Host "Starting Alpine Linux Installation..." -ForegroundColor Cyan
Write-Host "Use WHPX acceleration (works with Hyper-V)" -ForegroundColor Gray
Write-Host ""
Write-Host "INSTALLATION STEPS:" -ForegroundColor Yellow
Write-Host "1. Login as 'root' (no password)" -ForegroundColor White
Write-Host "2. Run: setup-alpine" -ForegroundColor White
Write-Host "3. Follow prompts (use 'vda' as disk)" -ForegroundColor White
Write-Host "4. When done, type 'poweroff'" -ForegroundColor White
Write-Host ""

& $qemu `
    -accel whpx,kernel-irqchip=off `
    -cpu max `
    -m 2G `
    -smp 2 `
    -drive file="$vmDir\alpine.qcow2",format=qcow2,if=virtio `
    -cdrom "$vmDir\alpine-virt-3.19.0-x86_64.iso" `
    -boot d `
    -netdev user,id=net0,hostfwd=tcp::2222-:22 `
    -device virtio-net-pci,netdev=net0 `
    -display sdl `
    -vga virtio
