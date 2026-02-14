# TrustOS v0.2.0 Release Notes

**Release Date:** 2026-02-14

## 🎉 What's New

### Ring 3 Userspace Execution
- **Real CPL-3 process execution** via `IRETQ` with setjmp/longjmp-style kernel return
- **`exec_ring3_process()` / `return_from_ring3()`** — safe Ring 3 entry/exit mechanism
- **SYSCALL/SYSRET** fast system call path with dedicated kernel stack
- **Page-aligned physical memory allocation** for user address spaces
- **EXIT/EXIT_GROUP syscall handlers** for clean process termination

### Embedded ELF64 Binary
- **183-byte static ELF64** hello-world binary embedded in kernel
- **Full ELF loader** maps LOAD segments to user address space at `0x400000`
- **Shell commands:** `exec test` (raw machine code) and `exec hello` (ELF parse + load)

### TrustFS Reliability
- **`free_block()`** — clears bitmap + increments free block counter
- **`free_inode_blocks()`** — frees all direct + indirect blocks on inode deletion
- **`unlink()`** — properly reclaims storage when nlink reaches 0
- **`truncate()`** — frees blocks beyond new size when shrinking files

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

## 📋 Test Ring 3 Execution

```
exec test    # Run raw machine code in Ring 3
exec hello   # Load and execute embedded ELF64 binary in Ring 3
```

Both commands print "Hello from Ring 3!" and exit cleanly with code 0.

## 📋 Available Commands

Type `help` in the shell to see all 100+ commands including:
- File system: `ls`, `cd`, `cat`, `mkdir`, `rm`, `cp`, `mv`
- Network: `ping`, `curl`, `wget`, `ifconfig`
- Execution: `exec test`, `exec hello`
- System: `ps`, `top`, `free`, `uname`
- GUI: `desktop` (launches graphical desktop)

---

*TrustOS - A secure, experimental kernel written in Rust*
