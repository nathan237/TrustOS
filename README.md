<div align="center">

<img src="media/logo.png" alt="TrustOS" width="280"/>

# TrustOS

### The OS you can actually read — in your language.

**A bare-metal operating system written entirely in Rust — 286,000+ lines, zero C, zero binary blobs, zero secrets.**

*Built by one developer. Auditable by anyone. Learnable in English & French.*

[![Build](https://img.shields.io/badge/build-passing-brightgreen?style=for-the-badge)]()
[![Rust](https://img.shields.io/badge/100%25%20Rust-F74C00?style=for-the-badge&logo=rust&logoColor=white)]()
[![Lines](https://img.shields.io/badge/code-286%2C000%2B%20lines-blue?style=for-the-badge)]()
[![Architectures](https://img.shields.io/badge/arch-x86__64%20%7C%20ARM64%20%7C%20RISC--V-blueviolet?style=for-the-badge)]()
[![Version](https://img.shields.io/badge/version-0.10.7-orange?style=for-the-badge)]()
[![Translated](https://img.shields.io/badge/source-4%20versions-9cf?style=for-the-badge)]()
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=for-the-badge)](LICENSE)
[![Author](https://img.shields.io/badge/created%20by-Nated0ge-ff69b4?style=for-the-badge&logo=github&logoColor=white)](https://github.com/nathan237)

[![Watch the demo](https://img.shields.io/badge/%E2%96%B6%20Watch%20Demo-YouTube-red?style=for-the-badge&logo=youtube&logoColor=white)](https://youtu.be/RBJJi8jW1_g)

<img src="media/screenshots/screenshot_desktop.png" alt="TrustOS Desktop" width="720"/>

[**Source Code for Developers**](#-source-code-translation-for-developers) | [What's New](#-whats-new-in-v0107) | [Download](#-download) | [Why TrustOS](#-why-trustos) | [Features](#-features) | [Quick Start](#-quick-start) | [Roadmap](#-roadmap) | [Changelog](#-changelog)

---

</div>

## 🌍 Source Code Translation for Developers

> **New in v0.10.4** — TrustOS ships its entire source code in **4 developer-friendly versions**, auto-generated from one canonical codebase.

Every `.rs` file in `kernel/src/` (500 files, 286K+ lines) is automatically translated into 4 presets — each one compiles, each one is a valid representation of TrustOS:

| Version | What it does | Who it's for |
|---------|-------------|-------------|
| **`translated/original/`** | Exact copy of the real source | Reference / diff baseline |
| **`translated/minimal/`** | All comments stripped, identifiers shortened to minimal names (`buf` → `b`) | Study pure code structure without noise |
| **`translated/educational-en/`** | Abbreviations expanded (`buf` → `buffer`, `ctx` → `context`, `irq` → `interrupt_request`) + inline English annotations on every `unsafe`, `static`, `impl`, `trait`, `enum`, `#![no_std]`… | **Learn Rust & OS dev in English** |
| **`translated/educational-fr/`** | Same expansions + **French annotations** on every Rust pattern | **Apprendre Rust & l'OS dev en français** |

### How it works

A custom **tokenizer-based Rust source translator** ([`tools/source_translator.py`](tools/source_translator.py), 1,500+ lines) parses every token, detects renameable identifiers vs. protected positions (keywords, `.field` access, `asm!()` blocks, FFI, external crate paths…), and applies preset-specific transformations while guaranteeing the output compiles.

```powershell
# Generate all 4 versions:
python tools/source_translator.py --all

# Generate a single version:
python tools/source_translator.py --preset educational --lang en -i kernel/src/ -o translated/educational-en/

# Or French:
python tools/source_translator.py --preset educational --lang fr -i kernel/src/ -o translated/educational-fr/
```

Every `translated/*/` folder includes a `mapping.json` with thousands of identifier mappings — fully deterministic, reloadable, and diffable.

> **Why this matters:** TrustOS is 286K+ lines of bare-metal Rust. The educational versions turn it into the largest annotated Rust OS learning resource that exists — in two languages.

## 🆕 What's New in v0.10.7

> **March 25, 2026** — AMD PSP Driver, Boot Timing & GPU Debug Toolkit

Massive AMD GPU firmware push: PSP (Platform Security Processor) driver, Polaris staging, SDMA expansion, and a full GPU debug command suite.

- 🔒 **AMD PSP driver** — New 791-line Platform Security Processor driver (`psp.rs`). PSP status, init, SOS boot, firmware staging pipeline. Shell commands: `gpufw psp`, `gpufw psp init`, `gpufw psp boot`, `gpufw psp sos`.
- 🛠️ **AMD GPU firmware overhaul** — `firmware.rs` rewritten (+1,515 lines): PSP boot sequences, VRAM physical address tracking, Polaris full init pipeline, firmware staging from blob to GPU memory.
- 🎮 **Polaris GPU staging** — New `_polaris_staged.rs` (207 lines): GFX/SDMA/SMU/IH microcode staging for Polaris GPUs.
- ⚡ **SDMA engine expansion** — `sdma.rs` +464 lines: full diagnostic suite, staged init, ring buffer management, firmware load, alloc/reset/test subcommands.
- 🕐 **Boot timing infrastructure** — TSC-based profiling with `boot_timing!` macro. Per-subsystem boot time measurement displayed at startup. TSC frequency auto-calibration (~2000 MHz Celeron G1610).
- 🔧 **GPU debug toolkit** — 10+ new `gpu` subcommands: `dump`, `pci`, `probe`, `vramregs`, `mmio read/write`, `mc diag/setup`, `vram`, `sdma` (status/alloc/reset/fw/ring/test/init), `heap`.
- 🧠 **AMD GPU mod.rs expanded** — +579 lines: extended GPU detection, info export API, PCI device ID fallback hardened, VRAM size improvements.
- 📊 **Register definitions reorg** — `regs.rs` +324 lines: PSP, SDMA, MC register blocks added.
- 🌐 **NVIDIA NV50+ driver expanded** — +214 lines of register probing and diagnostics.
- 📈 **286,000+ lines** across 500 source files, 3 architectures. +4,400 lines in this release.

---

## 📥 Download

**Grab an ISO and boot it in 30 seconds:**

| Edition | ISO | Size | What's Inside | Download |
|---------|-----|------|---------------|----------|
| **TrustOS** | `trustos.iso` | ~12 MB | Full OS: desktop, networking, emulators, TrustLang, TrustLab, 200+ commands. | [**⬇ Download**](https://github.com/nathan237/TrustOS/releases/latest/download/trustos.iso) |

> **For developers:** The full source code is also available in [**4 translated versions**](#-source-code-translation-for-developers) (original, minimal, educational-en, educational-fr) — clone the repo and explore `translated/`.

```bash
# Boot it right now in QEMU:
qemu-system-x86_64 -cdrom trustos.iso -m 512M -cpu max -smp 4 -display gtk -vga std -serial stdio
```

### 💾 Flash to USB with [Rufus](https://rufus.ie/)

TrustOS runs bare-metal on x86_64 PCs (UEFI or Legacy BIOS), ARM64 phones/tablets, and RISC-V boards. Flash the ISO to a USB drive and boot on real hardware:

<details>
<summary><b>🔹 UEFI Boot (modern PCs, 2012+)</b></summary>

1. Download & open [**Rufus**](https://rufus.ie/) (no install needed)
2. **Device** → select your USB drive
3. **Boot selection** → click **SELECT** → pick `trustos.iso`
4. Rufus will ask: **ISO mode** or **DD Image mode** → pick **DD Image mode**
5. **Click START** — the partition scheme / target system fields stay greyed out in DD mode, that's normal
6. Wait for the write to finish (~30 seconds)
7. Reboot your PC → press **F12** / **F2** / **DEL** at startup to open the boot menu
8. Select your USB drive (it may appear as `UEFI: <USB name>`)
9. TrustOS boots! 🚀

> ⚠️ If your PC has **Secure Boot** enabled, you may need to disable it in BIOS settings first (Security → Secure Boot → Disabled). TrustOS does not use signed bootloaders.

</details>

<details>
<summary><b>🔹 Legacy BIOS Boot (older PCs, pre-2012 or CSM mode)</b></summary>

1. Download & open [**Rufus**](https://rufus.ie/) (no install needed)
2. **Device** → select your USB drive
3. **Boot selection** → click **SELECT** → pick `trustos.iso`
4. Rufus will ask: **ISO mode** or **DD Image mode** → pick **DD Image mode**
5. **Click START** — the partition scheme / target system fields stay greyed out in DD mode, that's normal
6. Wait for the write to finish (~30 seconds)
7. Reboot your PC → press **F12** / **F2** / **DEL** at startup to open the boot menu
8. Select your USB drive (non-UEFI entry, usually just the drive name without `UEFI:` prefix)
9. TrustOS boots via Legacy/CSM! 🚀

> 💡 On some PCs, you need to enable **CSM** (Compatibility Support Module) or **Legacy Boot** in BIOS settings for the Legacy option to appear.

</details>

<details>
<summary><b>🔹 Linux / macOS (dd)</b></summary>

```bash
# Replace /dev/sdX with your USB device (use lsblk to find it)
sudo dd if=trustos.iso of=/dev/sdX bs=4M status=progress conv=fsync
```

</details>

> **Why DD Image mode?** TrustOS uses a hybrid ISO (Limine bootloader) that embeds both UEFI and Legacy BIOS boot sectors directly in the image. DD mode writes the raw image byte-for-byte, preserving both boot paths. ISO mode would reformat the drive and break the hybrid layout.

All releases: [**github.com/nathan237/TrustOS/releases**](https://github.com/nathan237/TrustOS/releases)

---

## 🔑 Why TrustOS?

TrustOS does things that **no other operating system on Earth** does. Here's why you should care:

### ⚡ A native x86_64 compiler running *inside* the OS
TrustLang compiles your programs to **raw Intel machine code** and executes them — all from within the kernel. No LLVM. No GCC. No external toolchain. Write code → native binary → execute. The entire compiler is 3,600 lines of Rust.

### 🔍 286,000 lines — and you can read every single one
No binary blobs. No proprietary drivers. No hidden telemetry. **Every driver, every protocol, every pixel, every encryption algorithm** is open Rust. One developer built it. Anyone can audit it.

### 🌍 Source code that teaches itself — in your language
Every file ships in **4 auto-generated versions**: original, minimal (stripped), educational-en (annotated English), educational-fr (annotated French). A custom tokenizer-based translator expands abbreviations, adds inline explanations of every `unsafe`, `trait`, `impl`, and Rust pattern — turning 286K lines of kernel code into the **largest annotated Rust OS learning resource in existence**.

### 🖥 A real desktop OS, not a toy
144 FPS SIMD-accelerated desktop with 14+ apps, a browser with HTML/CSS/JS, Game Boy and NES emulators, 3D chess, a code editor, network security toolkit, and the most complete kernel introspection lab ever built into a bare-metal OS.

### 🏗 Everything from scratch — zero dependencies
TLS 1.3, TCP/IP, DNS, DHCP, HTTP/HTTPS, FAT32, EXT4, NVMe, AHCI, VirtIO, VT-x/AMD-V hypervisor, Ed25519 signatures, audio synthesizer — all written in pure Rust. Not a single line of C. Not a single external crate.

### 📱 Runs on 3 architectures from one codebase
x86_64 PCs, ARM64 (Android phones, Raspberry Pi), RISC-V — same source, same OS.

### 🌐 Self-replicating over the network
PXE boot a blank machine → TrustOS installs itself, pushes the kernel over TFTP, and brings up identical instances on bare hardware — automatically.

> *"The only OS where you can trace every packet, every pixel, and every keystroke back to its source code — in your language."*

---

## 🏆 Feature Highlights

These are things no other bare-metal OS does:

### 1. Native x86_64 Compiler Running Inside the OS

TrustLang is a full programming language **built into the kernel** — with a lexer, parser, AST compiler, bytecode VM, **and a native x86_64 machine code backend**. You can write a program, compile it to raw Intel instructions, and execute it — all from within TrustOS, with zero external tools.

```
trustlang compile my_app.tl   → Compiles to native x86_64 and executes
trustlang run my_app.tl       → Bytecode VM execution
trustlang bench               → Benchmark native vs VM (shows speedup)
trustlang test                → 55+ automated test suite
```

3,612 lines. Lexer → Parser → AST → Bytecode OR native x86_64. Dual execution backends.

### 2. Self-Replicating Over the Network

TrustOS can PXE boot new machines, push the kernel binary over TFTP, and bring up identical TrustOS instances on bare hardware — automatically. Plug in a blank PC, turn it on, and TrustOS deploys itself.

### 3. Real-Time Kernel Introspection (TrustLab)

A 7-panel interactive lab that lets you watch the kernel at work in real-time: hardware status, kernel trace bus, file system tree, hex editor, TrustLang editor with syntax highlighting, execution pipeline. No other bare-metal OS ships this.

### 4. Everything From Scratch

TLS 1.3, TCP/IP, DNS, DHCP, HTTP, HTML/CSS parser, JavaScript engine, FAT32, EXT4, NVMe, AHCI, VirtIO, Game Boy emulator, NES emulator, 3D renderer, audio synthesizer, Ed25519 signatures, chess AI — all written in Rust, from zero, with no external libraries.

### 5. Source Code Translation for Developers

Every `.rs` file auto-translated into 4 versions (original, minimal, educational-en, educational-fr) by a custom tokenizer-based translator. 500 files × 4 presets = 2,000 compilable Rust files. Abbreviations expanded, inline annotations in English or French on every Rust pattern. The entire codebase becomes a learning tool. [**→ Learn more**](#-source-code-translation-for-developers)

### 6. Userland Conformance Audit

The `userland-audit` command runs an **85-check static audit** across 22 categories (Ring 3 execution, syscalls, memory, processes, file I/O, signals, IPC, networking, time, scheduling, ELF loading, address space isolation, exception handling, GDT/TSS, security, filesystem, threading, epoll, resource limits, random, system info, multi-architecture) plus **9 live Ring 3 execution tests** (write+exit, ELF load, brk+mmap, pipe IPC, signals, getpid+clock_gettime, exception safety, frame leak detection, address space isolation). Result: **81 PASS, 4 PARTIAL, 0 MISSING**.

---

##  Features

### COSMIC2 Desktop Environment
- Multi-layer GPU compositor: 8 rendering layers, SSE2 SIMD blitting, 144 FPS
- Taskbar, dock, start menu, window manager with 4px chrome borders
- 28 desktop app types: Terminal, Files, TrustCode, Calculator, Network, Snake, Chess 3D, TrustBrowser, Settings, Image Viewer, Hex Viewer, Music Player, Binary Viewer, Lab Mode, WiFi, 3D Demo…
- Touch & gesture input for mobile/tablet deployment

### TrustLang — Built-in Programming Language
- Rust-inspired syntax: `fn`, `let`, `mut`, `if/else`, `while`, `for`, `return`, `struct`
- Full pipeline: Lexer → Parser → AST → Compiler → Bytecode VM **+ native x86_64 backend**
- 20 builtin functions: `print`, `pixel`, `fill_rect`, `draw_circle`, `screen_w`, `sleep`…
- REPL, file execution, syntax checker, native benchmark
- 3,612 lines, dual execution mode (interpreted + native)

### Network Stack (all from scratch)
- VirtIO-net driver, TCP/IP (ARP, DHCP, DNS, TCP, UDP, ICMP), IPv6 + ICMPv6
- TLS 1.3 — full handshake, X.509 certificate parsing, all crypto from scratch
- HTTP/HTTPS client + HTTP server (`httpd start/stop/status`)
- Live-tested against real internet (google.com nmap + curl verified)

### TrustScan — Network Security Toolkit
Port scanner, packet sniffer, banner grabber, host discovery, traceroute, vulnerability scanner.

### TrustLab — Kernel Introspection
7-panel interactive workspace: Hardware Status, Kernel Trace, Command Guide, File System Tree, TrustLang Editor, Execution Pipeline, Hex Editor. Zero-cost 512-slot trace bus. Launch: `trustlab`

### Emulators
- **Game Boy Color** — Full CGB: LR35902 CPU, all 501 opcodes, scanline PPU, MBC1/3/5 + GameLab analysis dashboard
- **NES** — MOS 6502 (151 official + 8 unofficial opcodes), 2C02 PPU, mappers 0-3

### Hypervisor & Linux Compatibility
- **Intel VT-x** and **AMD-V** dual-backend with EPT/NPT
- **104 Linux-compatible syscalls**, ELF64 loader, Ring 3 userland, PTY/TTY, job control

### Ring 3 Userland
- Full SYSCALL/SYSRET Ring 3 execution with per-process page tables
- ELF64 loader with PIE relocations, System V ABI stack (argc/argv/auxv)
- `trustos-syscall` crate: tri-architecture syscall library (x86_64/aarch64/riscv64)
- `userland-audit`: 85-check conformance audit + 9 live Ring 3 tests
- COW fork, signals, pipes, BSD sockets, epoll, futex from userspace

### Security & Crypto
- Ed25519 signatures (RFC 8032) — full public-key cryptography
- TLS 1.3 — handshake, AEAD, X.509, all from scratch
- Capability-based security model

### GPU Drivers
- **AMD RDNA1** (Navi10/14) — PCI detection, MMIO mapping, VRAM probe, SDMA engine, compute dispatch, neural GEMM, PSP boot, firmware staging. PCI device ID fallback. Polaris support staged.
- **NVIDIA NV50+** — PMC/PBUS register probing, GPU dump, diagnostic output
- **VirtIO-GPU** — QEMU/KVM standard, 2D scanout

### World of Ants (WOA)
- Roguelike platformer game engine built into the kernel
- 60 FPS fixed-timestep game loop with TSC timing
- Zone-based procedural world generation, physics engine (gravity/velocity/acceleration)
- 1280×800 backbuffer renderer with pixel-art sprite system
- Launch: `woa`

### Remote Desktop
- UDP-based remote shell (port 7777) with screencap (port 7779)
- Full input injection: keyboard, mouse (relative & absolute), string input
- Remote ACPI reboot command
- Python client: `scripts/remote_console.py`

### Storage
- **TrustFS** — native bare-metal filesystem with WAL journal, indirect blocks
- **FAT32** — read/write for USB/disk interoperability
- **EXT4** — read-only for Linux compatibility
- **RamFS**, **DevFS**, **ProcFS** — in-memory filesystems
- **Persistence** — raw AHCI sector storage for cross-reboot file survival

### Multi-Architecture

| Target | Arch | Method | Status |
|--------|------|--------|--------|
| PC (USB/ISO) | x86_64 | Limine UEFI + Legacy BIOS (hybrid) | **Production** — tested on ThinkPad T61, ASUS B75 |
| VirtualBox / QEMU | x86_64 | Virtual machine | **Production** — daily development |
| PXE network boot | x86_64 | TFTP push to bare metal | **Working** — ASUS B75 via 10.0.0.x |
| Android | ARM64 | `fastboot flash boot` | Experimental |
| Raspberry Pi | ARM64 | SD card (kernel8.img) | Experimental |
| RISC-V boards | RISC-V | OpenSBI + U-Boot | WIP stubs |

---

## 📊 By the Numbers

| | |
|---|---|
| **286,000+ lines** of pure Rust | **500** source files |
| **3** architectures (x86_64, ARM64, RISC-V) | **< 1 sec** boot time |
| **104** Linux-compatible syscalls | **0** lines of C |
| **277+** shell commands | **0** binary blobs |
| **4** translated source versions (EN/FR) | **2,000** auto-generated compilable files |
| **85** userland conformance checks | **9** live Ring 3 tests |
| **28** desktop applications | **144 FPS** SIMD desktop |
| **4.4M** parameter AI (JARVIS) | **0** external runtime dependencies |

---

## 🚀 Quick Start

### Option A: Download & run (30 seconds)

1. Grab the ISO from [**Releases**](https://github.com/nathan237/TrustOS/releases) (or use the [Download](#-download) links above)
2. Boot it:

```bash
qemu-system-x86_64 -cdrom trustos.iso -m 512M -cpu max -smp 4 -display gtk -vga std -serial stdio
```

> Or flash to USB with **Rufus (DD Image mode)** / `dd` and boot on real hardware.
> Works on both **UEFI** and **Legacy BIOS** machines.

### Option B: Build from source

```powershell
git clone https://github.com/nathan237/TrustOS.git
cd TrustOS

# Build the kernel
cargo build --release -p trustos_kernel

# Generate all 4 translated source versions
python tools/source_translator.py --all
```

### First commands

| Command | What it does |
|---------|-------------|
| `desktop` | Launch COSMIC2 desktop environment |
| `woa` | Launch World of Ants game |
| `trustlang demo` | Run TrustLang demo program |
| `trustlang test` | Run 55+ native backend tests |
| `trustlab` | Open kernel introspection lab |
| `showcase` | Automated feature tour |
| `neofetch` | System info |
| `chess3d` | 3D chess vs AI |
| `gameboy` | Game Boy Color emulator |
| `userland-audit` | Full userland conformance audit (85 checks + 9 live tests) |
| `usertest` | Ring 3 execution test suite |
| `gpu info` | GPU driver status |
| `help` | All 277+ commands |

### Explore the translated source

```
translated/
  original/          → Exact source copy (reference)
  minimal/           → Stripped identifiers, no comments
  educational-en/    → Expanded names + English annotations
  educational-fr/    → Expanded names + French annotations
```

Each folder contains the full 500-file kernel — compilable, diffable, and annotated for learning.

---

## 📁 Project Structure

```
TrustOS/
  kernel/                     # Core bare-metal kernel (286K+ lines)
    src/                      # 500 .rs files
      trustlang/              # TrustLang compiler + native x86_64 (3,612 lines)
      shell/                  # 277+ commands
      desktop.rs              # COSMIC2 desktop manager
      woa/                    # World of Ants game engine
      jarvis/                 # JARVIS AI — 4.4M param transformer
      hwdiag/                 # Hardware diagnostics (23 subcommands)
      netstack/               # TCP/IP, DHCP, DNS, UDP, ARP, IPv6
      browser/                # HTML/CSS/JS browser engine
      gameboy/                # Game Boy Color emulator
      nes/                    # NES emulator
      hypervisor/             # VT-x/SVM, EPT/NPT
      vfs/                    # TrustFS, FAT32, EXT4, procfs
      tls13/                  # TLS 1.3, crypto, X.509
      netscan/                # Network security toolkit
      drivers/                # AHCI, USB, VirtIO, NVMe, AMD GPU (PSP/SDMA/Polaris), NVIDIA, RTL8169
      debug/                  # Remote shell (UDP 7777), serial, POST codes
  translated/                 # Auto-generated source versions
    original/                 # Exact copy (reference)
    minimal/                  # Stripped identifiers, no comments
    educational-en/           # Expanded + English annotations
    educational-fr/           # Expanded + French annotations
  tools/
    source_translator.py      # Tokenizer-based Rust translator (1,500+ lines)
    woa_prototype.html        # WOA game design prototype
  userland/                   # Userspace programs
  scripts/
    remote_console.py         # Remote shell client (UDP)
    remote_screen.py          # Remote screencap client
    pxe_server.py             # PXE/TFTP deployment
    build/                    # Build scripts (limine, multiarch)
    launch/                   # VM launch scripts (QEMU, VBox)
  docs/                       # Documentation, roadmaps, guides
  builds/
    trustos/                  # ISO output
  firmware/                   # UEFI firmware (OVMF)
  limine/                     # Bootloader binaries
  apple/                      # iOS security research
  sdk/                        # Cross-compilation SDK
```

---

## 📋 Changelog

### v0.10.7 — AMD PSP Driver, Boot Timing & GPU Debug Toolkit (March 25, 2026)

- **AMD PSP driver** — New 791-line Platform Security Processor driver. PSP status reporting, init sequence, SOS boot, firmware staging pipeline. Commands: `gpufw psp`, `gpufw psp init/boot/sos`.
- **AMD GPU firmware overhaul** — `firmware.rs` rewritten (+1,515 lines): PSP boot sequences, VRAM physical address tracking, Polaris full init, firmware staging from blob to GPU memory.
- **Polaris GPU staging** — New `_polaris_staged.rs` (207 lines): GFX/SDMA/SMU/IH microcode staging for Polaris GPUs.
- **SDMA engine expansion** — +464 lines: full diagnostic suite, staged init, ring buffer management, firmware load, 6 subcommands (status, alloc, reset, fw, ring, test).
- **Boot timing infrastructure** — TSC-based `boot_timing!` macro for per-subsystem profiling. TSC frequency auto-calibration.
- **GPU debug toolkit** — 10+ new `gpu` subcommands: dump, pci, probe, vramregs, mmio, mc, vram, sdma, heap.
- **Register definitions reorg** — +324 lines: PSP, SDMA, MC register blocks.
- **NVIDIA NV50+ expanded** — +214 lines of register probing.
- **+4,400 lines** across 30 files. 286,000+ lines total, 500 source files.

### v0.10.6 — Hardware Drivers, Remote Desktop & WOA (March 24, 2026)

- **World of Ants (WOA)** — New roguelike platformer game engine. 60 FPS fixed-timestep loop, TSC timing, zone-based world generation, physics, 1280×800 renderer, pixel-art sprites. Feature-gated (`woa`), enabled by default.
- **Remote Desktop** — Full remote input injection via UDP 7777: `key:`, `keys:`, `mouse:`, `mouseto:`, `mousepos`. Screencap on port 7779. Remote ACPI reboot. Python client `scripts/remote_console.py`.
- **AMD GPU driver hardened** — PCI device ID fallback (Navi10/14 detection without firmware). VRAM register fix (24-bit mask). Known VRAM size table. GPU init deferred after network (MMIO conflict with RTL8168).
- **NVIDIA NV50+ driver expanded** — +214 lines of register probing and diagnostics.
- **RTL8139/RTL8169 drivers rewritten** — +514 lines. Proper PCI device ID discrimination. Hardware-tested on B75 board.
- **Visual POST codes** — Real-time boot progress on framebuffer (0x11–0xFF) for hardware debugging.
- **Boot sequence reordered** — GPU after network, DHCP 800ms timeout, security subsystem deferred.
- **Input injection API** — `push_key()`, `inject_usb_mouse()`, `get_position()`.
- **283,000+ lines** across 498 source files. *(Superseded by v0.10.7)*

### v0.10.5 — Full Userland Integration & Conformance Audit (March 19, 2026)

- **104 Linux-compatible syscalls** — handle_full() dispatcher covers file I/O (24), process/thread (30), memory (4), networking (14), signals (4), epoll (5), scheduling (3), time (3), sync (3), resources (3), random, plus 4 TrustOS-specific syscalls.
- **`trustos-syscall` crate** — Tri-architecture syscall library: `syscall` (x86_64), `svc #0` (aarch64), `ecall` (riscv64). 30+ wrappers: write, read, fork, exec, mmap, brk, socket, pipe, signals, time.
- **CpuContext portability** — Per-architecture register structs for x86_64, aarch64, riscv64. No generic fallback.
- **`userland-audit` command** — 85-check conformance audit (22 categories) + 9 live Ring 3 tests. Result: 81 PASS, 4 PARTIAL, 0 MISSING.
- **`usertest` command** — 9-test Ring 3 suite: basic exec, ELF load, brk/mmap, pipe IPC, signals, stdio, exception safety, frame leak detection, address space isolation.
- **Init process rewrite** — PID 1 now uses real Linux syscalls via trustos-syscall.
- **hello-rust program** — New Ring 3 hello world in Rust.

### v0.10.4 — Universal Hardware Debugger Expansion (March 18, 2026)

- **SMBIOS/DMI parser** — Scans memory for SMBIOS 2.x/3.0 entry points. Identifies board, BIOS, chassis, CPU, DIMM slots (532 lines).
- **ATA SMART reader** — IDE PIO + AHCI DMA SMART reads. 50+ attributes, threshold comparison, pre-failure detection (543 lines). New AHCI driver functions: `send_smart_command()`, `smart_read_data()`.
- **HTML report generator** — Self-contained HTML with embedded CSS dark theme. SMBIOS, CPU, DIMM table, PCI, SMART, thermals, EFI. USB FAT32 export (280 lines).
- **EFI/UEFI probe** — Boot mode, Secure Boot, firmware vendor, UEFI version. Memory scan for EFI System Table post-ExitBootServices (249 lines).
- **EDID display parser** — Intel GMBUS I2C EDID read. Monitor manufacturer, model, native resolution, physical size, modes (370 lines).
- **ACPI battery & thermal zones** — EC battery (ThinkPad + generic), AC adapter, 8 thermal sensors, MSR temps, sleep states, EC hex dump (351 lines).
- **MARIONET probe integration** — SystemData enriched with SMBIOS, SMART, EFI, battery, thermal zones.
- **Hypervisor safety** — AMD SVM: all `unwrap()` removed. VT-x: error handling improvements.
- **HwDbg: 23 subcommands** — up from 17. New: `smbios`, `smart`, `efi`, `edid`, `battery`, `report`.
- **+4,650 lines** of new hardware diagnostic code.

### v0.10.3 — Source Code Translation & ThinkPad Hardware (March 15, 2026)

- **Source Code Translation System** — Custom tokenizer-based Rust translator (`tools/source_translator.py`, 1,500+ lines). Auto-generates 4 versions of the entire kernel source (480+ files): original, minimal, educational-en, educational-fr. Educational versions expand 120+ abbreviations and add inline annotations on every Rust pattern (`unsafe`, `trait`, `impl`, `enum`, `#![no_std]`…). All versions compile. Unified via `.\trustos.ps1 translate`.
- **ThinkPad EC driver** — Embedded Controller communication (ports 0x62/0x66) with IBF/OBF handshake and timeout. 8 temperature sensors, fan level get/set (0-7, auto, full speed, off), fan RPM readout.
- **CPU frequency/voltage** — Intel Enhanced SpeedStep via MSR 0x198/0x199. Read current freq/voltage, set P-states by FID/VID, predefined T61 Core 2 Duo profiles. CPU DTS thermal readout via MSR 0x19C.
- **HDA speaker path** — Fixed critical bug where only the headphone route had `conn_sel` set. Speaker path NID 18→10→4 never received audio. All output paths now fully configured.
- **HDA GPIO1 polarity** — T61 needs GPIO1=HIGH for amp power (not LOW like HP laptops). Confirmed via hardware testing.
- **HDA Amp Param Override** — Per spec 7.3.4.7, use AFG amp caps when widget override bit is clear. AD1984 widgets returned non-zero caps with numsteps=0.
- **Shell scrollback** — Backspace tracking, raw pixel suggestion rendering, auto-snap on restore. Tab autocomplete uses direct pixel clearing to avoid buffer corruption.
- **New commands:** `fan`, `temp`/`sensors`, `cpufreq`/`speedstep`.

### v0.10.1 — Desktop Apps & Shell Polish (March 14, 2026)

- **Settings GUI** — Full settings panel in COSMIC2 desktop: 8 categories with sidebar navigation. Display (resolution info, animations toggle), Sound (volume slider), Taskbar (clock/date/centered icons toggles), Personalization, Accessibility (high contrast toggle), Network, Apps, About.
- **NetScan GUI** — Network security toolkit as a tabbed desktop app: Dashboard overview, Port Scanner, Host Discovery, Packet Sniffer, Traceroute, Vulnerability Scanner. Keyboard-navigable tabs.
- **Shell scrollback fix** — `scroll_down()` now redraws at offset=0 (was a no-op). `redraw_from_scrollback` renders the current uncommitted line at live view. Auto-snap to bottom on any non-PageUp/PageDown keypress. New `restore_live_view()` API.
- **Tab autocomplete cursor fix** — `clear_suggestions_at_row()` left cursor on suggestion rows; Tab/Enter now restore cursor to input row before clearing.
- **ACPI shutdown hardening** — Try QEMU (0x604), Bochs (0xB004), VirtualBox (0x4004), Cloud Hypervisor (0x600) shutdown ports before standard ACPI PM1a multi-type scan.
- **Desktop icon rename** — "Chess 3D" replaces generic "Games".

### v0.10.0 — Real Hardware Stability (March 13, 2026)

- **T61 desktop freeze fixed** — `serial::read_byte()` now checks `SERIAL_PRESENT`; UART noise no longer spins the keyboard loop. Desktop input loop capped at 32 keys/frame.
- **HDA audio on ICH8** — Codec wake timeout 50ms with retry, AD198x/CX205xx quirks, EAPD, proper L+R amp unmute.
- **SIMD/FPU safety** — `fninit` + `ldmxcsr` in `enable_sse()` masks all SIMD/FPU exceptions from dirty BIOS state. New IDT handlers for #NM(7), #SS(12), #MF(16), #XM(19).
- **Fallible buffer allocs** — Double buffer, background cache, GL depth buffer use `try_reserve_exact` (no OOM panic).
- **Diagnostic framework** — Step-by-step visual diagnostics written directly to raw framebuffer for hardware debugging.

### v0.9.6 — T61 Desktop Crash Fix (March 13, 2026)

- **MXCSR/FPU init** — Real hardware BIOS leaves dirty SIMD/FPU state; now clean-initialized.
- **Missing IDT handlers** — Added #NM, #SS, #MF, #XM exception handlers.
- **Fallible framebuffer allocs** — `init_double_buffer` / `init_background_cache` use `try_reserve_exact`.
- **Autocomplete bounds check** — Shell autocomplete no longer panics on edge cases.

### v0.9.5 — Hardware Compatibility Hardening (March 13, 2026)

- **Serial port detection** — Loopback test + write timeout; no-UART systems boot without hangs.
- **APIC timer fallback** — PIT calibration failure → 1000 ticks/ms fallback; timer always starts.
- **4K OOM protection** — Backbuffer capped at 16 MB; 4K+ goes direct framebuffer.
- **BPP validation** — Non-32bpp framebuffers warned and handled gracefully.

### v0.9.4 — Keyboard Stability Fix (March 13, 2026)

- **Deadlock fix** — Keyboard IRQ + shell input buffer lock race eliminated with `without_interrupts` + `try_lock`.
- **Bootstrap guard** — Keyboard IRQ handlers gated behind `BOOTSTRAP_READY` (matches timer handler pattern).
- **i8042 timeout 10x** — 100K → 1M spin iterations for slow PS/2 controllers.
- **Pause/Break key** — E1 prefix (6-byte sequence) properly consumed.
- **Right Ctrl/Alt release tracking** — Extended modifier releases no longer silently dropped.
- **PS/2 response filter** — Added 0xAB to spurious scancode filter.

### v0.9.3 — Taskbar Fix & Framebuffer Cleanup (March 12, 2026)

- **Taskbar tray spacing** — Fixed tray icon overlap.
- **Aura removed** — Cleaned framebuffer effects.

### v0.9.2 — Legacy BIOS Boot & Audio Viz (March 12, 2026)

- **Legacy BIOS boot support** — Hybrid ISO now boots on PCs without UEFI. `limine bios-install`, Rock Ridge `-R -r -J` ISO flags, `limine.conf` + `limine.cfg` dual config.
- **Taskbar overlap fix** — Desktop windows no longer render behind the taskbar.
- **Audio visualizer** — Real-time waveform display in desktop.
- **TrustLang VM improvements** — Better error handling, optimized execution.

### v0.9.1 — Music Player Overhaul (March 12, 2026)

- **TrustPlayer** — New music player with waveform visualizer, playlist, playback controls.
- **Auto-tier fix** — Hardware tier detection corrected.

### v0.9.0 — Boot Fix, Transparent Logo, Chrome Browser (March 12, 2026)

- **Fix boot ALLOC ERROR crash** — CHECKPOINTS and BOOT_MEMORY_MAP replaced with fixed-size arrays (no heap before allocator init).
- **Fix build pipeline** — xorriso stderr no longer kills PowerShell script.
- **Transparent logo** — Logo renders over matrix rain (luminance threshold skip).
- **Chrome-style browser** — Tab bar, omnibox, rounded buttons, lock icon, 3-dot menu.
- **File manager** — Details/Tiles views, sidebar, search, column sorting.
- 30 files changed, ~5500 insertions, ~570 deletions.

### v0.8.0 — Native Compiler & Desktop Polish (March 12, 2026)

- **TrustLang native x86_64 backend** — compile `.tl` programs directly to Intel machine code and execute in-kernel. Dual execution: bytecode VM + native. 55+ automated tests + cross-validation + benchmarking.
- **x86_64 assembler module** (`x86asm.rs`) — full instruction emitter: MOV, ADD, SUB, IMUL, IDIV, CMP, Jcc, SETcc, CALL, RET, function prologue/epilogue.
- **Desktop border refinement** — window borders thickened to 4px for a bolder, more modern look.
- **Shell commands**: `trustlang compile`, `trustlang test`, `trustlang bench`.
- **Selftest integration** — native compiler smoke test added to the 96-test diagnostic suite.

### v0.7.0 — checkm8 & Multi-Edition (March 2026)

- **Project reorganization**: root cleaned from 400+ files to 11.
- **On-device AI engine** — 4.4M-parameter transformer architecture (work in progress).
- **checkm8 SecureROM exploit** — bare-metal xHCI USB exploit for Apple A12 DFU mode.
- **Apple hardware drivers** — AIC + UART for native Apple silicon.
- **ARM64 GICv2** — full interrupt controller + exception vectors.
- **COSMIC2 Desktop refresh** — redesigned windows, transparency, icons.

### v0.6.0 — Multi-Arch & Universal Boot (February 2026)

- Multi-Architecture: x86_64, aarch64, riscv64 from one codebase.
- Android boot: boot.img v2 pipeline, `fastboot flash boot`.
- Raspberry Pi SD: bare-metal RPi 4/5.
- Universal installer: 9 targets, one script.
- Touch & gesture input.

### v0.5.0 — CyberLab (February 2026)

- Live network scan verified on google.com — 11/11 tests passed.
- Code optimization (−2,800 lines).
- 95/96 auto-tests (99%).

### v0.4.x — Emulators & GameLab (February 2026)

- Game Boy Color emulator — full CGB: LR35902, scanline PPU, MBC1/3/5.
- GameLab — 2,000-line analysis dashboard.
- NES emulator — 6502 CPU, 2C02 PPU, mappers 0-3.
- Shell scripting, HTTP server, TrustPkg, TrustScan, IPv6.

### v0.3.x — Foundation (February 2026)

- ACPI, PIC, PIT, RTC, PTY/TTY, job control, NVMe swap, SMP.

### v0.2.0 — Userspace (February 2026)

- Ring 3 execution, ELF64 loader, TrustFS.

### v0.1.x — Initial Development (February 2026)

- TrustLab, COSMIC2 desktop, Ed25519, 3D Chess, audio synth, web sandbox.

---

## � Roadmap

> **Current status: v0.10.7** — March 25, 2026

TrustOS development follows a milestone-based approach. Here's where we are and where we're going:

### ✅ Completed (v0.1 → v0.10.6)

| Milestone | Status | Version |
|-----------|--------|---------|
| Kernel bootstrap (Limine, GDT, IDT, memory) | ✅ Done | v0.1 |
| Framebuffer, keyboard, shell | ✅ Done | v0.1 |
| COSMIC2 desktop (28 app types, 144 FPS compositor) | ✅ Done | v0.1–v0.10.7 |
| TrustLang compiler (bytecode VM + native x86_64) | ✅ Done | v0.8 |
| Network stack (TCP/IP, DHCP, DNS, ARP, IPv6) | ✅ Done | v0.3–v0.5 |
| TLS 1.3 + HTTP/HTTPS (all crypto from scratch) | ✅ Done | v0.4 |
| Emulators (Game Boy Color, NES) | ✅ Done | v0.4 |
| TrustLab kernel introspection (7 panels) | ✅ Done | v0.1 |
| Multi-architecture (x86_64 production, ARM64/RISC-V experimental) | ✅ Done | v0.6 |
| VT-x/AMD-V hypervisor (EPT/NPT) | ✅ Done | v0.4 |
| Ring 3 userland (104 syscalls, ELF64, SYSCALL/SYSRET) | ✅ Done | v0.2–v0.10.5 |
| Source code translation (4 versions, EN/FR) | ✅ Done | v0.10.3 |
| Hardware diagnostics (SMBIOS, SMART, EDID, thermal) | ✅ Done | v0.10.4 |
| GPU drivers (AMD RDNA1 + PSP + Polaris, NVIDIA NV50+, VirtIO) | ✅ Done | v0.7–v0.10.7 |
| Remote desktop (UDP input injection + screencap) | ✅ Done | v0.10.6 |
| WOA game engine (roguelike platformer) | ✅ Done | v0.10.6 |
| Real hardware boot (ThinkPad T61, ASUS B75 via PXE) | ✅ Done | v0.10–v0.10.6 |
| PXE network self-deployment | ✅ Done | v0.7 |

### 🔧 In Progress

| Milestone | Status | Details |
|-----------|--------|---------|
| **GPU compute for JARVIS** | 🔧 Active | AMD SDMA + compute dispatch + PSP boot + firmware staging implemented. Working on hardware validation on B75 board (fans spin, GPU detected, MMIO mapped, PSP init sequence in progress). Goal: train JARVIS on GPU instead of CPU. |
| **JARVIS training on hardware** | 🔧 Active | 4.4M-param transformer, gradient descent functional on CPU. Next: GPU-accelerated training via SDMA/neural GEMM. |
| **RTL8168 GbE stability** | 🔧 Active | Driver rewritten, hardware-tested on B75. Boot ordering fixed. Still tuning DMA ring buffers under load. |
| **WiFi (iwl4965)** | 🔧 Active | PCI detection + firmware load attempted. APM/BSM init WIP on ThinkPad T61. |

### 🎯 Next Up

| Milestone | Priority | Description |
|-----------|----------|-------------|
| **JARVIS GPU training** | 🔴 High | Complete AMD compute pipeline → train JARVIS on real GPU hardware |
| **Persistent filesystem** | 🔴 High | TrustFS WAL journal + block cache → files survive reboot reliably |
| **Audio on hardware** | 🟡 Medium | HDA audio path working on T61, needs mixer/volume + more codec support |
| **WiFi connectivity** | 🟡 Medium | iwl4965 firmware upload + scan/associate/DHCP |
| **WOA content** | 🟡 Medium | Procedural levels, enemies, items, collision system |
| **USB mass storage** | 🟡 Medium | BOT protocol over xHCI → read/write USB drives |
| **Security hardening** | 🟢 Low | Re-enable SMEP/CR0.WP after driver init, add remote shell auth |
| **Multi-user / login** | 🟢 Low | User accounts, permissions, login screen |
| **SMP scheduling** | 🟢 Low | Multi-core task scheduling (SMP init done, scheduler needs per-CPU queues) |

### 🌟 Long-Term Vision

- **Self-hosting** — Compile TrustOS from within TrustOS (TrustLang → native Rust subset)
- **JARVIS autonomy** — On-device AI that learns from hardware probing, user interaction, and federated mesh
- **Mesh networking** — Multiple TrustOS instances discover each other, share JARVIS weights
- **Mobile-first desktop** — Touch-optimized COSMIC2 on ARM64 phones/tablets
- **Full Linux binary compat** — Run unmodified Linux ELF binaries via syscall translation

---

## �📖 Documentation

| Document | Location |
|----------|----------|
| Developer Guide | [`docs/DEVELOPER_GUIDE.md`](docs/DEVELOPER_GUIDE.md) |
| Contributing | [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) |
| Roadmap V2 | [`docs/ROADMAP_V2.md`](docs/ROADMAP_V2.md) |
| Release Notes | [`docs/RELEASE_NOTES.md`](docs/RELEASE_NOTES.md) |
| Command Reference | [`docs/TRUSTOS_COMPLETE_COMMAND_REFERENCE.md`](docs/TRUSTOS_COMPLETE_COMMAND_REFERENCE.md) |
| Usage Guide | [`docs/TRUSTOS_USAGE_GUIDE.md`](docs/TRUSTOS_USAGE_GUIDE.md) |
| Source Translator | [`tools/source_translator.py`](tools/source_translator.py) |
| Translated Source (EN) | [`translated/educational-en/`](translated/educational-en/) |
| Translated Source (FR) | [`translated/educational-fr/`](translated/educational-fr/) |

---

## 🤝 Contributing

Contributions welcome. TrustOS is designed to be readable and hackable.

> See [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) for the full guide.

```bash
git clone https://github.com/YOUR_USERNAME/TrustOS.git
git checkout -b feature/my-feature
cargo build --release -p trustos_kernel
# Test in QEMU, then open a Pull Request
```

---

## 🤖 AI Disclosure

This project is built using **GitHub Copilot (Claude)** in VS Code agent mode. The AI generates the majority of the code from my prompts and architectural decisions. I design, direct, debug on real hardware, and review the output. This is a solo learning/experimental project, not production software.

---

## 📄 License

Apache License 2.0 — see [LICENSE](LICENSE).

---

## 👤 Author

**Nated0ge** — Sole creator & developer of TrustOS

- GitHub: [@nathan237](https://github.com/nathan237)
- Project: [TrustOS](https://github.com/nathan237/TrustOS)

---

<div align="center">

**Trust** the code. **Rust** is the reason.

Created by [Nated0ge](https://github.com/nathan237) — March 2026

283,000+ lines | 104 syscalls | 3 architectures | 276+ commands | Everything from scratch | Zero C

[Report Bug](https://github.com/nathan237/TrustOS/issues) | [Request Feature](https://github.com/nathan237/TrustOS/issues) | [Watch Demo](https://youtu.be/RBJJi8jW1_g)

</div>
