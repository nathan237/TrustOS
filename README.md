<div align="center">

<img src="media/logo.png" alt="TrustOS" width="280"/>

# TrustOS

### Boot on any hardware. Debug everything.

**A bare-metal OS written entirely in Rust — zero C, zero binary blobs, zero secrets.**
**Designed to run on real hardware and tell you exactly what's happening inside it.**

[![Build](https://img.shields.io/badge/build-passing-brightgreen?style=for-the-badge)]()
[![Rust](https://img.shields.io/badge/100%25%20Rust-F74C00?style=for-the-badge&logo=rust&logoColor=white)]()
[![Architectures](https://img.shields.io/badge/arch-x86__64%20%7C%20ARM64%20%7C%20RISC--V-blueviolet?style=for-the-badge)]()
[![Version](https://img.shields.io/badge/version-0.10.7-orange?style=for-the-badge)]()
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=for-the-badge)](LICENSE)
[![Author](https://img.shields.io/badge/created%20by-Nated0ge-ff69b4?style=for-the-badge&logo=github&logoColor=white)](https://github.com/nathan237)

---

</div>

## What is TrustOS?

TrustOS is a bare-metal operating system that boots directly on real hardware — no Linux, no BIOS services, no runtime dependencies. Once running, it gives you complete visibility into your machine: every PCI device, every CPU register, every GPU engine, every ACPI table, every DIMM slot.

The core use case: **boot on any x86_64 machine (USB or PXE), and get a full hardware diagnostic report in seconds — remotely, over the network.**

It ships with a Python remote monitor (`scripts/remote_screen.py`) that lets you see the screen and interact with the shell from any PC on the same network — no physical access required.

---

## Hardware Debug Capabilities

TrustOS probes hardware at the register level. No abstraction layers, no kernel drivers between you and the silicon.

| Module | Command | What it reads |
|--------|---------|--------------|
| **PCI Bus** | `pci` | Full enumeration: vendor/device IDs, class, BARs, capability chains (MSI, PCIe, PM), bus mastering state |
| **CPU** | `hwdbg cpu` | CPUID all leaves, brand string, feature flags (SSE/AVX/AES), family/model/stepping, MSR scan |
| **SMBIOS/DMI** | `hwdbg smbios` | Board manufacturer, BIOS version, DIMM slots (size/speed/type), chassis type, serial numbers |
| **ACPI** | `hwdbg acpi` | RSDP → XSDT walk, all tables (MADT, FADT, HPET, MCFG, SSDT), raw hex dump |
| **AMD GPU** | `gpu info` / `gpu probe` | MMIO map (BAR5), GRBM_STATUS, RLC state, SDMA engine status, VRAM size, PSP state, firmware staging |
| **NVIDIA GPU** | `gpu info` | PMC/PBUS registers, GPU identity, diagnostic dump |
| **Memory Map** | `hwdbg memmap` | Full UEFI memory map: type, physical range, page count |
| **Boot Timing** | `hwdbg timing` | TSC-based per-subsystem boot profiling (µs precision) |
| **POST Codes** | *(automatic)* | Visual POST codes on framebuffer (0x11–0xFF) during boot |
| **Network** | `hwdbg net` | NIC detection, MAC address read, PCI config |
| **Storage** | `pci` + `nvme info` | NVMe/AHCI controller detection, BAR decode |

**Proof — running on real hardware (Intel G4400, AMD RX 580X, BTC-250PRO LR mining board):**

<img src="media/screenshots/hwdbg_real_hardware.jpg" alt="TrustOS hardware diagnostic report on real hardware" width="720"/>

*CPUID full decode, CPU registers dump — output on physical screen via framebuffer.*

---

## Remote Monitor

TrustOS streams its framebuffer over UDP and accepts shell commands remotely. The combo is:

- **Kernel side** — UDP 7779 screencap (chunked SCRN protocol), UDP 7777 remote shell
- **Host side** — `scripts/remote_screen.py` Python client (tkinter GUI)

This means you can debug a headless machine, a server rack, or a mining board **without a monitor, keyboard, or physical access** — just an Ethernet cable.

```bash
# Start the remote monitor (host PC)
python scripts/remote_screen.py --ip 10.0.0.110 --interval 2

# Send a shell command remotely
python scripts/debug/b75_bridge.py send "gpu probe" --wait 5

# Tail the kernel netconsole (UDP 6666)
python scripts/debug/b75_bridge.py log 100
```

**Live session — GPU init sequence captured remotely:**

<img src="media/screenshots/monitor_gpu_init.png" alt="TrustOS Monitor v3.0 showing live AMD GPU init logs" width="720"/>

*Monitor v3.0 showing AMD RX 580X init: MMIO mapped at `0xFFFF8000DFE00000`, Polaris SDMA staged init, SDMA0/1 F32 halted and ready. All output streamed over UDP netconsole in real time.*

---

## Quick Start

### QEMU (30 seconds)

```bash
# Download the ISO from Releases, then:
qemu-system-x86_64 -cdrom trustos.iso -m 512M -cpu max -smp 4 \
  -display gtk -vga std -serial stdio \
  -netdev user,id=net0 -device rtl8139,netdev=net0
```

First command to run: `hwdbg auto` — dumps CPU, memory map, PCI bus, and SMBIOS in one shot.

### Bare Metal (USB)

1. Download the ISO from [**Releases**](https://github.com/nathan237/TrustOS/releases)
2. Flash with [**Rufus**](https://rufus.ie/) → **DD Image mode**
3. Boot from USB (F12 / DEL at startup)
4. Works on **UEFI** and **Legacy BIOS**

> Disable Secure Boot in BIOS if needed — TrustOS uses Limine, not a signed bootloader.

### PXE Network Boot

```bash
# On your dev machine (admin/root required):
python scripts/pxe_server.py --tftp-root pxe_tftp --server-ip 10.0.0.1
```

Power on the target machine with PXE enabled in BIOS. TrustOS boots over the network — no USB needed.

### Useful Commands

| Command | Description |
|---------|-------------|
| `hwdbg auto` | Full hardware diagnostic dump (CPU + memory + PCI + SMBIOS) |
| `gpu info` | AMD/NVIDIA GPU detection, MMIO, VRAM, engine status |
| `gpu probe` | Deep GPU register probe (GRBM, RLC, SDMA, PSP state) |
| `pci` | Full PCI bus enumeration with BAR decode |
| `hwdbg smbios` | Board, BIOS, DIMM, chassis from SMBIOS tables |
| `hwdbg acpi` | Walk all ACPI tables from RSDP |
| `hwdbg cpu` | CPUID full decode + CPU state dump |
| `gpu sdma status` | SDMA engine status (Polaris/Navi) |
| `help` | All available commands |

---

## Boot on Any Hardware

| Target | Method | Status |
|--------|--------|--------|
| x86_64 PC (USB/ISO) | Limine hybrid (UEFI + Legacy BIOS) | **Production** |
| PXE network boot | TFTP push, DHCP auto-config | **Working** |
| QEMU / VirtualBox | Virtual machine | **Production** |
| ARM64 | `fastboot flash boot` / SD card | Experimental |
| RISC-V | OpenSBI + U-Boot | WIP |

Tested on: ThinkPad T61, BTC-250PRO LR (mining board), QEMU/VirtualBox.

---

## Project Structure

```
TrustOS/
  kernel/src/
    drivers/          — AMD GPU (Polaris/Navi PSP/SDMA), NVIDIA, RTL8139/8169, AHCI, XHCI
    debug/            — Remote shell (UDP 7777), netconsole (UDP 6666), POST codes, watchdog
    netstack/         — TCP/IP, UDP, ARP, DHCP, DNS, IPv6
    memory/           — Physical frame allocator, paging, heap
    interrupts/       — IDT, APIC, exception handlers
    shell/            — 200+ commands including hwdbg, gpu, pci, sensors
  scripts/
    remote_screen.py  — Remote monitor GUI (screencap UDP 7779 + input injection)
    pxe_server.py     — DHCP + TFTP server for PXE boot
  debugonly/          — Minimal diagnostic kernel (no GUI, boots in <2s)
```

---

## Build

```powershell
# Build the kernel
cargo build --release -p trustos_kernel

# Build + ISO + launch VM
.\trustos.ps1 build

# Build ISO only (no VM)
.\trustos.ps1 build -NoRun
```

Requires: Rust nightly, `x86_64-unknown-none` target, `llvm-tools-preview` component.

---

## Changelog

### v0.10.7 — AMD PSP Driver, Boot Timing & GPU Debug Toolkit (March 25, 2026)

- **AMD PSP driver** — 791-line Platform Security Processor driver. PSP bootloader handshake, SOS boot sequence, GPCOM ring, firmware staging pipeline. Commands: `gpufw psp`, `gpufw psp init/boot/sos`.
- **AMD GPU firmware overhaul** — `firmware.rs` rewritten (+1,515 lines): Polaris direct MMIO path (no PSP), Navi10 PSP path, VRAM physical address tracking, staged init.
- **SDMA engine expansion** — +464 lines: ring buffer management, firmware load, diagnostic suite (`sdma status/alloc/reset/fw/ring/test`).
- **Boot timing** — TSC-based `boot_timing!` macro. Per-subsystem µs timing at startup. TSC auto-calibration.
- **GPU debug toolkit** — 10+ new subcommands: `gpu dump`, `gpu pci`, `gpu probe`, `gpu vramregs`, `gpu mmio`, `gpu mc`, `gpu sdma`.
- **+4,400 lines** across 30 files.

### v0.10.6 — Remote Desktop & Hardware Drivers (March 24, 2026)

- **Remote Desktop** — Full input injection via UDP 7777 (`key:`, `keys:`, `mouse:`, `mouseto:`). Screencap on UDP 7779. Remote ACPI reboot. Python client `remote_screen.py`.
- **RTL8139/RTL8169 drivers rewritten** — +514 lines. Hardware-tested on real boards.
- **Visual POST codes** — Real-time boot progress on framebuffer (0x11–0xFF).
- **AMD GPU driver hardened** — PCI device ID fallback, VRAM register fix, init deferred after network.
- **NVIDIA NV50+ expanded** — +214 lines of register probing.

### v0.10.5 — Hypervisor & Userland (March 20, 2026)

- **AMD SVM support** — VMCB, NPT, unified backend with Intel VT-x.
- **Ring 3 userland** — 104 Linux-compatible syscalls, ELF64 loader, COW fork, signals, pipes.
- **VirtIO block + console** — Stable virtio-blk and virtio-console for VM guests.

---

## License

Apache 2.0 — see [LICENSE](LICENSE).

---

## AI Disclosure

This project was built with AI assistance (GitHub Copilot for implementation, Claude for code review and architecture). All code is reviewed and understood by the author. No generated code is shipped without verification.

---

<div align="center">

*Built by [Nated0ge](https://github.com/nathan237) — one developer, bare metal, zero compromise.*

</div>
