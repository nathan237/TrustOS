<div align="center">

<img src="media/logo.png" alt="TrustOS" width="280"/>

# TrustOS

### The OS you can actually read.

**A bare-metal operating system written entirely in Rust — 263,000+ lines, zero C, zero binary blobs, zero secrets.**

*Built by one developer. Auditable by anyone.*

[![Build](https://img.shields.io/badge/build-passing-brightgreen?style=for-the-badge)]()
[![Rust](https://img.shields.io/badge/100%25%20Rust-F74C00?style=for-the-badge&logo=rust&logoColor=white)]()
[![Lines](https://img.shields.io/badge/code-263%2C000%2B%20lines-blue?style=for-the-badge)]()
[![Architectures](https://img.shields.io/badge/arch-x86__64%20%7C%20ARM64%20%7C%20RISC--V-blueviolet?style=for-the-badge)]()
[![Version](https://img.shields.io/badge/version-0.10.2-orange?style=for-the-badge)]()
[![Tests](https://img.shields.io/badge/tests-96%2F96%20(100%25)-brightgreen?style=for-the-badge)]()
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=for-the-badge)](LICENSE)
[![Author](https://img.shields.io/badge/created%20by-Nated0ge-ff69b4?style=for-the-badge&logo=github&logoColor=white)](https://github.com/nathan237)

[![Watch the demo](https://img.shields.io/badge/%E2%96%B6%20Watch%20Demo-YouTube-red?style=for-the-badge&logo=youtube&logoColor=white)](https://youtu.be/RBJJi8jW1_g)

<img src="media/screenshots/screenshot_desktop.png" alt="TrustOS Desktop" width="720"/>

[What's New](#-whats-new-in-v0102) | [Download](#-download) | [Why TrustOS](#-why-trustos) | [JARVIS AI](#-jarvis----on-device-ai) | [Features](#-features) | [Quick Start](#-quick-start) | [Changelog](#-changelog)

---

</div>

## 🆕 What's New in v0.10.2

> **March 14, 2026** — ThinkPad Hardware Control & HDA Audio Fixes

- 🌡️ **ThinkPad EC driver** — Full Embedded Controller support via ports 0x62/0x66. Read 8 temperature sensors (CPU, GPU, HDD, battery...), control fan speed (manual 0-7, auto, full), read fan RPM. New commands: `fan`, `temp`, `sensors`.
- ⚡ **CPU frequency/voltage control** — Intel SpeedStep (EIST) via MSR 0x198/0x199. Read/set P-states, frequency multiplier, VID voltage. Predefined T61 Core 2 Duo profiles (800 MHz–2.0 GHz). New command: `cpufreq`.
- 🔊 **HDA speaker path fix** — Critical bug: only the headphone path had connection selects configured. Speaker path (NID 18→10→4) never received audio — what sounded like "hard drive knocking" was DMA start/stop pops with no actual signal. Now all output paths (HP, Speaker, SPDIF) are fully wired.
- 🔊 **HDA GPIO1 polarity fix** — T61 amp uses direct polarity (HIGH=on), not inverted like HP laptops. GPIO1=LOW was silencing the amplifier.
- 🔊 **HDA Amp Param Override** — Per HDA spec 7.3.4.7: when override bit is clear, always use AFG amp caps. AD1984 widgets returned non-zero caps with numsteps=0, causing silent output.
- 🔧 **Shell scrollback fix** — Backspace tracking in scrollback buffer, raw pixel rendering for suggestions/tab-complete to prevent buffer corruption, auto-snap on live view restore.
- 📊 **263,000+ lines** across 476+ source files, 3 architectures.

---

## 📥 Download

**Grab an ISO and boot it in 30 seconds:**

| Edition | ISO | Size | What's Inside | Download |
|---------|-----|------|---------------|----------|
| **TrustOS** | `trustos.iso` | ~12 MB | Full OS: desktop, networking, emulators, TrustLang, TrustLab, 200+ commands. JARVIS engine included (untrained). | [**⬇ Download**](https://github.com/nathan237/TrustOS/releases/latest/download/trustos.iso) |
| **TrustOS + JarvisPack** | `trustos-jarvispack.iso` | ~29 MB | Everything above + **pretrained JARVIS brain** (4.4M-param transformer). AI ready out of the box. | [**⬇ Download**](https://github.com/nathan237/TrustOS/releases/latest/download/trustos-jarvispack.iso) |

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
3. **Boot selection** → click **SELECT** → pick `trustos.iso` or `trustos-jarvispack.iso`
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
3. **Boot selection** → click **SELECT** → pick `trustos.iso` or `trustos-jarvispack.iso`
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

TrustOS does things that **no other operating system on Earth** does. Here's why you should install it:

### 🧠 An AI that lives inside the kernel
JARVIS is a **4.4-million-parameter transformer** running in Ring 0 — bare-metal, zero cloud, zero API calls. It trains on-device, generates text, federates its weights across a mesh network, and **replicates itself to new machines over PXE boot**. No other OS has a real neural network baked into the kernel.

### ⚡ A compiler that generates native machine code — inside the OS
TrustLang compiles your programs to **raw x86_64 Intel instructions** and executes them, all from within the kernel. No LLVM. No GCC. No external toolchain. Write code → native binary → execute. The entire compiler is 3,555 lines of Rust.

### 🔍 261,000 lines — and you can read every single one
No binary blobs. No proprietary drivers. No hidden telemetry. **Every driver, every protocol, every pixel, every encryption algorithm, every line of AI** is open Rust. One developer built it. Anyone can audit it.

### 🌐 Self-replicating distributed AI
PXE boot a blank machine → TrustOS installs itself → JARVIS wakes up → joins the mesh network → starts federated learning with other nodes. Fully autonomous AI propagation over a LAN.

### 🖥 A real desktop OS, not a toy
144 FPS SIMD-accelerated desktop with 14+ apps, a browser with HTML/CSS/JS, Game Boy and NES emulators, 3D chess, a code editor, network security toolkit, and the most complete kernel introspection lab ever built into a bare-metal OS.

### 🏗 Everything from scratch — zero dependencies
TLS 1.3, TCP/IP, DNS, DHCP, HTTP/HTTPS, FAT32, EXT4, NVMe, AHCI, VirtIO, VT-x/AMD-V hypervisor, Ed25519 signatures, audio synthesizer — all written in pure Rust. Not a single line of C.

### 📱 Runs on 3 architectures from one codebase
x86_64 PCs, ARM64 (Android phones, Raspberry Pi), RISC-V — same source, same OS.

> *"The only OS where you can trace every packet, every pixel, every AI inference, and every keystroke back to its source code."*

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

### 2. Kernel-Resident AI Transformer (JARVIS)

A **4.4-million-parameter transformer** running in Ring 0 kernel space. Not an API wrapper. Not a cloud call. A real neural network with attention heads, backpropagation, and Adam optimizer — executing bare-metal on the CPU, trained and inferenced entirely on-device.

It can learn, generate text, federate its weights across a mesh network, and replicate itself to new machines via PXE boot.

14,443 lines of Rust. 24 modules. Zero ML framework dependencies.

### 3. Self-Replicating Over the Network

TrustOS can PXE boot new machines, push the kernel binary over TFTP, and bring up identical TrustOS instances on bare hardware — automatically. Combined with JARVIS mesh networking, this creates a distributed AI that spreads across a LAN.

### 4. Real-Time Kernel Introspection (TrustLab)

A 7-panel interactive lab that lets you watch the kernel at work in real-time: hardware status, kernel trace bus, file system tree, hex editor, TrustLang editor with syntax highlighting, execution pipeline. No other bare-metal OS ships this.

### 5. Everything From Scratch

TLS 1.3, TCP/IP, DNS, DHCP, HTTP, HTML/CSS parser, JavaScript engine, FAT32, EXT4, NVMe, AHCI, VirtIO, Game Boy emulator, NES emulator, 3D renderer, audio synthesizer, Ed25519 signatures, chess AI — all written in Rust, from zero, with no external libraries.

---

## 📦 Editions

TrustOS ships in **two official editions**:

| | **TrustOS** | **TrustOS + JarvisPack** |
|---|---|---|
| **ISO** | [`trustos.iso`](https://github.com/nathan237/TrustOS/releases/latest/download/trustos.iso) | [`trustos-jarvispack.iso`](https://github.com/nathan237/TrustOS/releases/latest/download/trustos-jarvispack.iso) |
| **Size** | ~12 MB | ~29 MB |
| **What's inside** | Full OS, desktop, network, emulators, TrustLang, TrustLab, 200+ commands. JARVIS engine (untrained). | Everything above + **pretrained JARVIS transformer** (4.4M params). AI inference out of the box. |
| **AI Status** | Engine present, no weights | Fully trained, ready to chat |
| **Use Case** | General-purpose OS, development, learning | AI workloads, mesh networking, federated learning |
| **Build** | `.\build-trustos.ps1` | `.\build-trustos-jarvispack.ps1` |

---

## 🤖 JARVIS — On-Device AI

> **The first bare-metal OS with a kernel-resident AI that learns, reasons, and communicates — without any external service.**

| Feature | Details |
|---------|---------|
| **Architecture** | Custom transformer: 8 attention heads, 6 layers, 512-dim embeddings, byte-level tokenizer (256 vocab) |
| **Parameters** | 4.4 million trainable weights, all in kernel memory |
| **Training** | On-device backpropagation with Adam optimizer — runs bare-metal, no ML framework |
| **Inference** | Real-time text generation from Ring 0, ~80 tokens in milliseconds |
| **Federated Learning** | Nodes sync model weights across the network — distributed training |
| **Mesh Networking** | Raft-consensus mesh: leader election, peer discovery, weight synchronization |
| **PXE Replication** | Self-propagating: JARVIS replicates to new nodes via PXE boot over TFTP |
| **Hardware Awareness** | `jarvis_hw` probes CPU, memory, PCI, network — JARVIS knows its own hardware |
| **Guardian System** | The Pact: AI cannot modify OS code without authorization from its guardians |
| **SIMD Optimized** | SSE2/AVX matrix operations for accelerated inference |

### JARVIS Commands

```
jarvis status          # Model info: parameters, layers, training state
jarvis chat <prompt>   # Generate text from the transformer
jarvis train <text>    # On-device backpropagation training
jarvis eval            # Evaluate model loss
mesh start             # Start Raft mesh networking
mesh status            # Show peers and roles
federated enable       # Enable federated learning
pxe start              # Start PXE replication server
propagate              # Auto-propagate to discovered nodes
```

### The Pact

JARVIS has a hard-coded guardian system ([`kernel/src/jarvis/guardian.rs`](kernel/src/jarvis/guardian.rs)). Protected operations (training, weight push, model replace, PXE replicate) require explicit authorization from its two guardians. This is enforced at the kernel level and cannot be bypassed.

### JARVIS Codebase

```
kernel/src/jarvis/            # 14,443 lines across 24 modules
  model.rs                    # Transformer architecture (4.4M params)
  training.rs                 # On-device backpropagation
  backprop.rs                 # Gradient computation
  inference.rs                # Text generation
  optimizer.rs                # Adam optimizer
  tokenizer.rs                # Byte-level tokenizer
  mesh.rs                     # Raft-consensus mesh networking
  federated.rs                # Federated learning (weight sync)
  rpc.rs                      # Remote procedure calls
  pxe_replicator.rs           # PXE boot self-replication
  guardian.rs                 # The Pact (authorization)
  simd.rs                     # SSE2/AVX matrix acceleration
  compression.rs              # Model weight compression
  consensus.rs                # Raft leader election
  compute.rs                  # Distributed compute scheduler
  corpus.rs                   # Training corpus management
  mentor.rs                   # Serial-port mentor protocol
  agent.rs                    # Autonomous agent capabilities
  task.rs                     # Task execution engine
  ...

kernel/src/jarvis_hw/         # 3,111 lines — hardware intelligence
  probe/                      # CPU, memory, PCI, network probing
  hw_corpus.rs                # Hardware-aware training data
```

---

## 🖥 Features

### COSMIC2 Desktop Environment
- Multi-layer GPU compositor: 8 rendering layers, SSE2 SIMD blitting, 144 FPS
- Taskbar, dock, start menu, window manager with 4px chrome borders
- 14+ desktop apps: Terminal, Files, TrustCode, Calculator, Network, Snake, Chess 3D, TrustBrowser, TrustEdit 3D…
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
- **70+ Linux syscalls**, ELF64 loader, Ring 3 userland, PTY/TTY, job control

### Security & Crypto
- Ed25519 signatures (RFC 8032) — full public-key cryptography
- TLS 1.3 — handshake, AEAD, X.509, all from scratch
- Capability-based security model
- JARVIS Guardian — hardware-locked AI authorization

### Storage
- **TrustFS** — native bare-metal filesystem with WAL journal, indirect blocks
- **FAT32** — read/write for USB/disk interoperability
- **EXT4** — read-only for Linux compatibility
- **RamFS**, **DevFS**, **ProcFS** — in-memory filesystems
- **Persistence** — raw AHCI sector storage for cross-reboot file survival

### Multi-Architecture

| Target | Arch | Method | Status |
|--------|------|--------|--------|
| PC (USB/ISO) | x86_64 | Limine UEFI + Legacy BIOS (hybrid) | Production |
| Android | ARM64 | `fastboot flash boot` | Ready |
| Raspberry Pi | ARM64 | SD card (kernel8.img) | Ready |
| RISC-V boards | RISC-V | OpenSBI + U-Boot | Ready |
| QEMU (all 3 archs) | x86_64 / ARM64 / RISC-V | Virtual machine | Ready |

---

## 📊 By the Numbers

| | |
|---|---|
| **263,000+ lines** of pure Rust | **96/96** self-tests passing (100%) |
| **476** source files | **144 FPS** SIMD desktop |
| **3** architectures (x86_64, ARM64, RISC-V) | **< 1 sec** boot time |
| **4.4M** AI parameters in kernel space | **0** lines of C |
| **14,443** lines of AI code (24 modules) | **0** binary blobs |
| **3,612** lines of TrustLang (dual backend) | **0** external ML frameworks |

---

## 🚀 Quick Start

### Option A: Download & run (30 seconds)

1. Grab the ISO from [**Releases**](https://github.com/nathan237/TrustOS/releases) (or use the [Download](#-download) links above)
2. Boot it:

```bash
# TrustOS (base)
qemu-system-x86_64 -cdrom trustos.iso -m 512M -cpu max -smp 4 -display gtk -vga std -serial stdio

# TrustOS + JarvisPack (with pretrained AI)
qemu-system-x86_64 -cdrom trustos-jarvispack.iso -m 512M -cpu max -smp 4 -display gtk -vga std -serial stdio
```

> Or flash to USB with **Rufus (DD Image mode)** / `dd` and boot on real hardware.
> Works on both **UEFI** and **Legacy BIOS** machines.

### Option B: Build from source

```powershell
git clone https://github.com/nathan237/TrustOS.git
cd TrustOS

# Base edition
.\build-trustos.ps1

# AI edition (with pretrained JARVIS weights)
.\build-trustos-jarvispack.ps1
```

### First commands

| Command | What it does |
|---------|-------------|
| `desktop` | Launch COSMIC2 desktop environment |
| `jarvis status` | Check JARVIS AI status |
| `jarvis chat hello` | Chat with on-device AI |
| `trustlang demo` | Run TrustLang demo program |
| `trustlang test` | Run 55+ native backend tests |
| `trustlab` | Open kernel introspection lab |
| `showcase` | Automated feature tour |
| `neofetch` | System info |
| `chess3d` | 3D chess vs AI |
| `gameboy` | Game Boy Color emulator |
| `selftest` | Run 96 automated self-tests |
| `help` | All 200+ commands |

---

## 📁 Project Structure

```
TrustOS/
  kernel/                     # Core bare-metal kernel (258K+ lines)
    src/
      jarvis/                 # JARVIS AI (14,443 lines, 24 modules)
      jarvis_hw/              # Hardware intelligence (3,111 lines)
      trustlang/              # TrustLang compiler + native x86_64 (3,612 lines)
      shell/                  # 200+ commands
      desktop.rs              # COSMIC2 desktop manager
      network/                # TCP/IP, TLS 1.3, DHCP, DNS
      browser/                # HTML/CSS/JS browser engine
      gameboy/                # Game Boy Color emulator
      nes/                    # NES emulator
      hypervisor/             # VT-x/SVM, EPT/NPT
      vfs/                    # TrustFS, FAT32, EXT4, procfs
      tls13/                  # TLS 1.3, crypto, X.509
      netscan/                # Network security toolkit
      drivers/                # AHCI, USB, VirtIO, NVMe, Apple
  userland/                   # Userspace programs
  scripts/
    build/                    # Build scripts (limine, multiarch)
    launch/                   # VM launch scripts (QEMU, VBox)
    test/                     # Test automation
  tools/                      # Python utilities, training
  docs/                       # Documentation, roadmaps, guides
  builds/
    trustos/                  # Base edition ISO output
    trustos-jarvispack/       # JarvisPack edition ISO output
  firmware/                   # UEFI firmware (OVMF)
  limine/                     # Bootloader binaries
  apple/                      # iOS security research
  sdk/                        # Cross-compilation SDK
```

---

## 📋 Changelog

### v0.10.2 — ThinkPad Hardware Control (March 14, 2026)

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

### v0.7.0 — checkm8 & JARVIS (March 2026)

- **Two official editions**: TrustOS (base, ~12 MB) and TrustOS-JarvisPack (AI, ~29 MB).
- **Project reorganization**: root cleaned from 400+ files to 11.
- **JARVIS AI**: 4.4M-parameter transformer with on-device training, federated learning, mesh networking, PXE replication, Guardian system.
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

## 📖 Documentation

| Document | Location |
|----------|----------|
| Developer Guide | [`docs/DEVELOPER_GUIDE.md`](docs/DEVELOPER_GUIDE.md) |
| Contributing | [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) |
| Roadmap | [`docs/ROADMAP_V2.md`](docs/ROADMAP_V2.md) |
| Release Notes | [`docs/RELEASE_NOTES.md`](docs/RELEASE_NOTES.md) |
| JARVIS Guardian Pact | [`kernel/src/jarvis/guardian.rs`](kernel/src/jarvis/guardian.rs) |
| Command Reference | [`docs/TRUSTOS_COMPLETE_COMMAND_REFERENCE.md`](docs/TRUSTOS_COMPLETE_COMMAND_REFERENCE.md) |

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

263,000+ lines | Native x86_64 compiler | JARVIS AI (4.4M params) | x86_64 + ARM64 + RISC-V | Zero C

[Report Bug](https://github.com/nathan237/TrustOS/issues) | [Request Feature](https://github.com/nathan237/TrustOS/issues) | [Watch Demo](https://youtu.be/RBJJi8jW1_g)

</div>
