# Hardware Test Guide

## TrustOS — Testing on Real Hardware

### Prerequisites
- USB drive (4 GB minimum)
- x86_64 PC with UEFI/BIOS support

### Steps

#### 1. Build the Image
```bash
cd kernel
cargo bootimage --release
```

Image: `target/x86_64-unknown-none/release/bootimage-trustos_kernel.bin`

#### 2. Create Bootable USB (Windows)
```powershell
# Identify the USB drive
Get-Disk

# Write the image (replace X with the disk number)
$img = "target\x86_64-unknown-none\release\bootimage-trustos_kernel.bin"
dd if=$img of=\\.\PhysicalDriveX bs=4M
```

Or use Rufus in DD mode.

#### 3. Boot
- Insert USB drive
- Restart PC
- Press F12/F2 for boot menu
- Select USB drive

### Expected Output
```
TRustOs v0.1.0
Initializing...
IDT loaded
Scheduler ready
IPC ready
Security initialized
Syscall interface initialized
Trace ready
GUI drivers initialized
Kernel ready. Starting init...
```

### Troubleshooting
- Black screen: Check BIOS legacy/UEFI settings
- No serial output: Normal (no VGA screen output implemented)
- Reboot loop: Bootloader incompatibility

### Alternative: VirtualBox
VirtualBox 7+ has better bootloader 0.9 support:
```bash
VBoxManage convertfromraw bootimage-trustos_kernel.bin trustos.vdi
VBoxManage createvm --name TrustOS --register
VBoxManage storagectl TrustOS --name SATA --add sata
VBoxManage storageattach TrustOS --storagectl SATA --port 0 --type hdd --medium trustos.vdi
VBoxManage startvm TrustOS
```

### Alternative: ISO Boot (Recommended)
Build the ISO using Limine bootloader:
```powershell
.\build-limine.ps1
```
Then boot from `trustos.iso` in QEMU or VirtualBox.

---

*Last updated: February 2026*
