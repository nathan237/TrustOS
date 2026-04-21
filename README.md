<div align="center">

<img src="media/logo.png" alt="TrustOS" width="280"/>

# TrustOS

### Boot on any hardware. Debug everything.

**A bare-metal OS written entirely in Rust — zero C, zero binary blobs, zero secrets.**
**Designed to run on real hardware and tell you exactly what's happening inside it.**

[![Build](https://img.shields.io/badge/build-passing-brightgreen?style=for-the-badge)]()
[![Rust](https://img.shields.io/badge/100%25%20Rust-F74C00?style=for-the-badge&logo=rust&logoColor=white)]()
[![Architectures](https://img.shields.io/badge/arch-x86__64%20%7C%20ARM64%20%7C%20RISC--V-blueviolet?style=for-the-badge)]()
[![Version](https://img.shields.io/badge/version-0.11.0-orange?style=for-the-badge)]()
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=for-the-badge)](LICENSE)
[![Lines](https://img.shields.io/badge/163K%2B%20lines-Rust-F74C00?style=for-the-badge)]()
[![Author](https://img.shields.io/badge/created%20by-Nated0ge-ff69b4?style=for-the-badge&logo=github&logoColor=white)](https://github.com/nathan237)

---

</div>

## What's New (March–April 2026)

Since v0.10.1, TrustOS has seen **40 commits** and over **1.2 million lines of changes** across 2,000+ files. Here are the highlights.

### AMD GPU — SDMA Engine Validated on Real Hardware

<div align="center">
<img src="win2.png" alt="GPU AMD SDMA OK" width="900"/>
</div>

After weeks of bring-up work on an AMD RX 580X (Polaris 10), the **SDMA pipeline is running on bare metal**. Ring buffer allocated in GART, firmware responsive, read/write pointers advancing. The Graphics Memory Controller (L1/L2 TLB, system aperture, VM flat mode) was the missing piece — 14+ debug iterations on real hardware to track it down.

**Hardware**: BTC-250PRO (Skylake), RX 580X via PCIe riser, entirely headless (UDP shell + PXE reboot).

**Next**: CP/graphics ring, compute dispatch, GPU-accelerated JARVIS training.

### Intel HDA Audio — From Silence to Sound

We wrote a complete **Intel HD Audio driver** from scratch in `no_std` Rust. Tested on ThinkPad T61 with an AD1984 codec — a notoriously tricky chip.

This was a 12-iteration debugging saga:
- **Root cause**: amp gain bits were swapped in our register encoding — output amplifiers were never actually being set
- GPIO polarity fix (T61 needs HIGH for amplifier power, not LOW)
- Triangle wave i16 overflow causing audio tearing
- Stream reset logic, full DAC configuration, codec dump tool
- Gain range validation against hardware-reported amp capabilities

Result: working audio output on real hardware, from a bare-metal OS, in pure Rust.

### Ring 3 Userland (v0.10.5)

TrustOS now has a **protected userland**. Full Ring 3 integration with an 85-check conformance audit:
- User-mode processes running in isolated address spaces
- Syscall interface validated against expected behavior
- Foundation for running untrusted code safely on bare metal

### Hardware Diagnostic Suite — 6 New Modules (v0.10.4)

The `hwdbg` toolkit now has **15+ subcommands** for deep hardware inspection:

| Tool | What it does |
|------|-------------|
| `pciraw` | Raw PCI/PCIe config space hex dump (256B legacy / 4KB extended) |
| `regdiff` | Register snapshot & bit-level diff (PCI, MSR, I/O ports) |
| `ioscan` | I/O port range scanner (legacy devices, COM/UART, IDE controllers) |
| `regwatch` | Live register monitor with automatic change detection |
| `aer` | PCIe Advanced Error Reporting — scan, decode, clear |
| `timing` | TSC-based boot profiling with per-subsystem checkpoint timeline |

### ThinkPad EC Driver + CPU Frequency Control (v0.10.2)

- Embedded Controller driver for ThinkPad laptops — fan control, thermal readout, battery status
- CPU frequency scaling via MSR writes
- Proves TrustOS runs and interacts with real laptop hardware, not just server boards

### CoreMark Benchmark — 25,000 iter/sec

Added the industry-standard **EEMBC CoreMark** benchmark. TrustOS achieves **25,000 iterations/second** on bare metal (Intel G4400). A verifiable, comparable number that shows the kernel isn't just functional — it's performant.

### Security Audit

Preemptive vulnerability fixes from a cross-OS security audit. Hardened before shipping, not after.

---

## What is TrustOS?

TrustOS is a bare-metal operating system that boots directly on real hardware — no Linux, no BIOS services, no runtime dependencies. Once running, it gives you complete visibility into your machine: every PCI device, every CPU register, every GPU engine, every ACPI table, every DIMM slot.

The core use case: **boot on any x86_64 machine (USB or PXE), and get a full hardware diagnostic report in seconds — remotely, over the network.**

It ships with a Python remote monitor (`scripts/remote_screen.py`) that lets you see the screen and interact with the shell from any PC on the same network — no physical access required.

---

## AMD GPU Bring-Up — Milestones

From-scratch AMD GPU driver in pure `no_std` Rust — no Linux, no Mesa, no libdrm. Direct MMIO register control on real silicon.

| Milestone | Status |
|-----------|--------|
| PCI enumeration & BAR decode (VRAM 256 MB, MMIO 512 KB, Doorbell 2 MB) | Done |
| MMIO register access — validated against Linux amdgpu source | Done |
| VRAM detection — 8 GB GDDR5 from CONFIG_MEMSIZE | Done |
| Graphics Memory Controller — L1/L2 TLB, system aperture, VM flat mode | Done |
| Firmware loading — SDMA0/1, PFP, ME, MEC via MMIO | Done |
| SDMA ring buffer — GART ring, RPTR/WPTR advancing | Done |
| PSP bootstrap (Navi) — bootloader protocol scaffolded | In Progress |
| Compute dispatch — PM4 packets, HQD init | In Progress |
| GPU-accelerated AI — INT8 GEMM across 36 CUs for JARVIS | Planned |

**Key Discovery**: After 14+ iterations, SDMA's read pointer was stuck at 0. Root cause: the Graphics Memory Controller was never initialized — L1 TLB had system-access-mode disabled, so firmware couldn't resolve ring buffer addresses. Not a PCIe issue, not a register bug — the memory subsystem.

---

## Hardware Debug Capabilities

TrustOS probes hardware at the register level. No abstraction layers between you and the silicon.

| Module | What it reads |
|--------|--------------|
| **PCI Bus** | Full enumeration: vendor/device IDs, class, BARs, capability chains (MSI, PCIe, PM), bus mastering |
| **CPU** | CPUID all leaves, brand string, feature flags (SSE/AVX/AES), family/model/stepping, MSR scan |
| **SMBIOS/DMI** | Board manufacturer, BIOS version, DIMM slots (size/speed/type), chassis, serials |
| **ACPI** | RSDP -> XSDT walk, all tables (MADT, FADT, HPET, MCFG, SSDT), raw hex dump |
| **AMD GPU** | MMIO registers, GRBM status, SDMA engine state, VRAM size, memory controller config |
| **NVIDIA GPU** | PMC/PBUS registers, GPU identity, diagnostic dump |
| **Intel HDA** | Codec enumeration, widget tree, amplifier capabilities, pin configuration |
| **ThinkPad EC** | Fan speed, thermal sensors, battery, embedded controller registers |
| **Memory Map** | Full UEFI memory map: type, physical range, page count |
| **Boot Timing** | TSC-based per-subsystem boot profiling (us precision) |
| **Network** | NIC detection, MAC address, PCI config |
| **Storage** | NVMe/AHCI controller detection, BAR decode |

<img src="media/screenshots/hwdbg_real_hardware.jpg" alt="TrustOS hardware diagnostic on real hardware" width="720"/>

*CPUID full decode, CPU registers dump — bare-metal framebuffer output.*

<img src="media/screenshots/hwdbg_pci_boot.jpg" alt="TrustOS PCI enumeration on real hardware" width="720"/>

*PCI bus enumeration with AMD GPU + RTL8168 detected, heap status, and TSC boot checkpoints.*

---

## Remote Monitor

TrustOS streams its framebuffer over UDP and accepts shell commands remotely:

- **UDP 7779** — Screencap (chunked SCRN protocol)
- **UDP 7777** — Remote shell
- **UDP 6666** — Netconsole (kernel debug output)

Debug headless machines — no monitor, no keyboard, just Ethernet.

```bash
python scripts/remote_screen.py --ip 10.0.0.110 --interval 2
```

<img src="media/screenshots/monitor_gpu_init.png" alt="TrustOS Monitor showing live GPU init" width="720"/>

*Monitor v3.0 — GPU init streamed over UDP netconsole in real time.*

---

## Quick Start

### QEMU (30 seconds)

```bash
qemu-system-x86_64 -cdrom trustos.iso -m 512M -cpu max -smp 4 \
  -display gtk -vga std -serial stdio \
  -netdev user,id=net0 -device rtl8139,netdev=net0
```

Type `help` for all commands. `hwdbg auto` dumps CPU + memory + PCI + SMBIOS in one shot.

### Bare Metal (USB)

1. Download ISO from [**Releases**](https://github.com/nathan237/TrustOS/releases)
2. Flash with [**Rufus**](https://rufus.ie/) — **DD Image mode**
3. Boot (F12 / DEL). Works on **UEFI** and **Legacy BIOS**.

### PXE Network Boot

```bash
python scripts/pxe_server.py --tftp-root pxe_tftp --server-ip 10.0.0.1
```

---

## Platform Support

| Target | Method | Status |
|--------|--------|--------|
| x86_64 PC (USB/ISO) | Limine hybrid (UEFI + Legacy BIOS) | **Production** |
| PXE network boot | TFTP + DHCP auto-config | **Working** |
| QEMU / VirtualBox | VM | **Production** |
| ARM64 | `fastboot flash boot` / SD card | Experimental |
| RISC-V | OpenSBI + U-Boot | WIP |

Tested on: ThinkPad T61, BTC-250PRO LR (mining board), QEMU/VirtualBox.

---

## Project Structure

```
TrustOS/
  kernel/src/
    main.rs           — Entry point (Limine boot protocol)
    drivers/           — AMD GPU (Polaris/Navi), NVIDIA, Intel HDA, ThinkPad EC
    netstack/          — TCP/IP, UDP, ARP, DHCP, DNS, IPv6
    memory/            — Physical frame allocator, paging, heap
    interrupts/        — IDT, APIC, exception handlers
    shell/             — 200+ commands (hwdbg, pci, gpu, sensors, etc.)
    hwdiag/            — Hardware diagnostic modules (pciraw, regdiff, ioscan, aer, timing...)
    jarvis/            — JARVIS AI (4.4M-param transformer, guardian system)
    framebuffer/       — Rendering, POST codes
    scheduler/         — Process scheduler, Ring 3 userland
    vfs/               — Virtual filesystem
  userland/            — Ring 3 processes, syscall interface
  scripts/
    remote_screen.py   — Remote monitor (screencap + input injection)
    pxe_server.py      — DHCP + TFTP for PXE boot
```

---

## Build

```powershell
cargo build --release -p trustos_kernel    # Build kernel
.\trustos.ps1 build                         # Build + ISO + VM
.\trustos.ps1 build -NoRun                  # Build + ISO only
```

Requires: Rust nightly, `x86_64-unknown-none` target, `llvm-tools-preview`.

---

## Changelog

### v0.11.0 — Audio, Userland, GPU, Benchmarks (April 2026)

- **AMD SDMA validated** — Ring buffer running on RX 580X, GMC fully initialized, firmware responsive
- **Intel HDA audio** — Complete driver for AD1984 codec, 12-iteration debug to working audio on ThinkPad T61
- **Ring 3 userland** — Protected user-mode processes, 85-check conformance audit
- **Hardware diagnostics** — 6 new modules (pciraw, regdiff, ioscan, regwatch, aer, timing), 15+ hwdbg subcommands
- **ThinkPad EC** — Embedded Controller driver, fan/thermal/battery readout, CPU frequency control
- **CoreMark** — 25,000 iter/sec on bare metal (EEMBC standard benchmark)
- **Security** — Preemptive fixes from cross-OS vulnerability audit
- **CI** — Fixed `build-std` compatibility with rust-src component

### v0.10.1 — Settings GUI & Network (March 13, 2026)

- Settings GUI, NetScan GUI, shell scrollback fix, ACPI shutdown hardening
- T61 hardware optimization, matrix rain, GitHub Pages site

---

## License

Apache 2.0 — see [LICENSE](LICENSE).

---

## AI Disclosure

This project was built with AI assistance (GitHub Copilot, Claude). All code is reviewed and understood by the author. No generated code is shipped without verification.
