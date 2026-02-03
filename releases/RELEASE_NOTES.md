# TrustOS v0.1.0 Release Notes

**Release Date:** 2026-02-03

## 🎉 What's New

### Shell UX Improvements
- **Blinking cursor** for better visibility
- **Dynamic autocomplete** with arrow key navigation (100+ commands)
- **Timestamp in prompt** `[HH:MM:SS]`
- **Scrollback buffer** - PageUp/PageDown to scroll terminal history (1000 lines)
- **Comprehensive help** command with full OS capabilities

### Linux Subsystem
- Alpine Linux binary execution
- 100+ syscalls mapped in transpiler
- `linux extract` and `alpine test` commands

### Network
- VirtIO-net driver fully functional
- DHCP client working
- HTTP/HTTPS requests supported

---

## 📦 Contents

This ZIP contains the bootable TrustOS image ready for QEMU:
- `boot/trustos_kernel` - The kernel binary
- `limine.conf` - Bootloader configuration
- `EFI/BOOT/BOOTX64.EFI` - UEFI bootloader

## 🚀 Quick Start

```powershell
# Extract and run with QEMU
qemu-system-x86_64 -bios OVMF.fd -drive format=raw,file=fat:rw:. -m 512M -serial stdio -device virtio-net-pci,netdev=net0 -netdev user,id=net0
```

## 📋 Available Commands

Type `help` in the shell to see all 100+ commands including:
- File system: `ls`, `cd`, `cat`, `mkdir`, `rm`, `cp`, `mv`
- Network: `ping`, `curl`, `wget`, `ifconfig`
- Linux: `linux shell`, `alpine`, `transpile`
- System: `ps`, `top`, `free`, `uname`
- GUI: `gui` (launches graphical desktop)

---

*TrustOS - A secure, experimental kernel written in Rust*
