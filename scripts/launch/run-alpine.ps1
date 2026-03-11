# Run Alpine Linux from disk with XFCE desktop
# Uses WHPX acceleration (works with Hyper-V)

$qemu = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$vmDir = "C:\Users\nathan\Documents\Scripts\OSrust\qemu-alpine"

Write-Host "Starting Alpine Linux with WHPX acceleration..." -ForegroundColor Cyan
Write-Host ""
Write-Host "To install XFCE desktop after boot:" -ForegroundColor Yellow
Write-Host "  apk update" -ForegroundColor White
Write-Host "  setup-xorg-base" -ForegroundColor White
Write-Host "  apk add xfce4 xfce4-terminal dbus lightdm-gtk-greeter" -ForegroundColor White
Write-Host "  rc-update add dbus" -ForegroundColor White
Write-Host "  rc-update add lightdm" -ForegroundColor White
Write-Host "  reboot" -ForegroundColor White
Write-Host ""

& $qemu `
    -accel whpx,kernel-irqchip=off `
    -cpu max `
    -m 2G `
    -smp 2 `
    -drive file="$vmDir\alpine.qcow2",format=qcow2,if=virtio `
    -netdev user,id=net0,hostfwd=tcp::2222-:22 `
    -device virtio-net-pci,netdev=net0 `
    -display sdl `
    -device virtio-vga `
    -usb `
    -device usb-tablet
